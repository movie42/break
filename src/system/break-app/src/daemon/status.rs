use std::path::Path;
use std::process::Command;

use super::paths::{LABEL, PLIST_PATH};

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
