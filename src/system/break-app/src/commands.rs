use break_core::rules::{Rules, SiteGroup, SiteTarget, RULES_VERSION};
use break_core::{schedule, store};

use crate::browsers::{self, QuitReport};
use crate::dns_guard::{self, DnsGuardStatus};
use crate::daemon::status::{self, DaemonStatus};
use crate::daemon::{install, DaemonError};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub rules: Rules,
    pub blocking_now: bool,
    pub daemon: DaemonStatus,
    pub daemon_needs_reinstall: bool,
    pub rules_path: String,
    pub rejected_sites: Vec<String>,
    pub dns_guard: DnsGuardStatus,
}

fn normalize(input: Rules) -> (Rules, Vec<String>) {
    let mut rejected = Vec::new();
    let mut groups = Vec::new();

    for group in input.groups {
        let mut sites: Vec<SiteTarget> = Vec::new();
        for site in group.sites {
            match SiteTarget::from_input(&site.host) {
                Some(target) => {
                    if !sites.iter().any(|kept| kept.host == target.host) {
                        sites.push(target);
                    }
                }
                None => rejected.push(site.host),
            }
        }
        groups.push(SiteGroup {
            id: group.id,
            name: group.name.trim().to_string(),
            sites,
            apps: group.apps,
        });
    }

    let mut normalized = Rules {
        version: RULES_VERSION,
        groups,
        schedule: input.schedule,
        legacy_sites: Vec::new(),
        legacy_apps: Vec::new(),
    };
    normalized.prune_group_links();

    (normalized, rejected)
}

fn to_state(rules: Rules, rejected_sites: Vec<String>) -> Result<AppState, String> {
    let rules_path = store::rules_path()
        .map_err(|err| err.to_string())?
        .to_string_lossy()
        .into_owned();

    let daemon = status::current();

    Ok(AppState {
        dns_guard: dns_guard::current(),
        blocking_now: schedule::is_blocking_now(&rules),
        daemon_needs_reinstall: daemon != DaemonStatus::NotInstalled
            && !status::installed_is_current(),
        daemon,
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
        Err(DaemonError::Cancelled) => Err("설치를 취소했습니다.".to_string()),
        Err(other) => Err(format!("설치에 실패했습니다: {other}")),
    }
}

#[tauri::command]
pub fn uninstall_daemon() -> Result<AppState, String> {
    let rules_path = store::rules_path().map_err(|err| err.to_string())?;
    match install::uninstall(&rules_path) {
        Ok(()) => reload(),
        Err(DaemonError::Cancelled) => Err("제거를 취소했습니다.".to_string()),
        Err(other) => Err(format!("제거에 실패했습니다: {other}")),
    }
}

#[tauri::command]
pub fn running_browsers() -> Vec<String> {
    browsers::running()
}

#[tauri::command]
pub fn quit_browsers() -> QuitReport {
    browsers::quit_running()
}
