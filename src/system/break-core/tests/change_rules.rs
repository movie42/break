use break_core::change::{check_save, ChangeError};
use break_core::rules::{
    InstantBlock, Rules, Schedule, SiteGroup, SiteTarget, TimeOfDay, TimeWindow, Weekday,
};
use chrono::{DateTime, Duration, Local, TimeZone, Utc};

fn group(id: &str, hosts: &[&str]) -> SiteGroup {
    let mut group = SiteGroup::new(id, id);
    group.sites = hosts
        .iter()
        .map(|host| SiteTarget {
            host: (*host).to_string(),
        })
        .collect();
    group
}

fn monday_window(id: &str, start: u8, end: u8, groups: &[&str]) -> TimeWindow {
    TimeWindow {
        id: id.to_string(),
        start: TimeOfDay::new(start, 0),
        end: TimeOfDay::new(end, 0),
        days: vec![Weekday::Monday],
        group_ids: groups.iter().map(|id| (*id).to_string()).collect(),
        enabled: true,
    }
}

fn monday(hour: u32) -> DateTime<Utc> {
    Local
        .with_ymd_and_hms(2026, 10, 19, hour, 0, 0)
        .single()
        .expect("local time")
        .with_timezone(&Utc)
}

fn rules() -> Rules {
    Rules {
        groups: vec![
            group("social", &["youtube.com"]),
            group("shop", &["coupang.com"]),
        ],
        schedule: Schedule {
            windows: vec![
                monday_window("running", 9, 12, &["social"]),
                monday_window("later", 20, 22, &["shop"]),
            ],
        },
        ..Rules::default()
    }
}

#[test]
fn a_running_window_cannot_be_turned_off() {
    let old = rules();
    let mut new = old.clone();
    new.schedule.windows[0].enabled = false;
    assert_eq!(
        check_save(&old, &new, monday(10)),
        Err(ChangeError::RunningWindow)
    );
}

#[test]
fn a_running_window_cannot_be_deleted_or_shortened() {
    let old = rules();
    let mut deleted = old.clone();
    deleted.schedule.windows.remove(0);
    assert_eq!(
        check_save(&old, &deleted, monday(10)),
        Err(ChangeError::RunningWindow)
    );

    let mut shortened = old.clone();
    shortened.schedule.windows[0].end = TimeOfDay::new(10, 30);
    assert_eq!(
        check_save(&old, &shortened, monday(10)),
        Err(ChangeError::RunningWindow)
    );
}

#[test]
fn a_window_that_is_not_running_can_be_deleted() {
    let old = rules();
    let mut new = old.clone();
    new.schedule.windows.remove(1);
    assert_eq!(check_save(&old, &new, monday(10)), Ok(()));
}

#[test]
fn a_running_window_can_be_deleted_once_it_ends() {
    let old = rules();
    let mut new = old.clone();
    new.schedule.windows.remove(0);
    assert_eq!(check_save(&old, &new, monday(12)), Ok(()));
}

#[test]
fn a_site_cannot_leave_a_group_a_running_window_uses() {
    let old = rules();
    let mut new = old.clone();
    new.groups[0].sites.clear();
    assert_eq!(
        check_save(&old, &new, monday(10)),
        Err(ChangeError::LockedGroup)
    );

    let mut added = old.clone();
    added.groups[0].sites.push(SiteTarget {
        host: "x.com".to_string(),
    });
    assert_eq!(check_save(&old, &added, monday(10)), Ok(()));
}

#[test]
fn an_instant_block_can_be_extended_but_not_shortened() {
    let mut old = rules();
    old.instant = Some(InstantBlock {
        ends_at: monday(15),
        group_ids: vec!["shop".to_string()],
    });

    let mut longer = old.clone();
    longer.instant.as_mut().expect("instant").ends_at = monday(16);
    assert_eq!(check_save(&old, &longer, monday(13)), Ok(()));

    let mut shorter = old.clone();
    shorter.instant.as_mut().expect("instant").ends_at = monday(14);
    assert_eq!(
        check_save(&old, &shorter, monday(13)),
        Err(ChangeError::InstantShortened)
    );

    let mut removed = old.clone();
    removed.instant = None;
    assert_eq!(
        check_save(&old, &removed, monday(13)),
        Err(ChangeError::InstantShortened)
    );

    let mut emptied = old.clone();
    emptied.groups[1].sites.clear();
    assert_eq!(
        check_save(&old, &emptied, monday(13)),
        Err(ChangeError::LockedGroup)
    );
}

#[test]
fn an_ended_instant_block_can_be_cleared() {
    let mut old = rules();
    old.instant = Some(InstantBlock {
        ends_at: monday(15),
        group_ids: vec![],
    });
    let mut new = old.clone();
    new.instant = None;
    assert_eq!(
        check_save(&old, &new, monday(15) + Duration::minutes(1)),
        Ok(())
    );
}
