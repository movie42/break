use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

const QUIT_DEADLINE: Duration = Duration::from_secs(4);
const POLL_INTERVAL: Duration = Duration::from_millis(200);
const BUNDLE_MARKER: &str = ".app/Contents/MacOS/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Browser {
    pub name: String,
    pub process: String,
}

#[derive(Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuitReport {
    pub closed: Vec<String>,
    pub still_open: Vec<String>,
}

pub fn running() -> Vec<String> {
    running_browsers()
        .into_iter()
        .map(|browser| browser.name)
        .collect()
}

pub fn quit_running() -> QuitReport {
    let targets = running_browsers();
    for browser in &targets {
        ask_to_quit(&browser.name);
    }

    let deadline = Instant::now() + QUIT_DEADLINE;
    loop {
        let still_open: Vec<String> = targets
            .iter()
            .filter(|browser| is_running(&browser.process))
            .map(|browser| browser.name.clone())
            .collect();

        if still_open.is_empty() || Instant::now() >= deadline {
            return QuitReport {
                closed: targets
                    .iter()
                    .map(|browser| browser.name.clone())
                    .filter(|name| !still_open.contains(name))
                    .collect(),
                still_open,
            };
        }

        thread::sleep(POLL_INTERVAL);
    }
}

fn running_browsers() -> Vec<Browser> {
    let mut found: BTreeMap<String, Browser> = BTreeMap::new();
    let own = std::env::current_exe()
        .ok()
        .and_then(|path| bundle_of(&path).map(Path::to_path_buf));

    for path in running_executables() {
        let Some(bundle) = bundle_of(&path) else {
            continue;
        };
        let Some(name) = bundle_name(bundle) else {
            continue;
        };
        let Some(process) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if own.as_deref() == Some(bundle) || found.contains_key(&name) || !handles_web(bundle) {
            continue;
        }
        found.insert(
            name.clone(),
            Browser {
                name,
                process: process.to_string(),
            },
        );
    }

    found.into_values().collect()
}

fn running_executables() -> Vec<PathBuf> {
    let Ok(output) = Command::new("/bin/ps").args(["-Ao", "comm="]).output() else {
        return Vec::new();
    };

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect()
}

fn bundle_of(path: &Path) -> Option<&Path> {
    let text = path.to_str()?;
    let index = text.rfind(BUNDLE_MARKER)?;
    Some(Path::new(&text[..index + 4]))
}

fn bundle_name(bundle: &Path) -> Option<String> {
    bundle
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_string)
}

fn handles_web(bundle: &Path) -> bool {
    let plist = bundle.join("Contents").join("Info.plist");
    let Ok(output) = Command::new("/usr/bin/plutil")
        .args(["-extract", "CFBundleURLTypes", "json", "-o", "-"])
        .arg(&plist)
        .output()
    else {
        return false;
    };
    if !output.status.success() {
        return false;
    }

    match serde_json::from_slice::<serde_json::Value>(&output.stdout) {
        Ok(value) => declares_web_scheme(&value),
        Err(_) => false,
    }
}

fn declares_web_scheme(value: &serde_json::Value) -> bool {
    let Some(types) = value.as_array() else {
        return false;
    };

    types.iter().any(|entry| {
        entry
            .get("CFBundleURLSchemes")
            .and_then(|schemes| schemes.as_array())
            .is_some_and(|schemes| {
                schemes.iter().any(|scheme| {
                    scheme.as_str().is_some_and(|scheme| {
                        scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
                    })
                })
            })
    })
}

fn is_running(process: &str) -> bool {
    Command::new("pgrep")
        .args(["-x", process])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn ask_to_quit(app: &str) {
    let _ = Command::new("osascript")
        .args(["-e", &quit_script(app)])
        .output();
}

fn quit_script(app: &str) -> String {
    format!(
        "with timeout of 3 seconds\ntell application \"{}\" to quit\nend timeout",
        app.replace('\\', "\\\\").replace('"', "\\\"")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_quit_script_targets_the_named_application() {
        let script = quit_script("Google Chrome");
        assert!(script.contains("tell application \"Google Chrome\" to quit"));
        assert!(script.starts_with("with timeout of 3 seconds"));
    }

    #[test]
    fn a_quote_in_a_name_cannot_close_the_applescript_literal() {
        assert!(quit_script("Odd\"Browser").contains("\"Odd\\\"Browser\""));
    }

    #[test]
    fn a_main_executable_resolves_to_its_own_bundle() {
        let path = PathBuf::from("/Applications/Aside.app/Contents/MacOS/Aside");
        assert_eq!(
            bundle_of(&path),
            Some(Path::new("/Applications/Aside.app"))
        );
        assert_eq!(
            bundle_name(bundle_of(&path).expect("bundle")),
            Some("Aside".to_string())
        );
    }

    #[test]
    fn a_helper_resolves_to_the_helper_bundle_not_the_browser() {
        let path = PathBuf::from(
            "/Applications/Aside.app/Contents/Frameworks/Aside Framework.framework/Versions/1.0/Helpers/Aside Helper.app/Contents/MacOS/Aside Helper",
        );
        assert_eq!(
            bundle_name(bundle_of(&path).expect("bundle")),
            Some("Aside Helper".to_string())
        );
    }

    #[test]
    fn a_path_outside_a_bundle_is_not_an_app() {
        assert_eq!(bundle_of(Path::new("/usr/sbin/cupsd")), None);
        assert_eq!(bundle_of(Path::new("kernel_task")), None);
    }

    #[test]
    fn an_app_that_opens_web_addresses_is_a_browser() {
        let value = serde_json::json!([
            { "CFBundleURLName": "Web", "CFBundleURLSchemes": ["http", "https"] }
        ]);
        assert!(declares_web_scheme(&value));
    }

    #[test]
    fn an_app_with_only_its_own_scheme_is_not_a_browser() {
        let value = serde_json::json!([
            { "CFBundleURLName": "Obsidian", "CFBundleURLSchemes": ["obsidian"] }
        ]);
        assert!(!declares_web_scheme(&value));
    }

    #[test]
    fn a_plist_without_url_types_is_not_a_browser() {
        assert!(!declares_web_scheme(&serde_json::json!(null)));
        assert!(!declares_web_scheme(&serde_json::json!([])));
        assert!(!declares_web_scheme(&serde_json::json!([{}])));
    }

    #[test]
    fn a_process_that_cannot_exist_is_not_running() {
        assert!(!is_running("break-no-such-browser"));
    }
}
