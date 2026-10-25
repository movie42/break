use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::change::{self, ChangeError};
use crate::password;
use crate::rules::{AlertRecipient, Rules};

pub const SOCKET_PATH: &str = "/var/run/com.movie42.break.sock";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MailAccount {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub secret: String,
    pub from: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "op")]
pub enum Request {
    Ping,
    Save {
        rules: Rules,
    },
    SetPassword {
        password: String,
    },
    ConsentAdultFilter,
    RequestAdultFilterOff {
        password: String,
    },
    CancelAdultFilterOff,
    EnableAdultFilter,
    AddRecipient {
        email: String,
    },
    RequestRecipientRemoval {
        email: String,
        password: String,
    },
    CancelRecipientRemoval {
        email: String,
    },
    SetMailAccount {
        account: MailAccount,
        password: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub ok: bool,
    #[serde(default)]
    pub message: Option<String>,
}

impl Response {
    pub fn ok() -> Self {
        Self {
            ok: true,
            message: None,
        }
    }

    pub fn refused(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            message: Some(message.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RequestError {
    #[error(transparent)]
    Change(#[from] ChangeError),
    #[error("비밀번호가 맞지 않습니다")]
    WrongPassword,
    #[error("비밀번호를 먼저 정해야 합니다")]
    NoPassword,
    #[error("비밀번호는 이미 정해져 있습니다")]
    PasswordAlreadySet,
    #[error("{0}")]
    BadPassword(String),
    #[error("이미 동의했습니다")]
    AlreadyConsented,
    #[error("유해사이트 차단 동의를 먼저 받아야 합니다")]
    NotConsented,
    #[error("메일 주소로 읽을 수 없습니다: {0}")]
    BadEmail(String),
    #[error("이미 등록된 주소입니다")]
    DuplicateEmail,
    #[error("등록된 주소가 아닙니다")]
    UnknownEmail,
    #[error("이 요청은 차단 프로그램이 직접 처리합니다")]
    NotARulesChange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Applied {
    pub event: String,
    pub new_recipient: Option<String>,
    pub removal_notice: Option<String>,
}

impl Applied {
    fn event(event: impl Into<String>) -> Self {
        Self {
            event: event.into(),
            new_recipient: None,
            removal_notice: None,
        }
    }
}

pub fn normalize_email(input: &str) -> Option<String> {
    let email = input.trim().to_ascii_lowercase();
    let (local, domain) = email.split_once('@')?;
    let valid = !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !email.chars().any(char::is_whitespace)
        && email.matches('@').count() == 1;
    valid.then_some(email)
}

fn check_password(rules: &Rules, input: &str) -> Result<(), RequestError> {
    let stored = rules
        .lock
        .password_hash
        .as_deref()
        .ok_or(RequestError::NoPassword)?;
    if password::verify(input, stored) {
        Ok(())
    } else {
        Err(RequestError::WrongPassword)
    }
}

pub fn verify_password(rules: &Rules, input: &str) -> bool {
    check_password(rules, input).is_ok()
}

pub fn apply(
    rules: &mut Rules,
    request: Request,
    now: DateTime<Utc>,
) -> Result<Applied, RequestError> {
    match request {
        Request::Save { rules: next } => {
            change::check_save(rules, &next, now)?;
            *rules = next;
            Ok(Applied::event("규칙 저장"))
        }
        Request::SetPassword { password } => {
            if rules.lock.password_hash.is_some() {
                return Err(RequestError::PasswordAlreadySet);
            }
            let hashed = password::hash(&password)
                .map_err(|err| RequestError::BadPassword(err.to_string()))?;
            rules.lock.password_hash = Some(hashed);
            Ok(Applied::event("비밀번호를 정했습니다"))
        }
        Request::ConsentAdultFilter => {
            if rules.adult_filter.consented_at.is_some() {
                return Err(RequestError::AlreadyConsented);
            }
            rules.adult_filter.consented_at = Some(now);
            rules.adult_filter.enabled = true;
            Ok(Applied::event("유해사이트 차단에 동의했습니다"))
        }
        Request::RequestAdultFilterOff { password } => {
            if rules.adult_filter.consented_at.is_none() {
                return Err(RequestError::NotConsented);
            }
            check_password(rules, &password)?;
            if rules.adult_filter.enabled && rules.adult_filter.disable_requested_at.is_none() {
                rules.adult_filter.disable_requested_at = Some(now);
            }
            Ok(Applied::event("유해사이트 차단 끄기를 신청했습니다"))
        }
        Request::CancelAdultFilterOff => {
            rules.adult_filter.disable_requested_at = None;
            Ok(Applied::event("유해사이트 차단 끄기 신청을 취소했습니다"))
        }
        Request::EnableAdultFilter => {
            if rules.adult_filter.consented_at.is_none() {
                return Err(RequestError::NotConsented);
            }
            rules.adult_filter.enabled = true;
            rules.adult_filter.disable_requested_at = None;
            Ok(Applied::event("유해사이트 차단을 켰습니다"))
        }
        Request::AddRecipient { email } => {
            let email = normalize_email(&email).ok_or(RequestError::BadEmail(email))?;
            if rules
                .alerts
                .recipients
                .iter()
                .any(|recipient| recipient.email == email)
            {
                return Err(RequestError::DuplicateEmail);
            }
            rules.alerts.recipients.push(AlertRecipient {
                email: email.clone(),
                added_at: now,
                removal_requested_at: None,
            });
            Ok(Applied {
                event: format!("알림 주소 {email}를 넣었습니다"),
                new_recipient: Some(email),
                removal_notice: None,
            })
        }
        Request::RequestRecipientRemoval { email, password } => {
            check_password(rules, &password)?;
            let email = normalize_email(&email).unwrap_or(email);
            let recipient = rules
                .alerts
                .recipients
                .iter_mut()
                .find(|recipient| recipient.email == email)
                .ok_or(RequestError::UnknownEmail)?;
            if recipient.removal_requested_at.is_none() {
                recipient.removal_requested_at = Some(now);
            }
            Ok(Applied {
                event: format!("알림 주소 {email} 빼기를 신청했습니다"),
                new_recipient: None,
                removal_notice: Some(email),
            })
        }
        Request::CancelRecipientRemoval { email } => {
            let email = normalize_email(&email).unwrap_or(email);
            let recipient = rules
                .alerts
                .recipients
                .iter_mut()
                .find(|recipient| recipient.email == email)
                .ok_or(RequestError::UnknownEmail)?;
            recipient.removal_requested_at = None;
            Ok(Applied::event(format!(
                "알림 주소 {email} 빼기 신청을 취소했습니다"
            )))
        }
        Request::Ping | Request::SetMailAccount { .. } => Err(RequestError::NotARulesChange),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day() -> chrono::Duration {
        chrono::Duration::seconds(change::RELEASE_DELAY_SECONDS)
    }

    fn now() -> DateTime<Utc> {
        "2026-10-18T03:00:00Z".parse().expect("time")
    }

    fn with_password() -> Rules {
        let mut rules = Rules::default();
        apply(
            &mut rules,
            Request::SetPassword {
                password: "참을인세번".to_string(),
            },
            now(),
        )
        .expect("set password");
        rules
    }

    #[test]
    fn a_password_can_be_set_only_once() {
        let mut rules = with_password();
        let again = apply(
            &mut rules,
            Request::SetPassword {
                password: "다른비밀번호".to_string(),
            },
            now(),
        );
        assert_eq!(again, Err(RequestError::PasswordAlreadySet));
    }

    #[test]
    fn consent_turns_the_adult_filter_on() {
        let mut rules = with_password();
        apply(&mut rules, Request::ConsentAdultFilter, now()).expect("consent");
        assert!(rules.adult_filter.enabled);
        assert_eq!(rules.adult_filter.consented_at, Some(now()));
    }

    #[test]
    fn turning_the_filter_off_needs_the_password_and_waits_a_day() {
        let mut rules = with_password();
        apply(&mut rules, Request::ConsentAdultFilter, now()).expect("consent");

        let wrong = apply(
            &mut rules,
            Request::RequestAdultFilterOff {
                password: "틀림".to_string(),
            },
            now(),
        );
        assert_eq!(wrong, Err(RequestError::WrongPassword));

        apply(
            &mut rules,
            Request::RequestAdultFilterOff {
                password: "참을인세번".to_string(),
            },
            now(),
        )
        .expect("request off");
        assert!(rules.adult_filter.enabled);

        let almost = now() + chrono::Duration::hours(23);
        assert!(change::apply_due_releases(&mut rules, almost, day()).is_empty());
        assert!(rules.adult_filter.enabled);

        let day_later = now() + chrono::Duration::hours(24);
        assert_eq!(
            change::apply_due_releases(&mut rules, day_later, day()).len(),
            1
        );
        assert!(!rules.adult_filter.enabled);
    }

    #[test]
    fn a_cancelled_request_never_turns_the_filter_off() {
        let mut rules = with_password();
        apply(&mut rules, Request::ConsentAdultFilter, now()).expect("consent");
        apply(
            &mut rules,
            Request::RequestAdultFilterOff {
                password: "참을인세번".to_string(),
            },
            now(),
        )
        .expect("request off");
        apply(&mut rules, Request::CancelAdultFilterOff, now()).expect("cancel");

        change::apply_due_releases(&mut rules, now() + chrono::Duration::days(2), day());
        assert!(rules.adult_filter.enabled);
    }

    #[test]
    fn a_save_cannot_turn_the_filter_off() {
        let mut rules = with_password();
        apply(&mut rules, Request::ConsentAdultFilter, now()).expect("consent");
        let mut edited = rules.clone();
        edited.adult_filter.enabled = false;

        let result = apply(&mut rules, Request::Save { rules: edited }, now());
        assert_eq!(
            result,
            Err(RequestError::Change(ChangeError::AdultFilterTouched))
        );
        assert!(rules.adult_filter.enabled);
    }

    #[test]
    fn adding_a_recipient_is_immediate_and_removing_waits() {
        let mut rules = with_password();
        let added = apply(
            &mut rules,
            Request::AddRecipient {
                email: " Friend@Example.com ".to_string(),
            },
            now(),
        )
        .expect("add");
        assert_eq!(added.new_recipient.as_deref(), Some("friend@example.com"));

        apply(
            &mut rules,
            Request::RequestRecipientRemoval {
                email: "friend@example.com".to_string(),
                password: "참을인세번".to_string(),
            },
            now(),
        )
        .expect("request removal");
        assert_eq!(rules.alerts.recipients.len(), 1);

        change::apply_due_releases(&mut rules, now() + chrono::Duration::hours(24), day());
        assert!(rules.alerts.recipients.is_empty());
    }

    #[test]
    fn a_save_cannot_drop_a_recipient() {
        let mut rules = with_password();
        apply(
            &mut rules,
            Request::AddRecipient {
                email: "me@example.com".to_string(),
            },
            now(),
        )
        .expect("add");
        let mut edited = rules.clone();
        edited.alerts.recipients.clear();

        let result = apply(&mut rules, Request::Save { rules: edited }, now());
        assert_eq!(
            result,
            Err(RequestError::Change(ChangeError::AlertsTouched))
        );
    }

    #[test]
    fn an_email_needs_one_at_and_a_dotted_domain() {
        assert_eq!(normalize_email("a@b.co"), Some("a@b.co".to_string()));
        assert_eq!(normalize_email("a@b"), None);
        assert_eq!(normalize_email("a b@c.com"), None);
        assert_eq!(normalize_email("@c.com"), None);
        assert_eq!(normalize_email("a@@c.com"), None);
    }
}
