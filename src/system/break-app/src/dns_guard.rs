use std::fs;

use break_enforcer::pf::{STATUS_OK, STATUS_PATH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum DnsGuardStatus {
    Off,
    Applied,
    Failed,
}

pub fn current() -> DnsGuardStatus {
    read(fs::read_to_string(STATUS_PATH).ok().as_deref())
}

pub fn read(status: Option<&str>) -> DnsGuardStatus {
    match status.map(str::trim) {
        None => DnsGuardStatus::Off,
        Some(STATUS_OK) => DnsGuardStatus::Applied,
        Some(_) => DnsGuardStatus::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_status_file_means_the_guard_is_not_in_play() {
        assert_eq!(read(None), DnsGuardStatus::Off);
    }

    #[test]
    fn the_daemon_reporting_ok_shows_as_applied() {
        assert_eq!(read(Some("ok")), DnsGuardStatus::Applied);
        assert_eq!(read(Some("ok\n")), DnsGuardStatus::Applied);
    }

    #[test]
    fn any_other_message_is_a_failure() {
        assert_eq!(
            read(Some("방화벽 설정에 실패했습니다 (pfctl -E): ...")),
            DnsGuardStatus::Failed
        );
    }

    #[test]
    fn an_empty_status_file_is_a_failure_not_a_success() {
        assert_eq!(read(Some("")), DnsGuardStatus::Failed);
        assert_eq!(read(Some("   \n")), DnsGuardStatus::Failed);
    }
}
