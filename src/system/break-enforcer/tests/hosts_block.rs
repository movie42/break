use std::fs;
use std::path::PathBuf;

use break_core::rules::SiteTarget;
use break_enforcer::hosts::{
    apply_to_content, read_and_write, render_block, strip_block, BEGIN_MARKER, END_MARKER,
};

fn sites(hosts: &[&str]) -> Vec<SiteTarget> {
    hosts
        .iter()
        .map(|host| SiteTarget {
            host: (*host).to_string(),
        })
        .collect()
}

fn temp_file(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("break-hosts-test-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    dir.join("hosts")
}

#[test]
fn block_covers_both_address_families_and_the_www_form() {
    let block = render_block(&sites(&["youtube.com"]));

    assert!(block.starts_with(BEGIN_MARKER));
    assert!(block.trim_end().ends_with(END_MARKER));
    assert!(block.contains("127.0.0.1\tyoutube.com\n"));
    assert!(block.contains("127.0.0.1\twww.youtube.com\n"));
    assert!(block.contains("::1\tyoutube.com\n"));
    assert!(block.contains("::1\twww.youtube.com\n"));
}

#[test]
fn an_empty_site_list_renders_no_markers() {
    assert_eq!(render_block(&[]), "");
}

#[test]
fn adding_to_an_empty_file_leaves_only_the_block() {
    let result = apply_to_content("", &sites(&["youtube.com"]));
    assert_eq!(result, render_block(&sites(&["youtube.com"])));
}

#[test]
fn applying_twice_does_not_duplicate_the_block() {
    let targets = sites(&["youtube.com"]);
    let once = apply_to_content("127.0.0.1\tlocalhost\n", &targets);
    let twice = apply_to_content(&once, &targets);

    assert_eq!(once, twice);
    assert_eq!(twice.matches(BEGIN_MARKER).count(), 1);
}

#[test]
fn clearing_restores_the_lines_the_user_wrote() {
    let original = "127.0.0.1\tlocalhost\n255.255.255.255\tbroadcasthost\n";
    let blocked = apply_to_content(original, &sites(&["youtube.com"]));
    let cleared = apply_to_content(&blocked, &[]);

    assert_eq!(cleared, original);
    assert!(!cleared.contains(BEGIN_MARKER));
    assert!(!cleared.contains(END_MARKER));
}

#[test]
fn lines_outside_the_markers_survive_in_order() {
    let original = "# comment\n127.0.0.1\tlocalhost\n\n10.0.0.5\tinternal.example\n";
    let blocked = apply_to_content(original, &sites(&["youtube.com", "reddit.com"]));

    let kept: Vec<&str> = blocked
        .lines()
        .take_while(|line| line.trim() != BEGIN_MARKER)
        .collect();
    assert_eq!(
        kept,
        vec![
            "# comment",
            "127.0.0.1\tlocalhost",
            "",
            "10.0.0.5\tinternal.example",
            "",
        ]
    );
}

#[test]
fn a_block_missing_its_end_marker_is_cut_to_the_end_of_the_file() {
    let broken = format!("127.0.0.1\tlocalhost\n{BEGIN_MARKER}\n127.0.0.1\tyoutube.com\n");
    assert_eq!(strip_block(&broken), "127.0.0.1\tlocalhost\n");
}

#[test]
fn writing_reports_whether_the_file_changed() {
    let path = temp_file("write");
    fs::write(&path, "127.0.0.1\tlocalhost\n").expect("seed");

    assert!(read_and_write(&path, &sites(&["youtube.com"])).expect("first write"));
    assert!(!read_and_write(&path, &sites(&["youtube.com"])).expect("second write"));

    let content = fs::read_to_string(&path).expect("read back");
    assert!(content.starts_with("127.0.0.1\tlocalhost\n"));
    assert!(content.contains("127.0.0.1\tyoutube.com\n"));

    assert!(read_and_write(&path, &[]).expect("clear"));
    assert_eq!(
        fs::read_to_string(&path).expect("read back"),
        "127.0.0.1\tlocalhost\n"
    );
}

#[test]
fn writing_keeps_the_original_file_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let path = temp_file("permissions");
    fs::write(&path, "127.0.0.1\tlocalhost\n").expect("seed");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("chmod");

    read_and_write(&path, &sites(&["youtube.com"])).expect("write");

    let mode = fs::metadata(&path).expect("metadata").permissions().mode();
    assert_eq!(mode & 0o777, 0o644);
}
