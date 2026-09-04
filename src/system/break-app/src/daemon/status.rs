use std::fs;
use std::path::Path;
use std::process::Command;

use super::paths::{installed_binary, source_binary, LABEL, PLIST_PATH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum DaemonStatus {
    NotInstalled,
    Installed,
    Running,
}

pub fn current() -> DaemonStatus {
    if !Path::new(PLIST_PATH).exists() {
        return DaemonStatus::NotInstalled;
    }
    if is_running() {
        DaemonStatus::Running
    } else {
        DaemonStatus::Installed
    }
}

fn is_running() -> bool {
    Command::new("launchctl")
        .args(["print", &format!("system/{LABEL}")])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn installed_is_current() -> bool {
    match source_binary() {
        Some(source) => files_match(&installed_binary(), &source),
        None => true,
    }
}

fn files_match(left: &Path, right: &Path) -> bool {
    let (Ok(left_meta), Ok(right_meta)) = (fs::metadata(left), fs::metadata(right)) else {
        return true;
    };
    if left_meta.len() != right_meta.len() {
        return false;
    }

    match (fs::read(left), fs::read(right)) {
        (Ok(left_bytes), Ok(right_bytes)) => left_bytes == right_bytes,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(name: &str, body: &[u8]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("break-app-status-test");
        fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join(name);
        fs::write(&path, body).expect("write");
        path
    }

    #[test]
    fn the_same_bytes_match() {
        let left = write("same-a", b"break-daemon-v2");
        let right = write("same-b", b"break-daemon-v2");
        assert!(files_match(&left, &right));
    }

    #[test]
    fn a_different_length_does_not_match() {
        let left = write("len-a", b"break-daemon-v1");
        let right = write("len-b", b"break-daemon-v2-longer");
        assert!(!files_match(&left, &right));
    }

    #[test]
    fn the_same_length_with_different_bytes_does_not_match() {
        let left = write("bytes-a", b"break-daemon-v1");
        let right = write("bytes-b", b"break-daemon-v2");
        assert!(!files_match(&left, &right));
    }

    #[test]
    fn a_missing_file_is_treated_as_current() {
        let left = write("missing-a", b"break-daemon-v2");
        let right = std::env::temp_dir().join("break-app-status-test/nope");
        let _ = fs::remove_file(&right);
        assert!(files_match(&left, &right));
    }
}
