use break_core::rules::{Rules, Schedule, TimeOfDay, TimeWindow, Weekday};
use break_core::schedule::{is_blocking_at, window_contains};
use chrono::{Local, TimeZone};

fn window(start: (u8, u8), end: (u8, u8), days: &[Weekday]) -> TimeWindow {
    TimeWindow {
        id: "w".to_string(),
        start: TimeOfDay::new(start.0, start.1),
        end: TimeOfDay::new(end.0, end.1),
        days: days.to_vec(),
    }
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
