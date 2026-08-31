use break_core::rules::{AppTarget, Rules, Schedule, SiteTarget, TimeOfDay, TimeWindow, Weekday};
use break_core::rules::normalize_host;

fn sample() -> Rules {
    Rules {
        version: break_core::RULES_VERSION,
        sites: vec![SiteTarget {
            host: "youtube.com".to_string(),
        }],
        apps: vec![AppTarget {
            bundle_id: "com.apple.Safari".to_string(),
            display_name: "Safari".to_string(),
        }],
        schedule: Schedule {
            windows: vec![TimeWindow {
                id: "w1".to_string(),
                start: TimeOfDay::new(22, 0),
                end: TimeOfDay::new(2, 0),
                days: vec![Weekday::Monday, Weekday::Tuesday],
            }],
        },
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
    assert!(!json.contains("\"bundle_id\""));
}

#[test]
fn missing_optional_fields_fall_back_to_empty() {
    let decoded: Rules = serde_json::from_str(r#"{"version":1}"#).expect("역직렬화 실패");
    assert!(decoded.sites.is_empty());
    assert!(decoded.apps.is_empty());
    assert!(decoded.schedule.windows.is_empty());
}

#[test]
fn normalizes_scheme_path_and_www() {
    assert_eq!(normalize_host("https://x.com/path"), Some("x.com".into()));
    assert_eq!(normalize_host("www.x.com"), Some("x.com".into()));
    assert_eq!(normalize_host("x.com"), Some("x.com".into()));
    assert_eq!(normalize_host("HTTP://WWW.X.COM:8080/a?b=1"), Some("x.com".into()));
    assert_eq!(normalize_host("  x.com.  "), Some("x.com".into()));
    assert_eq!(normalize_host("localhost"), None);
    assert_eq!(normalize_host(""), None);
}

#[test]
fn rejects_duplicate_sites() {
    let mut rules = Rules::default();
    assert!(rules.add_site("https://x.com/a").is_some());
    assert!(rules.add_site("www.x.com").is_none());
    assert_eq!(rules.sites.len(), 1);
}
