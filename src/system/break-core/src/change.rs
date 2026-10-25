use chrono::{DateTime, Duration, Local, Utc};

use crate::rules::{AlertRecipient, Rules};
use crate::schedule::{instant_active, window_contains};

pub const RELEASE_DELAY_SECONDS: i64 = 24 * 60 * 60;
pub const TEST_RELEASE_DELAY_SECONDS: i64 = 60;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ChangeError {
    #[error("실행 중인 시간대는 끄거나 바꾸거나 지울 수 없습니다")]
    RunningWindow,
    #[error("실행 중인 차단이 쓰는 그룹에서는 주소를 빼거나 그룹을 지울 수 없습니다")]
    LockedGroup,
    #[error("즉시 차단은 끝 시각을 앞당기거나 그룹을 뺄 수 없습니다")]
    InstantShortened,
    #[error("유해사이트 차단 설정은 이 요청으로 바꿀 수 없습니다")]
    AdultFilterTouched,
    #[error("알림 주소는 이 요청으로 바꿀 수 없습니다")]
    AlertsTouched,
    #[error("비밀번호는 이 요청으로 바꿀 수 없습니다")]
    LockTouched,
}

pub fn release_due(requested_at: DateTime<Utc>, now: DateTime<Utc>, delay: Duration) -> bool {
    now >= requested_at + delay
}

pub fn check_save(old: &Rules, new: &Rules, now: DateTime<Utc>) -> Result<(), ChangeError> {
    if new.adult_filter != old.adult_filter {
        return Err(ChangeError::AdultFilterTouched);
    }
    if new.alerts != old.alerts {
        return Err(ChangeError::AlertsTouched);
    }
    if new.lock != old.lock {
        return Err(ChangeError::LockTouched);
    }

    let local = now.with_timezone(&Local);
    let mut locked_groups: Vec<&String> = Vec::new();

    for window in old
        .schedule
        .windows
        .iter()
        .filter(|window| window_contains(window, &local))
    {
        let kept = new
            .schedule
            .windows
            .iter()
            .find(|candidate| candidate.id == window.id)
            .ok_or(ChangeError::RunningWindow)?;
        let same_shape = kept.enabled
            && kept.start == window.start
            && kept.end == window.end
            && window.days.iter().all(|day| kept.days.contains(day));
        let same_groups = window
            .group_ids
            .iter()
            .all(|id| kept.group_ids.contains(id));
        if !same_shape || !same_groups {
            return Err(ChangeError::RunningWindow);
        }
        locked_groups.extend(window.group_ids.iter());
    }

    if instant_active(old, &now) {
        let (Some(before), Some(after)) = (&old.instant, &new.instant) else {
            return Err(ChangeError::InstantShortened);
        };
        let groups_kept = before
            .group_ids
            .iter()
            .all(|id| after.group_ids.contains(id));
        if after.ends_at < before.ends_at || !groups_kept {
            return Err(ChangeError::InstantShortened);
        }
        locked_groups.extend(before.group_ids.iter());
    }

    for group_id in locked_groups {
        let Some(before) = old.group(group_id) else {
            continue;
        };
        let after = new.group(group_id).ok_or(ChangeError::LockedGroup)?;
        let sites_kept = before
            .sites
            .iter()
            .all(|site| after.sites.iter().any(|kept| kept.host == site.host));
        if !sites_kept {
            return Err(ChangeError::LockedGroup);
        }
    }

    Ok(())
}

pub fn apply_due_releases(rules: &mut Rules, now: DateTime<Utc>, delay: Duration) -> Vec<String> {
    let mut released = Vec::new();

    if let Some(requested_at) = rules.adult_filter.disable_requested_at {
        if release_due(requested_at, now, delay) {
            rules.adult_filter.enabled = false;
            rules.adult_filter.disable_requested_at = None;
            released.push("유해사이트 차단이 꺼졌습니다".to_string());
        }
    }

    let (due, kept): (Vec<AlertRecipient>, Vec<AlertRecipient>) =
        std::mem::take(&mut rules.alerts.recipients)
            .into_iter()
            .partition(|recipient| {
                recipient
                    .removal_requested_at
                    .is_some_and(|requested_at| release_due(requested_at, now, delay))
            });
    rules.alerts.recipients = kept;
    for recipient in due {
        released.push(format!("알림 주소 {}를 뺐습니다", recipient.email));
    }

    released
}
