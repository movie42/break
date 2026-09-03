use break_core::rules::normalize_host;
use break_core::rules::{
    AppTarget, Rules, Schedule, SiteGroup, SiteTarget, TimeOfDay, TimeWindow, Weekday,
    MIGRATED_GROUP_ID, RULES_VERSION,
};

fn sample() -> Rules {
    Rules {
        version: RULES_VERSION,
        groups: vec![SiteGroup {
            id: "g1".to_string(),
            name: "소셜".to_string(),
            sites: vec![SiteTarget {
                host: "youtube.com".to_string(),
            }],
            apps: vec![AppTarget {
                bundle_id: "com.apple.Safari".to_string(),
                display_name: "Safari".to_string(),
            }],
        }],
        schedule: Schedule {
            windows: vec![TimeWindow {
                id: "w1".to_string(),
                start: TimeOfDay::new(22, 0),
                end: TimeOfDay::new(2, 0),
                days: vec![Weekday::Monday, Weekday::Tuesday],
                group_ids: vec!["g1".to_string()],
                enabled: true,
            }],
        },
        legacy_sites: Vec::new(),
        legacy_apps: Vec::new(),
    }
}

#[test]
fn serializes_and_deserializes_without_loss() {
    let original = sample();
    let json = serde_json::to_string(&original).expect("직렬화 실패");
    let decoded: Rules = serde_json::from_str(&json).expect("역직렬화 실패");
    assert_eq!(original, decoded);
}

#[test]
fn uses_camel_case_field_names() {
    let json = serde_json::to_string(&sample()).expect("직렬화 실패");
    assert!(json.contains("\"bundleId\""));
    assert!(json.contains("\"displayName\""));
    assert!(json.contains("\"groupIds\""));
    assert!(!json.contains("\"bundle_id\""));
    assert!(!json.contains("\"group_ids\""));
}

#[test]
fn an_empty_legacy_list_is_not_written_back() {
    let json = serde_json::to_string(&sample()).expect("직렬화 실패");
    assert!(!json.contains("\"sites\":[],"));
    assert!(!json.contains("\"apps\":[]}"));
}

#[test]
fn missing_optional_fields_fall_back_to_empty() {
    let decoded: Rules = serde_json::from_str(r#"{"version":2}"#).expect("역직렬화 실패");
    assert!(decoded.groups.is_empty());
    assert!(decoded.schedule.windows.is_empty());
}

#[test]
fn a_version_1_file_becomes_one_group_every_window_points_at() {
    let raw = r#"{
        "version": 1,
        "sites": [{ "host": "youtube.com" }],
        "apps": [{ "bundleId": "com.hnc.Discord", "displayName": "Discord" }],
        "schedule": {
            "windows": [{
                "id": "w1",
                "start": { "hour": 22, "minute": 0 },
                "end": { "hour": 2, "minute": 0 },
                "days": ["monday"]
            }]
        }
    }"#;

    let mut rules: Rules = serde_json::from_str(raw).expect("역직렬화 실패");
    rules.migrate();

    assert_eq!(rules.version, RULES_VERSION);
    assert!(rules.legacy_sites.is_empty());
    assert!(rules.legacy_apps.is_empty());

    let group = rules.group(MIGRATED_GROUP_ID).expect("옮겨진 그룹");
    assert_eq!(group.sites[0].host, "youtube.com");
    assert_eq!(group.apps[0].bundle_id, "com.hnc.Discord");
    assert_eq!(
        rules.schedule.windows[0].group_ids,
        vec![MIGRATED_GROUP_ID.to_string()]
    );
}

#[test]
fn migrating_a_version_2_file_changes_nothing() {
    let mut rules = sample();
    rules.migrate();
    assert_eq!(rules, sample());
}

#[test]
fn a_link_to_a_deleted_group_is_pruned() {
    let mut rules = sample();
    rules.schedule.windows[0]
        .group_ids
        .push("gone".to_string());
    rules.prune_group_links();
    assert_eq!(
        rules.schedule.windows[0].group_ids,
        vec!["g1".to_string()]
    );
}

#[test]
fn normalizes_scheme_path_and_www() {
    assert_eq!(normalize_host("https://x.com/path"), Some("x.com".into()));
    assert_eq!(normalize_host("www.x.com"), Some("x.com".into()));
    assert_eq!(normalize_host("x.com"), Some("x.com".into()));
    assert_eq!(
        normalize_host("HTTP://WWW.X.COM:8080/a?b=1"),
        Some("x.com".into())
    );
    assert_eq!(normalize_host("  x.com.  "), Some("x.com".into()));
    assert_eq!(normalize_host("localhost"), None);
    assert_eq!(normalize_host(""), None);
}

#[test]
fn rejects_duplicate_sites_in_a_group() {
    let mut group = SiteGroup::new("g1", "소셜");
    assert!(group.add_site("https://x.com/a").is_some());
    assert!(group.add_site("www.x.com").is_none());
    assert_eq!(group.sites.len(), 1);
}

#[test]
fn a_version_2_file_reads_every_window_as_on() {
    let rules: Rules = serde_json::from_str(
        r#"{
            "version": 2,
            "groups": [{ "id": "g1", "name": "소셜", "sites": [], "apps": [] }],
            "schedule": {
                "windows": [{
                    "id": "w1",
                    "start": { "hour": 22, "minute": 0 },
                    "end": { "hour": 2, "minute": 0 },
                    "days": ["monday"],
                    "groupIds": ["g1"]
                }]
            }
        }"#,
    )
    .expect("역직렬화 실패");

    assert!(rules.schedule.windows[0].enabled);
}
