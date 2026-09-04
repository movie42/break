use break_core::rules::{Rules, Schedule, SiteGroup, SiteTarget, TimeOfDay, TimeWindow, Weekday};
use break_core::schedule::{blocked_sites_at, is_blocking_at, window_contains};
use chrono::{Local, TimeZone};

fn window(start: (u8, u8), end: (u8, u8), days: &[Weekday]) -> TimeWindow {
    TimeWindow {
        id: "w".to_string(),
        start: TimeOfDay::new(start.0, start.1),
        end: TimeOfDay::new(end.0, end.1),
        days: days.to_vec(),
        group_ids: Vec::new(),
        enabled: true,
    }
}

fn window_for(
    start: (u8, u8),
    end: (u8, u8),
    days: &[Weekday],
    group_ids: &[&str],
) -> TimeWindow {
    TimeWindow {
        group_ids: group_ids.iter().map(|id| (*id).to_string()).collect(),
        ..window(start, end, days)
    }
}

fn group(id: &str, hosts: &[&str]) -> SiteGroup {
    SiteGroup {
        id: id.to_string(),
        name: id.to_string(),
        sites: hosts
            .iter()
            .map(|host| SiteTarget {
                host: (*host).to_string(),
            })
            .collect(),
        apps: Vec::new(),
    }
}

fn hosts(rules: &Rules, at: &chrono::DateTime<Local>) -> Vec<String> {
    blocked_sites_at(rules, at)
        .into_iter()
        .map(|site| site.host)
        .collect()
}

fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> chrono::DateTime<Local> {
    Local
        .with_ymd_and_hms(year, month, day, hour, minute, 0)
        .single()
        .expect("로컬 시각 변환 실패")
}

fn monday(hour: u32, minute: u32) -> chrono::DateTime<Local> {
    at(2026, 8, 31, hour, minute)
}

fn tuesday(hour: u32, minute: u32) -> chrono::DateTime<Local> {
    at(2026, 9, 1, hour, minute)
}

#[test]
fn same_day_window_uses_half_open_boundaries() {
    let w = window((9, 0), (18, 0), &[Weekday::Monday]);
    assert!(!window_contains(&w, &monday(8, 59)));
    assert!(window_contains(&w, &monday(9, 0)));
    assert!(window_contains(&w, &monday(17, 59)));
    assert!(!window_contains(&w, &monday(18, 0)));
}

#[test]
fn overnight_window_follows_the_start_day() {
    let w = window((22, 0), (2, 0), &[Weekday::Monday]);
    assert!(window_contains(&w, &monday(22, 0)));
    assert!(window_contains(&w, &monday(23, 59)));
    assert!(window_contains(&w, &tuesday(1, 0)));
    assert!(!window_contains(&w, &tuesday(2, 0)));
    assert!(!window_contains(&w, &monday(1, 0)));
}

#[test]
fn equal_start_and_end_blocks_the_whole_day() {
    let w = window((0, 0), (0, 0), &[Weekday::Monday]);
    assert!(window_contains(&w, &monday(0, 0)));
    assert!(window_contains(&w, &monday(12, 30)));
    assert!(window_contains(&w, &monday(23, 59)));
    assert!(!window_contains(&w, &tuesday(12, 30)));
}

#[test]
fn day_off_is_never_blocked() {
    let w = window((9, 0), (18, 0), &[Weekday::Tuesday]);
    assert!(!window_contains(&w, &monday(12, 0)));
}

#[test]
fn invalid_time_never_blocks() {
    let w = window((25, 0), (99, 99), &[Weekday::Monday]);
    assert!(!window_contains(&w, &monday(12, 0)));
}

#[test]
fn overlapping_windows_block_if_any_matches() {
    let rules = Rules {
        schedule: Schedule {
            windows: vec![
                window((9, 0), (12, 0), &[Weekday::Monday]),
                window((11, 0), (18, 0), &[Weekday::Monday]),
            ],
        },
        ..Rules::default()
    };
    assert!(is_blocking_at(&rules, &monday(11, 30)));
    assert!(is_blocking_at(&rules, &monday(17, 0)));
    assert!(!is_blocking_at(&rules, &monday(18, 0)));
}

#[test]
fn no_windows_means_no_blocking() {
    assert!(!is_blocking_at(&Rules::default(), &monday(12, 0)));
}

#[test]
fn only_groups_linked_to_an_active_window_are_blocked() {
    let rules = Rules {
        groups: vec![
            group("social", &["youtube.com"]),
            group("shop", &["coupang.com"]),
        ],
        schedule: Schedule {
            windows: vec![
                window_for((9, 0), (12, 0), &[Weekday::Monday], &["social"]),
                window_for((13, 0), (18, 0), &[Weekday::Monday], &["shop"]),
            ],
        },
        ..Rules::default()
    };

    assert_eq!(hosts(&rules, &monday(10, 0)), vec!["youtube.com"]);
    assert_eq!(hosts(&rules, &monday(14, 0)), vec!["coupang.com"]);
    assert!(hosts(&rules, &monday(12, 30)).is_empty());
}

#[test]
fn overlapping_windows_block_the_union_once() {
    let rules = Rules {
        groups: vec![
            group("social", &["youtube.com", "x.com"]),
            group("shop", &["coupang.com", "x.com"]),
        ],
        schedule: Schedule {
            windows: vec![
                window_for((9, 0), (12, 0), &[Weekday::Monday], &["social"]),
                window_for((11, 0), (18, 0), &[Weekday::Monday], &["shop"]),
            ],
        },
        ..Rules::default()
    };

    assert_eq!(
        hosts(&rules, &monday(11, 30)),
        vec!["coupang.com", "x.com", "youtube.com"]
    );
}

#[test]
fn a_window_linked_to_nothing_blocks_nothing() {
    let rules = Rules {
        groups: vec![group("social", &["youtube.com"])],
        schedule: Schedule {
            windows: vec![window((0, 0), (0, 0), &[Weekday::Monday])],
        },
        ..Rules::default()
    };

    assert!(is_blocking_at(&rules, &monday(12, 0)));
    assert!(hosts(&rules, &monday(12, 0)).is_empty());
}

#[test]
fn an_overnight_window_still_blocks_after_midnight() {
    let rules = Rules {
        groups: vec![group("social", &["youtube.com"])],
        schedule: Schedule {
            windows: vec![window_for((22, 0), (2, 0), &[Weekday::Monday], &["social"])],
        },
        ..Rules::default()
    };

    assert_eq!(hosts(&rules, &tuesday(1, 0)), vec!["youtube.com"]);
    assert!(hosts(&rules, &tuesday(2, 0)).is_empty());
}

#[test]
fn an_off_window_blocks_nothing() {
    let mut off = window_for((0, 0), (0, 0), &[Weekday::Monday], &["social"]);
    off.enabled = false;

    let rules = Rules {
        groups: vec![group("social", &["youtube.com"])],
        schedule: Schedule {
            windows: vec![off],
        },
        ..Rules::default()
    };

    assert!(!window_contains(&rules.schedule.windows[0], &monday(12, 0)));
    assert!(!is_blocking_at(&rules, &monday(12, 0)));
    assert!(hosts(&rules, &monday(12, 0)).is_empty());
}

#[test]
fn a_file_without_enabled_reads_as_on() {
    let json = r#"{
        "version": 2,
        "groups": [{ "id": "social", "name": "social", "sites": [{ "host": "youtube.com" }], "apps": [] }],
        "schedule": {
            "windows": [{
                "id": "w",
                "start": { "hour": 0, "minute": 0 },
                "end": { "hour": 0, "minute": 0 },
                "days": ["monday"],
                "groupIds": ["social"]
            }]
        }
    }"#;

    let rules: Rules = serde_json::from_str(json).expect("규칙 파싱 실패");

    assert!(rules.schedule.windows[0].enabled);
    assert_eq!(hosts(&rules, &monday(12, 0)), vec!["youtube.com"]);
}
