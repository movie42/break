use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};

use crate::rules::{Rules, TimeWindow, Weekday};

const MINUTES_PER_DAY: u16 = 24 * 60;

fn previous_day(day: Weekday) -> Weekday {
    match day {
        Weekday::Monday => Weekday::Sunday,
        Weekday::Tuesday => Weekday::Monday,
        Weekday::Wednesday => Weekday::Tuesday,
        Weekday::Thursday => Weekday::Wednesday,
        Weekday::Friday => Weekday::Thursday,
        Weekday::Saturday => Weekday::Friday,
        Weekday::Sunday => Weekday::Saturday,
    }
}

pub fn window_contains<Tz: TimeZone>(window: &TimeWindow, at: &DateTime<Tz>) -> bool {
    if !window.start.is_valid() || !window.end.is_valid() {
        return false;
    }

    let today = Weekday::from_chrono(at.weekday());
    let now = u16::try_from(at.hour()).unwrap_or(0) * 60 + u16::try_from(at.minute()).unwrap_or(0);
    let start = window.start.minutes_from_midnight();
    let end = window.end.minutes_from_midnight();

    if start == end {
        return window.days.contains(&today);
    }

    if start < end {
        return window.days.contains(&today) && now >= start && now < end;
    }

    let started_today = window.days.contains(&today) && now >= start && now < MINUTES_PER_DAY;
    let started_yesterday = window.days.contains(&previous_day(today)) && now < end;
    started_today || started_yesterday
}

pub fn is_blocking_at<Tz: TimeZone>(rules: &Rules, at: &DateTime<Tz>) -> bool {
    rules
        .schedule
        .windows
        .iter()
        .any(|window| window_contains(window, at))
}

pub fn is_blocking_now(rules: &Rules) -> bool {
    is_blocking_at(rules, &Local::now())
}
