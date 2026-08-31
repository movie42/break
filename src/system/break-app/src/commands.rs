use break_core::rules::{Rules, RULES_VERSION};
use break_core::{schedule, store};
use break_enforcer::{platform_enforcer, Enforcer, Error as EnforcerError};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum EnforcementStatus {
    Applied,
    NotPrivileged { message: String },
    Failed { message: String },
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub rules: Rules,
    pub blocking_now: bool,
    pub enforcement: EnforcementStatus,
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

fn enforce(rules: &Rules, blocking_now: bool) -> EnforcementStatus {
    let enforcer = platform_enforcer();
    let result = if blocking_now {
        enforcer.apply_sites(&rules.sites)
    } else {
        enforcer.clear_sites()
    };

    match result {
        Ok(()) => EnforcementStatus::Applied,
        Err(EnforcerError::NotPrivileged) => EnforcementStatus::NotPrivileged {
            message: EnforcerError::NotPrivileged.to_string(),
        },
        Err(err) => EnforcementStatus::Failed {
            message: err.to_string(),
        },
    }
}

fn to_state(rules: Rules, rejected_sites: Vec<String>) -> Result<AppState, String> {
    let rules_path = store::rules_path()
        .map_err(|err| err.to_string())?
        .to_string_lossy()
        .into_owned();
    let blocking_now = schedule::is_blocking_now(&rules);

    Ok(AppState {
        enforcement: enforce(&rules, blocking_now),
        blocking_now,
        rules,
        rules_path,
        rejected_sites,
    })
}

#[tauri::command]
pub fn load_rules() -> Result<AppState, String> {
    let rules = store::load().map_err(|err| err.to_string())?;
    to_state(rules, Vec::new())
}

#[tauri::command]
pub fn save_rules(rules: Rules) -> Result<AppState, String> {
    let (normalized, rejected) = normalize(rules);
    store::save(&normalized).map_err(|err| err.to_string())?;
    to_state(normalized, rejected)
}
