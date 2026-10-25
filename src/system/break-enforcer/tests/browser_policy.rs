#![cfg(target_os = "macos")]

use break_core::rules::SiteTarget;
use break_enforcer::policy::{
    chromium_bundle_ids, is_break_managed, policy_path, render, write_one, MANAGED_KEY,
};

fn sites(hosts: &[&str]) -> Vec<SiteTarget> {
    hosts
        .iter()
        .map(|host| SiteTarget {
            host: (*host).to_string(),
        })
        .collect()
}

#[test]
fn every_blocked_site_becomes_a_url_blocklist_entry() {
    let plist = render(&sites(&["youtube.com", "reddit.com"]));

    assert!(plist.contains("<key>URLBlocklist</key>"));
    assert!(plist.contains("<string>youtube.com</string>"));
    assert!(plist.contains("<string>reddit.com</string>"));
    assert_eq!(plist.matches("<string>").count(), 2);
}

#[test]
fn the_plist_is_well_formed_and_declared_as_ours() {
    let plist = render(&sites(&["youtube.com"]));

    assert!(plist.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(plist.contains("<plist version=\"1.0\">"));
    assert!(plist.trim_end().ends_with("</plist>"));
    assert!(plist.contains(&format!("<key>{MANAGED_KEY}</key>")));
    assert!(is_break_managed(&plist));
}

#[test]
fn a_policy_file_written_by_someone_else_is_not_ours() {
    let foreign = "<?xml version=\"1.0\"?><plist version=\"1.0\"><dict>\
                   <key>URLBlocklist</key><array><string>x.com</string></array>\
                   </dict></plist>";
    assert!(!is_break_managed(foreign));
}

#[test]
fn no_sites_renders_no_policy() {
    assert_eq!(render(&[]), "");
}

#[test]
fn a_host_with_xml_significant_characters_is_escaped() {
    let plist = render(&sites(&["a&b<c>.com"]));

    assert!(plist.contains("<string>a&amp;b&lt;c&gt;.com</string>"));
    assert!(!plist.contains("a&b<c>.com"));
}

#[test]
fn the_policy_lands_in_the_managed_preferences_directory() {
    let path = policy_path("com.google.Chrome");

    assert_eq!(
        path.to_string_lossy(),
        "/Library/Managed Preferences/com.google.Chrome.plist"
    );
}

#[test]
fn chromium_browsers_are_found_and_safari_is_not() {
    let ids = chromium_bundle_ids();

    assert!(!ids.contains(&"com.apple.Safari".to_string()));
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted, "정렬된 순서로 돌려준다");
    assert_eq!(
        ids.len(),
        ids.iter().collect::<std::collections::HashSet<_>>().len(),
        "중복이 없다"
    );
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("break-policy-test-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

#[test]
fn writing_the_same_policy_twice_reports_a_change_only_once() {
    let path = temp_dir("same-twice").join("com.google.Chrome.plist");
    let body = render(&sites(&["youtube.com"]));

    assert!(write_one(&path, &body).expect("first write"));
    assert!(!write_one(&path, &body).expect("second write"));
}

#[test]
fn removing_a_policy_reports_a_change_only_once() {
    let path = temp_dir("remove-twice").join("com.google.Chrome.plist");
    write_one(&path, &render(&sites(&["youtube.com"]))).expect("write");

    assert!(write_one(&path, "").expect("first clear"));
    assert!(!write_one(&path, "").expect("second clear"));
    assert!(!path.exists());
}

#[test]
fn clearing_a_policy_that_was_never_written_is_not_a_change() {
    let path = temp_dir("never-written").join("com.google.Chrome.plist");

    assert!(!write_one(&path, "").expect("clear"));
}

#[test]
fn a_policy_file_written_by_someone_else_is_left_alone() {
    let path = temp_dir("foreign").join("com.google.Chrome.plist");
    let foreign = "<?xml version=\"1.0\"?><plist version=\"1.0\"><dict>\
                   <key>URLBlocklist</key><array><string>x.com</string></array>\
                   </dict></plist>";
    std::fs::write(&path, foreign).expect("seed foreign policy");

    assert!(!write_one(&path, &render(&sites(&["youtube.com"]))).expect("apply"));
    assert!(!write_one(&path, "").expect("clear"));
    assert_eq!(std::fs::read_to_string(&path).expect("read back"), foreign);
}

#[test]
fn changing_the_blocked_sites_is_a_change() {
    let path = temp_dir("changed-sites").join("com.google.Chrome.plist");

    assert!(write_one(&path, &render(&sites(&["youtube.com"]))).expect("first"));
    assert!(write_one(&path, &render(&sites(&["youtube.com", "reddit.com"]))).expect("second"));
}
