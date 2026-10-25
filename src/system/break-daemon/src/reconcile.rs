use std::path::Path;

use break_core::rules::SiteTarget;
use break_core::{schedule, store};
use break_enforcer::Enforcer;
#[cfg(test)]
use break_enforcer::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Guard {
    Applied,
    Failed(String),
}

impl Guard {
    fn log_suffix(&self) -> String {
        match self {
            Guard::Applied => " (브라우저 정책 + 보안 DNS 차단)".to_string(),
            Guard::Failed(message) => format!(" (보강 차단 실패: {message})"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Blocked(Vec<String>, usize, Guard),
    Cleared,
    RulesUnreadable(String),
    Failed(String),
}

impl Outcome {
    pub fn log_line(&self) -> String {
        match self {
            Outcome::Blocked(hosts, adult, guard) => {
                let adult = if *adult > 0 {
                    format!(" + 유해사이트 {adult}개")
                } else {
                    String::new()
                };
                format!(
                    "차단 적용: {}{adult}{}",
                    hosts.join(", "),
                    guard.log_suffix()
                )
            }
            Outcome::Cleared => "차단 해제".to_string(),
            Outcome::RulesUnreadable(message) => {
                format!("규칙 파일을 읽지 못해 차단 없음으로 둡니다: {message}")
            }
            Outcome::Failed(message) => format!("hosts 갱신 실패: {message}"),
        }
    }
}

pub fn desired(rules_path: &Path) -> Result<(Vec<SiteTarget>, bool), String> {
    let rules = store::load_from(rules_path).map_err(|err| err.to_string())?;
    Ok((
        schedule::blocked_sites_now(&rules),
        rules.adult_filter.enabled,
    ))
}

#[cfg(test)]
pub fn tick<E: Enforcer>(enforcer: &E, rules_path: &Path, adult: &[SiteTarget]) -> Outcome {
    tick_report(enforcer, rules_path, adult).0
}

pub fn tick_report<E: Enforcer>(
    enforcer: &E,
    rules_path: &Path,
    adult: &[SiteTarget],
) -> (Outcome, bool) {
    let (sites, adult_on) = match desired(rules_path) {
        Ok(desired) => desired,
        Err(message) => {
            let outcome = match clear_all(enforcer) {
                Ok(()) => Outcome::RulesUnreadable(message),
                Err(message) => Outcome::Failed(message),
            };
            return (outcome, false);
        }
    };

    apply(enforcer, &sites, if adult_on { adult } else { &[] })
}

pub fn clear<E: Enforcer>(enforcer: &E) -> Outcome {
    apply(enforcer, &[], &[]).0
}

fn apply<E: Enforcer>(enforcer: &E, sites: &[SiteTarget], adult: &[SiteTarget]) -> (Outcome, bool) {
    if sites.is_empty() && adult.is_empty() {
        let outcome = match clear_all(enforcer) {
            Ok(()) => Outcome::Cleared,
            Err(message) => Outcome::Failed(message),
        };
        return (outcome, false);
    }

    let compact: Vec<SiteTarget> = adult
        .iter()
        .filter(|site| !sites.iter().any(|kept| kept.host == site.host))
        .cloned()
        .collect();

    let hosts_changed = match enforcer.apply_sites(sites, &compact) {
        Ok(changed) => changed,
        Err(err) => return (Outcome::Failed(err.to_string()), false),
    };

    let hosts = sites.iter().map(|site| site.host.clone()).collect();
    let policy = if sites.is_empty() {
        enforcer.clear_browser_policy()
    } else {
        enforcer.apply_browser_policy(sites)
    };
    let dns = enforcer.apply_dns_guard();

    let outcome = match (policy, dns) {
        (Ok(()), Ok(())) => Outcome::Blocked(hosts, adult.len(), Guard::Applied),
        (Err(err), _) | (Ok(()), Err(err)) => {
            Outcome::Blocked(hosts, adult.len(), Guard::Failed(err.to_string()))
        }
    };
    (outcome, hosts_changed)
}

fn clear_all<E: Enforcer>(enforcer: &E) -> Result<(), String> {
    let sites = enforcer.clear_sites();
    let policy = enforcer.clear_browser_policy();
    let guard = enforcer.clear_dns_guard();

    match (sites, policy, guard) {
        (Ok(()), Ok(()), Ok(())) => Ok(()),
        (Err(err), _, _) | (Ok(()), Err(err), _) | (Ok(()), Ok(()), Err(err)) => {
            Err(err.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fs;

    use break_core::rules::{
        AppTarget, Rules, Schedule, SiteGroup, TimeOfDay, TimeWindow, Weekday,
    };

    use super::*;

    #[derive(Default)]
    struct FakeEnforcer {
        applied: RefCell<Vec<Vec<String>>>,
        compact: RefCell<Vec<Vec<String>>>,
        cleared: RefCell<usize>,
        guard_applied: RefCell<usize>,
        guard_cleared: RefCell<usize>,
        policy_applied: RefCell<Vec<Vec<String>>>,
        policy_cleared: RefCell<usize>,
        guard_error: Option<Error>,
    }

    impl FakeEnforcer {
        fn with_failing_guard() -> Self {
            Self {
                guard_error: Some(Error::NotPrivileged),
                ..Self::default()
            }
        }
    }

    impl Enforcer for FakeEnforcer {
        fn apply_sites(&self, sites: &[SiteTarget], compact: &[SiteTarget]) -> Result<bool, Error> {
            self.applied
                .borrow_mut()
                .push(sites.iter().map(|site| site.host.clone()).collect());
            self.compact
                .borrow_mut()
                .push(compact.iter().map(|site| site.host.clone()).collect());
            Ok(true)
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
        fn apply_dns_guard(&self) -> Result<(), Error> {
            *self.guard_applied.borrow_mut() += 1;
            match &self.guard_error {
                Some(Error::NotPrivileged) => Err(Error::NotPrivileged),
                _ => Ok(()),
            }
        }
        fn clear_dns_guard(&self) -> Result<(), Error> {
            *self.guard_cleared.borrow_mut() += 1;
            Ok(())
        }
        fn apply_browser_policy(&self, sites: &[SiteTarget]) -> Result<(), Error> {
            self.policy_applied
                .borrow_mut()
                .push(sites.iter().map(|site| site.host.clone()).collect());
            Ok(())
        }
        fn clear_browser_policy(&self) -> Result<(), Error> {
            *self.policy_cleared.borrow_mut() += 1;
            Ok(())
        }
    }

    fn blocked(hosts: &[&str]) -> Outcome {
        Outcome::Blocked(
            hosts.iter().map(|host| (*host).to_string()).collect(),
            0,
            Guard::Applied,
        )
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
            groups: vec![SiteGroup {
                id: "g1".to_string(),
                name: "테스트".to_string(),
                sites: hosts
                    .iter()
                    .map(|host| SiteTarget {
                        host: (*host).to_string(),
                    })
                    .collect(),
                apps: Vec::new(),
                preset: None,
            }],
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
                    group_ids: vec!["g1".to_string()],
                    enabled: true,
                }],
            },
            ..Rules::default()
        }
    }

    #[test]
    fn a_missing_rules_file_clears_the_block() {
        let enforcer = FakeEnforcer::default();
        let missing = std::env::temp_dir().join("break-daemon-test-missing/rules.json");

        assert_eq!(tick(&enforcer, &missing, &[]), Outcome::Cleared);
        assert_eq!(*enforcer.cleared.borrow(), 1);
    }

    #[test]
    fn a_window_covering_now_applies_the_sites() {
        let enforcer = FakeEnforcer::default();
        let path = rules_file("blocking", &always_blocking(&["youtube.com"]));

        assert_eq!(tick(&enforcer, &path, &[]), blocked(&["youtube.com"]));
        assert_eq!(*enforcer.applied.borrow(), vec![vec!["youtube.com"]]);
        assert_eq!(*enforcer.guard_applied.borrow(), 1);
        assert_eq!(*enforcer.policy_applied.borrow(), vec![vec!["youtube.com"]]);
    }

    #[test]
    fn sites_without_any_window_are_not_blocked() {
        let enforcer = FakeEnforcer::default();
        let mut rules = always_blocking(&["youtube.com"]);
        rules.schedule.windows.clear();
        let path = rules_file("no-window", &rules);

        assert_eq!(tick(&enforcer, &path, &[]), Outcome::Cleared);
    }

    #[test]
    fn a_window_linked_to_no_group_blocks_nothing() {
        let enforcer = FakeEnforcer::default();
        let mut rules = always_blocking(&["youtube.com"]);
        rules.schedule.windows[0].group_ids.clear();
        let path = rules_file("no-group", &rules);

        assert_eq!(tick(&enforcer, &path, &[]), Outcome::Cleared);
    }

    #[test]
    fn only_the_groups_a_window_names_are_blocked() {
        let enforcer = FakeEnforcer::default();
        let mut rules = always_blocking(&["youtube.com"]);
        rules.groups.push(SiteGroup {
            id: "g2".to_string(),
            name: "쇼핑".to_string(),
            sites: vec![SiteTarget {
                host: "coupang.com".to_string(),
            }],
            apps: Vec::new(),
            preset: None,
        });
        let path = rules_file("one-group", &rules);

        assert_eq!(tick(&enforcer, &path, &[]), blocked(&["youtube.com"]));
    }

    #[test]
    fn a_failing_dns_guard_keeps_the_hosts_block() {
        let enforcer = FakeEnforcer::with_failing_guard();
        let path = rules_file("guard-fails", &always_blocking(&["youtube.com"]));

        let outcome = tick(&enforcer, &path, &[]);
        assert_eq!(
            outcome,
            Outcome::Blocked(
                vec!["youtube.com".to_string()],
                0,
                Guard::Failed(Error::NotPrivileged.to_string())
            )
        );
        assert_eq!(*enforcer.applied.borrow(), vec![vec!["youtube.com"]]);
        assert!(outcome.log_line().contains("보강 차단 실패"));
    }

    #[test]
    fn a_blocked_log_line_names_the_dns_guard() {
        assert_eq!(
            blocked(&["youtube.com"]).log_line(),
            "차단 적용: youtube.com (브라우저 정책 + 보안 DNS 차단)"
        );
    }

    #[test]
    fn leaving_the_block_clears_the_dns_guard_too() {
        let enforcer = FakeEnforcer::default();
        let mut rules = always_blocking(&["youtube.com"]);
        rules.schedule.windows.clear();
        let path = rules_file("clears-guard", &rules);

        assert_eq!(tick(&enforcer, &path, &[]), Outcome::Cleared);
        assert_eq!(*enforcer.cleared.borrow(), 1);
        assert_eq!(*enforcer.guard_cleared.borrow(), 1);
        assert_eq!(*enforcer.policy_cleared.borrow(), 1);
    }

    #[test]
    fn clear_never_reads_the_rules_file() {
        let enforcer = FakeEnforcer::default();
        assert_eq!(clear(&enforcer), Outcome::Cleared);
        assert_eq!(*enforcer.cleared.borrow(), 1);
        assert_eq!(*enforcer.guard_cleared.borrow(), 1);
    }

    fn adult_list(hosts: &[&str]) -> Vec<SiteTarget> {
        hosts
            .iter()
            .map(|host| SiteTarget {
                host: (*host).to_string(),
            })
            .collect()
    }

    #[test]
    fn the_adult_list_goes_to_hosts_but_not_the_browser_policy() {
        let enforcer = FakeEnforcer::default();
        let mut rules = always_blocking(&["youtube.com"]);
        rules.adult_filter.enabled = true;
        let path = rules_file("adult-on", &rules);

        let outcome = tick(&enforcer, &path, &adult_list(&["adult.example"]));
        assert_eq!(
            outcome,
            Outcome::Blocked(vec!["youtube.com".to_string()], 1, Guard::Applied)
        );
        assert_eq!(*enforcer.applied.borrow(), vec![vec!["youtube.com"]]);
        assert_eq!(*enforcer.compact.borrow(), vec![vec!["adult.example"]]);
        assert_eq!(*enforcer.policy_applied.borrow(), vec![vec!["youtube.com"]]);
    }

    #[test]
    fn the_adult_list_blocks_even_with_no_window() {
        let enforcer = FakeEnforcer::default();
        let mut rules = always_blocking(&["youtube.com"]);
        rules.schedule.windows.clear();
        rules.adult_filter.enabled = true;
        let path = rules_file("adult-only", &rules);

        let outcome = tick(&enforcer, &path, &adult_list(&["adult.example"]));
        assert_eq!(outcome, Outcome::Blocked(Vec::new(), 1, Guard::Applied));
        assert_eq!(*enforcer.policy_cleared.borrow(), 1);
        assert!(enforcer.policy_applied.borrow().is_empty());
    }

    #[test]
    fn an_off_adult_filter_ignores_the_list() {
        let enforcer = FakeEnforcer::default();
        let mut rules = always_blocking(&["youtube.com"]);
        rules.schedule.windows.clear();
        let path = rules_file("adult-off", &rules);

        assert_eq!(
            tick(&enforcer, &path, &adult_list(&["adult.example"])),
            Outcome::Cleared
        );
    }

    #[test]
    fn the_log_line_counts_the_adult_list_instead_of_naming_it() {
        let outcome = Outcome::Blocked(vec!["youtube.com".to_string()], 80_000, Guard::Applied);
        assert_eq!(
            outcome.log_line(),
            "차단 적용: youtube.com + 유해사이트 80000개 (브라우저 정책 + 보안 DNS 차단)"
        );
    }
}
