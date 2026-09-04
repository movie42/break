#![cfg(target_os = "macos")]

use std::fs;
use std::path::PathBuf;

use break_enforcer::marker::{BEGIN_MARKER, END_MARKER};
use break_enforcer::pf::conf::{apply_to_content, is_linked, read_and_write, render_lines};

const STOCK_PF_CONF: &str = "#
# Default PF configuration file.
#
scrub-anchor \"com.apple/*\"
nat-anchor \"com.apple/*\"
rdr-anchor \"com.apple/*\"
dummynet-anchor \"com.apple/*\"
anchor \"com.apple/*\"
load anchor \"com.apple\" from \"/etc/pf.anchors/com.apple\"
";

fn temp_file(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("break-pf-conf-test-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    dir.join("pf.conf")
}

#[test]
fn linking_adds_the_anchor_point_and_its_load_line() {
    let linked = apply_to_content(STOCK_PF_CONF, true);

    assert!(linked.contains(BEGIN_MARKER));
    assert!(linked.trim_end().ends_with(END_MARKER));
    assert!(linked.contains("anchor \"com.movie42.break\"\n"));
    assert!(linked
        .contains("load anchor \"com.movie42.break\" from \"/etc/pf.anchors/com.movie42.break\"\n"));
}

#[test]
fn the_break_anchor_comes_after_the_apple_anchor() {
    let linked = apply_to_content(STOCK_PF_CONF, true);

    let apple = linked.find("load anchor \"com.apple\"").expect("apple anchor");
    let ours = linked.find(BEGIN_MARKER).expect("break marker");
    assert!(apple < ours);
}

#[test]
fn the_stock_lines_survive_untouched() {
    let linked = apply_to_content(STOCK_PF_CONF, true);
    let head: String = linked
        .lines()
        .take_while(|line| line.trim() != BEGIN_MARKER)
        .collect::<Vec<_>>()
        .join("\n");

    assert_eq!(head.trim_end(), STOCK_PF_CONF.trim_end());
}

#[test]
fn unlinking_restores_the_file_byte_for_byte() {
    let linked = apply_to_content(STOCK_PF_CONF, true);
    assert_eq!(apply_to_content(&linked, false), STOCK_PF_CONF);
}

#[test]
fn linking_twice_does_not_duplicate_the_block() {
    let once = apply_to_content(STOCK_PF_CONF, true);
    let twice = apply_to_content(&once, true);

    assert_eq!(once, twice);
    assert_eq!(twice.matches(BEGIN_MARKER).count(), 1);
}

#[test]
fn a_stock_file_reports_as_not_linked() {
    assert!(!is_linked(STOCK_PF_CONF));
    assert!(is_linked(&apply_to_content(STOCK_PF_CONF, true)));
}

#[test]
fn a_block_emptied_by_hand_reports_as_not_linked() {
    let tampered = format!("{STOCK_PF_CONF}\n{BEGIN_MARKER}\n{END_MARKER}\n");
    assert!(!is_linked(&tampered));
}

#[test]
fn writing_reports_whether_the_file_changed() {
    let path = temp_file("write");
    fs::write(&path, STOCK_PF_CONF).expect("seed");

    assert!(read_and_write(&path, true).expect("link"));
    assert!(!read_and_write(&path, true).expect("link again"));
    assert!(is_linked(&fs::read_to_string(&path).expect("read back")));

    assert!(read_and_write(&path, false).expect("unlink"));
    assert_eq!(
        fs::read_to_string(&path).expect("read back"),
        STOCK_PF_CONF
    );
    assert!(!read_and_write(&path, false).expect("unlink again"));
}

#[test]
fn the_load_line_points_at_the_anchor_file_the_rules_module_owns() {
    assert!(render_lines().contains(break_enforcer::pf::rules::ANCHOR_PATH));
    assert!(render_lines().contains(break_enforcer::pf::rules::ANCHOR_NAME));
}
