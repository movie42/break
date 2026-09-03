use std::fs;
use std::path::PathBuf;

use break_core::rules::{Rules, SiteGroup};
use break_core::store::{load_from, save_to, RULES_FILE_NAME};

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "break-core-test-{tag}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("임시 디렉토리 생성 실패");
    dir
}

#[test]
fn missing_file_loads_empty_rules() {
    let dir = temp_dir("missing");
    let rules = load_from(&dir.join(RULES_FILE_NAME)).expect("불러오기 실패");
    assert_eq!(rules, Rules::default());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn saved_rules_come_back_unchanged() {
    let dir = temp_dir("roundtrip");
    let path = dir.join(RULES_FILE_NAME);

    let mut rules = Rules::default();
    let mut group = SiteGroup::new("g1", "소셜");
    group.add_site("https://youtube.com/feed");
    rules.groups.push(group);
    save_to(&path, &rules).expect("저장 실패");

    let loaded = load_from(&path).expect("불러오기 실패");
    assert_eq!(loaded, rules);
    assert_eq!(loaded.groups[0].sites[0].host, "youtube.com");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn broken_json_is_quarantined_not_overwritten() {
    let dir = temp_dir("corrupt");
    let path = dir.join(RULES_FILE_NAME);
    fs::write(&path, "{ this is not json").expect("쓰기 실패");

    let rules = load_from(&path).expect("불러오기 실패");
    assert_eq!(rules, Rules::default());

    assert!(!path.exists());
    let quarantined: Vec<_> = fs::read_dir(&dir)
        .expect("디렉토리 읽기 실패")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with("rules.json.corrupt-"))
        .collect();
    assert_eq!(quarantined.len(), 1, "격리 파일이 1개여야 한다");

    let preserved = fs::read_to_string(dir.join(&quarantined[0])).expect("격리 파일 읽기 실패");
    assert_eq!(preserved, "{ this is not json");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_newer_rules_file_is_refused_instead_of_read_as_empty() {
    let dir = temp_dir("future");
    let path = dir.join(RULES_FILE_NAME);
    fs::write(&path, r#"{"version":99,"groups":[],"schedule":{"windows":[]}}"#)
        .expect("쓰기 실패");

    let error = load_from(&path).expect_err("거절해야 한다");
    assert!(matches!(
        error,
        break_core::store::StoreError::UnsupportedVersion { found: 99, .. }
    ));
    assert!(path.exists(), "격리하지 않고 그대로 둔다");
    let _ = fs::remove_dir_all(&dir);
}
