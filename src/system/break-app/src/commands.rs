use std::path::{Path, PathBuf};

use break_core::events::{self, Event};
use break_core::protocol::{self, MailAccount, Request};
use break_core::rules::{Rules, SiteGroup, SiteTarget, RULES_VERSION};
use break_core::{mail, schedule, store};

use crate::browsers::{self, QuitReport};
use crate::daemon::status::{self, DaemonStatus};
use crate::daemon::{client, install, DaemonError};
use crate::dns_guard::{self, DnsGuardStatus};

const RECENT_EVENTS: usize = 200;
const APP_MAIL_FILE: &str = "mail.json";

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub rules: Rules,
    pub blocking_now: bool,
    pub daemon: DaemonStatus,
    pub daemon_needs_reinstall: bool,
    pub rules_path: String,
    pub rejected_sites: Vec<String>,
    pub dns_guard: DnsGuardStatus,
    pub guarded: bool,
    pub mail_account: Option<String>,
    pub release_delay_seconds: i64,
}

fn normalize(input: Rules) -> (Rules, Vec<String>) {
    let mut rejected = Vec::new();
    let mut groups = Vec::new();

    for group in input.groups {
        let mut sites: Vec<SiteTarget> = Vec::new();
        for site in group.sites {
            match SiteTarget::from_input(&site.host) {
                Some(target) => {
                    if !sites.iter().any(|kept| kept.host == target.host) {
                        sites.push(target);
                    }
                }
                None => rejected.push(site.host),
            }
        }
        groups.push(SiteGroup {
            id: group.id,
            name: group.name.trim().to_string(),
            sites,
            apps: group.apps,
            preset: group.preset,
        });
    }

    let mut normalized = Rules {
        version: RULES_VERSION,
        groups,
        schedule: input.schedule,
        instant: input.instant,
        adult_filter: input.adult_filter,
        alerts: input.alerts,
        lock: input.lock,
        legacy_sites: Vec::new(),
        legacy_apps: Vec::new(),
    };
    normalized.prune_group_links();
    normalized.drop_expired_instant(chrono::Utc::now());

    (normalized, rejected)
}

fn user_rules_path() -> Result<PathBuf, String> {
    store::rules_path().map_err(|err| err.to_string())
}

pub fn active_rules_path() -> Result<PathBuf, String> {
    let system = Path::new(store::SYSTEM_RULES_PATH);
    if system.exists() {
        Ok(system.to_path_buf())
    } else {
        user_rules_path()
    }
}

pub fn current_rules() -> Result<Rules, String> {
    store::load_from(&active_rules_path()?).map_err(|err| err.to_string())
}

fn app_mail_path() -> Result<PathBuf, String> {
    Ok(store::rules_dir()
        .map_err(|err| err.to_string())?
        .join(APP_MAIL_FILE))
}

pub fn app_mail_account() -> Option<MailAccount> {
    mail::load_account(&app_mail_path().ok()?)
}

fn to_state(rules: Rules, rejected_sites: Vec<String>) -> Result<AppState, String> {
    let rules_path = active_rules_path()?.to_string_lossy().into_owned();
    let daemon = status::current();

    Ok(AppState {
        dns_guard: dns_guard::current(),
        blocking_now: schedule::is_blocking_now(&rules),
        daemon_needs_reinstall: daemon != DaemonStatus::NotInstalled
            && !status::installed_is_current(),
        guarded: Path::new(store::SYSTEM_RULES_PATH).exists(),
        mail_account: app_mail_account().map(|account| account.from),
        release_delay_seconds: crate::daemon::plist::release_delay_seconds(),
        daemon,
        rules,
        rules_path,
        rejected_sites,
    })
}

fn reload() -> Result<AppState, String> {
    to_state(current_rules()?, Vec::new())
}

fn submit(request: Request) -> Result<(), String> {
    if client::reachable() {
        let response = client::send(&request)?;
        return if response.ok {
            Ok(())
        } else {
            Err(response
                .message
                .unwrap_or_else(|| "차단 프로그램이 요청을 거절했습니다".to_string()))
        };
    }

    if Path::new(store::SYSTEM_RULES_PATH).exists() {
        return Err(
            "차단 프로그램이 응답하지 않습니다. 잠시 뒤 다시 시도하거나 다시 설치하세요"
                .to_string(),
        );
    }
    if matches!(request, Request::SetMailAccount { .. }) {
        return Err("메일 계정은 차단 프로그램을 설치한 뒤 정할 수 있습니다".to_string());
    }

    let path = user_rules_path()?;
    let mut rules = store::load_from(&path).map_err(|err| err.to_string())?;
    protocol::apply(&mut rules, request, chrono::Utc::now()).map_err(|err| err.to_string())?;
    store::save_to(&path, &rules).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn load_rules() -> Result<AppState, String> {
    reload()
}

#[tauri::command]
pub fn save_rules(rules: Rules) -> Result<AppState, String> {
    let (normalized, rejected) = normalize(rules);
    submit(Request::Save { rules: normalized })?;
    to_state(current_rules()?, rejected)
}

#[tauri::command]
pub fn set_password(password: String) -> Result<AppState, String> {
    submit(Request::SetPassword { password })?;
    reload()
}

#[tauri::command]
pub fn consent_adult_filter() -> Result<AppState, String> {
    submit(Request::ConsentAdultFilter)?;
    reload()
}

#[tauri::command]
pub fn request_adult_filter_off(password: String) -> Result<AppState, String> {
    submit(Request::RequestAdultFilterOff { password })?;
    reload()
}

#[tauri::command]
pub fn cancel_adult_filter_off() -> Result<AppState, String> {
    submit(Request::CancelAdultFilterOff)?;
    reload()
}

#[tauri::command]
pub fn enable_adult_filter() -> Result<AppState, String> {
    submit(Request::EnableAdultFilter)?;
    reload()
}

#[tauri::command]
pub fn add_recipient(email: String) -> Result<AppState, String> {
    submit(Request::AddRecipient { email })?;
    reload()
}

#[tauri::command]
pub fn request_recipient_removal(email: String, password: String) -> Result<AppState, String> {
    submit(Request::RequestRecipientRemoval { email, password })?;
    reload()
}

#[tauri::command]
pub fn cancel_recipient_removal(email: String) -> Result<AppState, String> {
    submit(Request::CancelRecipientRemoval { email })?;
    reload()
}

#[tauri::command]
pub fn set_mail_account(
    account: MailAccount,
    password: Option<String>,
) -> Result<AppState, String> {
    submit(Request::SetMailAccount {
        account: account.clone(),
        password,
    })?;
    let path = app_mail_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    mail::save_account(&path, &account).map_err(|err| err.to_string())?;
    reload()
}

#[tauri::command]
pub fn recent_events() -> Vec<Event> {
    events::read_recent(Path::new(events::EVENTS_PATH), RECENT_EVENTS)
}

#[tauri::command]
pub fn daemon_status() -> DaemonStatus {
    status::current()
}

#[tauri::command]
pub fn install_daemon() -> Result<AppState, String> {
    let rules_path = user_rules_path()?;
    match install::install(&rules_path) {
        Ok(()) => reload(),
        Err(DaemonError::Cancelled) => Err("설치를 취소했습니다.".to_string()),
        Err(other) => Err(format!("설치에 실패했습니다: {other}")),
    }
}

#[tauri::command]
pub fn uninstall_daemon() -> Result<AppState, String> {
    let rules_path = user_rules_path()?;
    match install::uninstall(&rules_path) {
        Ok(()) => reload(),
        Err(DaemonError::Cancelled) => Err("제거를 취소했습니다.".to_string()),
        Err(other) => Err(format!("제거에 실패했습니다: {other}")),
    }
}

#[tauri::command]
pub fn running_browsers() -> Vec<String> {
    browsers::running()
}

#[tauri::command]
pub fn quit_browsers() -> QuitReport {
    browsers::quit_running()
}

#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}
