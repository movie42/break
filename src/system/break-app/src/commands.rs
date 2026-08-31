use break_core::rules::{Rules, RULES_VERSION};
use break_core::{schedule, store};

use crate::daemon::status::{self, DaemonStatus};
use crate::daemon::{install, DaemonError};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub rules: Rules,
    pub blocking_now: bool,
    pub daemon: DaemonStatus,
    pub rules_path: String,
    pub rejected_sites: Vec<String>,
}

fn normalize(input: Rules) -> (Rules, Vec<String>) {
    let mut normalized = Rules {
        version: RULES_VERSION,
        sites: Vec::new(),
        apps: input.apps,
        schedule: input.schedule,
    };
    let mut rejected = Vec::new();

    for site in input.sites {
        if normalized.add_site(&site.host).is_none()
            && break_core::rules::normalize_host(&site.host).is_none()
        {
            rejected.push(site.host);
        }
    }

    (normalized, rejected)
}

fn to_state(rules: Rules, rejected_sites: Vec<String>) -> Result<AppState, String> {
    let rules_path = store::rules_path()
        .map_err(|err| err.to_string())?
        .to_string_lossy()
        .into_owned();

    Ok(AppState {
        blocking_now: schedule::is_blocking_now(&rules),
        daemon: status::current(),
        rules,
        rules_path,
        rejected_sites,
    })
}

fn reload() -> Result<AppState, String> {
    let rules = store::load().map_err(|err| err.to_string())?;
    to_state(rules, Vec::new())
}

#[tauri::command]
pub fn load_rules() -> Result<AppState, String> {
    reload()
}

#[tauri::command]
pub fn save_rules(rules: Rules) -> Result<AppState, String> {
    let (normalized, rejected) = normalize(rules);
    store::save(&normalized).map_err(|err| err.to_string())?;
    to_state(normalized, rejected)
}

#[tauri::command]
pub fn daemon_status() -> DaemonStatus {
    status::current()
}

#[tauri::command]
pub fn install_daemon() -> Result<AppState, String> {
    let rules_path = store::rules_path().map_err(|err| err.to_string())?;
    match install::install(&rules_path) {
        Ok(()) => reload(),
        Err(DaemonError::Cancelled) => Err(DaemonError::Cancelled.to_string()),
        Err(other) => Err(format!("설치에 실패했습니다: {other}")),
    }
}

#[tauri::command]
pub fn uninstall_daemon() -> Result<AppState, String> {
    let rules_path = store::rules_path().map_err(|err| err.to_string())?;
    match install::uninstall(&rules_path) {
        Ok(()) | Err(DaemonError::Cancelled) => reload(),
        Err(other) => Err(format!("제거에 실패했습니다: {other}")),
    }
}
