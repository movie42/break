use std::path::Path;

use break_core::rules::SiteTarget;
use break_core::{schedule, store};
use break_enforcer::{Enforcer, Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Blocked(Vec<String>),
    Cleared,
    RulesUnreadable(String),
    Failed(String),
}

impl Outcome {
    pub fn log_line(&self) -> String {
        match self {
            Outcome::Blocked(hosts) => format!("차단 적용: {}", hosts.join(", ")),
            Outcome::Cleared => "차단 해제".to_string(),
            Outcome::RulesUnreadable(message) => {
                format!("규칙 파일을 읽지 못해 차단 없음으로 둡니다: {message}")
            }
            Outcome::Failed(message) => format!("hosts 갱신 실패: {message}"),
        }
    }
}

pub fn desired_sites(rules_path: &Path) -> Result<Vec<SiteTarget>, String> {
    let rules = store::load_from(rules_path).map_err(|err| err.to_string())?;
    if schedule::is_blocking_now(&rules) {
        Ok(rules.sites)
    } else {
        Ok(Vec::new())
    }
}

pub fn tick<E: Enforcer>(enforcer: &E, rules_path: &Path) -> Outcome {
    let sites = match desired_sites(rules_path) {
        Ok(sites) => sites,
        Err(message) => {
            return match enforcer.clear_sites() {
                Ok(()) => Outcome::RulesUnreadable(message),
                Err(err) => Outcome::Failed(err.to_string()),
            };
        }
    };

    apply(enforcer, &sites)
}

pub fn clear<E: Enforcer>(enforcer: &E) -> Outcome {
    apply(enforcer, &[])
}

fn apply<E: Enforcer>(enforcer: &E, sites: &[SiteTarget]) -> Outcome {
    let result = if sites.is_empty() {
        enforcer.clear_sites()
    } else {
        enforcer.apply_sites(sites)
    };

    match result {
        Ok(()) if sites.is_empty() => Outcome::Cleared,
        Ok(()) => Outcome::Blocked(sites.iter().map(|site| site.host.clone()).collect()),
        Err(Error::NotPrivileged) => Outcome::Failed(Error::NotPrivileged.to_string()),
        Err(err) => Outcome::Failed(err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fs;

    use break_core::rules::{AppTarget, Rules, Schedule, TimeOfDay, TimeWindow, Weekday};

    use super::*;

    #[derive(Default)]
    struct FakeEnforcer {
        applied: RefCell<Vec<Vec<String>>>,
        cleared: RefCell<usize>,
    }

    impl Enforcer for FakeEnforcer {
        fn apply_sites(&self, sites: &[SiteTarget]) -> Result<(), Error> {
            self.applied
                .borrow_mut()
                .push(sites.iter().map(|site| site.host.clone()).collect());
            Ok(())
        }
        fn clear_sites(&self) -> Result<(), Error> {
            *self.cleared.borrow_mut() += 1;
            Ok(())
        }
        fn apply_apps(&self, _apps: &[AppTarget]) -> Result<(), Error> {
            Err(Error::NotPrivileged)
        }
        fn clear_apps(&self) -> Result<(), Error> {
            Err(Error::NotPrivileged)
        }
    }

    fn rules_file(name: &str, rules: &Rules) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("break-daemon-test-{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("rules.json");
        store::save_to(&path, rules).expect("save rules");
        path
    }

    fn always_blocking(hosts: &[&str]) -> Rules {
        Rules {
            sites: hosts
                .iter()
                .map(|host| SiteTarget {
                    host: (*host).to_string(),
                })
                .collect(),
            schedule: Schedule {
                windows: vec![TimeWindow {
                    id: "all-day".to_string(),
                    start: TimeOfDay::new(0, 0),
                    end: TimeOfDay::new(0, 0),
                    days: vec![
                        Weekday::Monday,
                        Weekday::Tuesday,
                        Weekday::Wednesday,
                        Weekday::Thursday,
                        Weekday::Friday,
                        Weekday::Saturday,
                        Weekday::Sunday,
                    ],
                }],
            },
            ..Rules::default()
        }
    }

    #[test]
    fn a_missing_rules_file_clears_the_block() {
        let enforcer = FakeEnforcer::default();
        let missing = std::env::temp_dir().join("break-daemon-test-missing/rules.json");

        assert_eq!(tick(&enforcer, &missing), Outcome::Cleared);
        assert_eq!(*enforcer.cleared.borrow(), 1);
    }

    #[test]
    fn a_window_covering_now_applies_the_sites() {
        let enforcer = FakeEnforcer::default();
        let path = rules_file("blocking", &always_blocking(&["youtube.com"]));

        assert_eq!(
            tick(&enforcer, &path),
            Outcome::Blocked(vec!["youtube.com".to_string()])
        );
        assert_eq!(*enforcer.applied.borrow(), vec![vec!["youtube.com"]]);
    }

    #[test]
    fn sites_without_any_window_are_not_blocked() {
        let enforcer = FakeEnforcer::default();
        let mut rules = always_blocking(&["youtube.com"]);
        rules.schedule.windows.clear();
        let path = rules_file("no-window", &rules);

        assert_eq!(tick(&enforcer, &path), Outcome::Cleared);
    }

    #[test]
    fn clear_never_reads_the_rules_file() {
        let enforcer = FakeEnforcer::default();
        assert_eq!(clear(&enforcer), Outcome::Cleared);
        assert_eq!(*enforcer.cleared.borrow(), 1);
    }
}
