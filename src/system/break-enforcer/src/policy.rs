use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use break_core::rules::SiteTarget;

use crate::{atomic, is_dry_run, Error};

pub const MANAGED_PREFERENCES_DIR: &str = "/Library/Managed Preferences";
pub const APPLICATIONS_DIR: &str = "/Applications";
pub const MANAGED_KEY: &str = "BreakManaged";

static APPLIED: Mutex<Option<String>> = Mutex::new(None);

pub fn apply(sites: &[SiteTarget]) -> Result<bool, Error> {
    if is_dry_run() {
        log_dry_run(sites);
        return Ok(false);
    }
    write_all(&render(sites))
}

pub fn clear() -> Result<bool, Error> {
    if is_dry_run() {
        log_dry_run(&[]);
        return Ok(false);
    }
    write_all("")
}

pub fn render(sites: &[SiteTarget]) -> String {
    if sites.is_empty() {
        return String::new();
    }

    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n<dict>\n",
    );
    out.push_str(&format!("\t<key>{MANAGED_KEY}</key>\n\t<true/>\n"));
    out.push_str("\t<key>URLBlocklist</key>\n\t<array>\n");
    for site in sites {
        out.push_str("\t\t<string>");
        out.push_str(&escape(&site.host));
        out.push_str("</string>\n");
    }
    out.push_str("\t</array>\n</dict>\n</plist>\n");
    out
}

pub fn is_break_managed(content: &str) -> bool {
    content.contains(&format!("<key>{MANAGED_KEY}</key>"))
}

pub fn chromium_bundle_ids() -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    let Ok(entries) = fs::read_dir(APPLICATIONS_DIR) else {
        return ids;
    };

    for entry in entries.flatten() {
        let bundle = entry.path();
        if bundle.extension().and_then(|ext| ext.to_str()) != Some("app") {
            continue;
        }
        if !is_chromium(&bundle) || !handles_web(&bundle) {
            continue;
        }
        if let Some(id) = bundle_id(&bundle) {
            ids.push(id);
        }
    }

    ids.sort();
    ids.dedup();
    ids
}

fn write_all(body: &str) -> Result<bool, Error> {
    if APPLIED
        .lock()
        .unwrap_or_else(|err| err.into_inner())
        .as_deref()
        == Some(body)
    {
        return Ok(false);
    }

    let mut failure: Option<io::Error> = None;
    let mut changed = false;
    for id in chromium_bundle_ids() {
        match write_one(&policy_path(&id), body) {
            Ok(written) => changed |= written,
            Err(err) => {
                failure.get_or_insert(err);
            }
        }
    }

    match failure {
        Some(err) => Err(Error::Io(err)),
        None => {
            *APPLIED.lock().unwrap_or_else(|err| err.into_inner()) = Some(body.to_string());
            Ok(changed)
        }
    }
}

pub fn write_one(path: &Path, body: &str) -> io::Result<bool> {
    let current = fs::read_to_string(path);
    match &current {
        Ok(content) if !is_break_managed(content) => return Ok(false),
        Ok(content) if content == body => return Ok(false),
        Err(err) if err.kind() != io::ErrorKind::NotFound => return Err(err.kind().into()),
        _ => {}
    }

    if body.is_empty() {
        return match fs::remove_file(path) {
            Ok(()) => Ok(true),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(err) => Err(err),
        };
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    atomic::write(path, body)?;
    Ok(true)
}

pub fn policy_path(bundle_id: &str) -> PathBuf {
    Path::new(MANAGED_PREFERENCES_DIR).join(format!("{bundle_id}.plist"))
}

fn is_chromium(bundle: &Path) -> bool {
    let Ok(entries) = fs::read_dir(bundle.join("Contents").join("Frameworks")) else {
        return false;
    };

    entries.flatten().any(|entry| {
        let path = entry.path();
        path.extension().and_then(|ext| ext.to_str()) == Some("framework")
            && path.join("Versions").join("Current").join("Helpers").is_dir()
    })
}

fn handles_web(bundle: &Path) -> bool {
    let Ok(output) = Command::new("/usr/bin/plutil")
        .args(["-extract", "CFBundleURLTypes", "json", "-o", "-"])
        .arg(bundle.join("Contents").join("Info.plist"))
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

fn bundle_id(bundle: &Path) -> Option<String> {
    let output = Command::new("/usr/bin/plutil")
        .args(["-extract", "CFBundleIdentifier", "raw", "-o", "-"])
        .arg(bundle.join("Contents").join("Info.plist"))
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!id.is_empty() && !id.contains('/')).then_some(id)
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn log_dry_run(sites: &[SiteTarget]) {
    let hosts: Vec<&str> = sites.iter().map(|site| site.host.as_str()).collect();
    println!(
        "[break dry-run] browser policy: {} → {}",
        chromium_bundle_ids().join(", "),
        hosts.join(", ")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url_types(json: &str) -> serde_json::Value {
        serde_json::from_str(json).expect("json")
    }

    #[test]
    fn an_app_that_opens_http_links_is_a_browser() {
        assert!(declares_web_scheme(&url_types(
            r#"[{"CFBundleURLSchemes":["http","https"]}]"#
        )));
        assert!(declares_web_scheme(&url_types(
            r#"[{"CFBundleURLSchemes":["HTTPS"]}]"#
        )));
    }

    #[test]
    fn an_electron_app_with_only_its_own_scheme_is_not_a_browser() {
        assert!(!declares_web_scheme(&url_types(
            r#"[{"CFBundleURLSchemes":["slack"]},{"CFBundleURLSchemes":["vscode"]}]"#
        )));
    }

    #[test]
    fn an_app_declaring_no_url_types_is_not_a_browser() {
        assert!(!declares_web_scheme(&url_types("[]")));
        assert!(!declares_web_scheme(&url_types("{}")));
    }
}
