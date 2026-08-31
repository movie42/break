use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::paths::{
    installed_binary, source_binary, INSTALL_DIR, LABEL, LOG_DIR, LOG_PATH, PLIST_PATH,
};
use super::{plist, DaemonError};

pub fn install(rules_path: &Path) -> Result<(), DaemonError> {
    let source = source_binary().ok_or_else(|| {
        DaemonError::BinaryNotFound(super::paths::BINARY_NAME.to_string())
    })?;

    let staged_binary = stage_binary(&source)?;
    let staged_plist = stage_plist(rules_path)?;
    let script = install_script(&staged_binary, &staged_plist);
    let result = run_privileged(&script);
    let _ = fs::remove_file(&staged_binary);
    let _ = fs::remove_file(&staged_plist);
    result
}

pub fn uninstall(rules_path: &Path) -> Result<(), DaemonError> {
    run_privileged(&uninstall_script(rules_path))
}

fn stage_binary(source: &Path) -> Result<PathBuf, DaemonError> {
    let path = std::env::temp_dir().join(super::paths::BINARY_NAME);
    let _ = fs::remove_file(&path);
    fs::copy(source, &path)?;
    Ok(path)
}

fn stage_plist(rules_path: &Path) -> Result<PathBuf, DaemonError> {
    let path = std::env::temp_dir().join(format!("{LABEL}.plist"));
    fs::write(&path, plist::render(rules_path))?;
    Ok(path)
}

fn install_script(source: &Path, staged_plist: &Path) -> String {
    let binary = installed_binary();
    [
        "cd /".to_string(),
        format!("mkdir -p {}", sh_quote(INSTALL_DIR)),
        format!("mkdir -p {}", sh_quote(LOG_DIR)),
        format!(
            "cp {} {}",
            sh_quote(&source.to_string_lossy()),
            sh_quote(&binary.to_string_lossy())
        ),
        format!("chown root:wheel {}", sh_quote(&binary.to_string_lossy())),
        format!("chmod 755 {}", sh_quote(&binary.to_string_lossy())),
        format!(
            "cp {} {}",
            sh_quote(&staged_plist.to_string_lossy()),
            sh_quote(PLIST_PATH)
        ),
        format!("chown root:wheel {}", sh_quote(PLIST_PATH)),
        format!("chmod 644 {}", sh_quote(PLIST_PATH)),
        format!("touch {}", sh_quote(LOG_PATH)),
        format!("launchctl bootout system/{LABEL} 2>/dev/null || true"),
        format!("launchctl bootstrap system {}", sh_quote(PLIST_PATH)),
    ]
    .join(" && ")
}

fn uninstall_script(rules_path: &Path) -> String {
    let binary = installed_binary();
    [
        "cd /".to_string(),
        format!(
            "{} --rules {} --clear 2>/dev/null || true",
            sh_quote(&binary.to_string_lossy()),
            sh_quote(&rules_path.to_string_lossy())
        ),
        format!("launchctl bootout system/{LABEL} 2>/dev/null || true"),
        format!("rm -f {}", sh_quote(PLIST_PATH)),
        format!("rm -f {}", sh_quote(&binary.to_string_lossy())),
    ]
    .join(" && ")
}

fn run_privileged(script: &str) -> Result<(), DaemonError> {
    let applescript = format!(
        "do shell script \"{}\" with administrator privileges",
        escape_applescript(script)
    );

    let output = Command::new("osascript").args(["-e", &applescript]).output()?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if is_cancelled(&stderr) {
        return Err(DaemonError::Cancelled);
    }
    Err(DaemonError::Script(stderr))
}

fn is_cancelled(stderr: &str) -> bool {
    stderr.contains("-128") || stderr.contains("User canceled")
}

fn sh_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn escape_applescript(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_with_spaces_stays_one_argument() {
        assert_eq!(sh_quote("/Users/한 사람/rules.json"), "'/Users/한 사람/rules.json'");
    }

    #[test]
    fn a_single_quote_in_a_path_is_escaped() {
        assert_eq!(sh_quote("/Users/o'brien/r.json"), "'/Users/o'\\''brien/r.json'");
    }

    #[test]
    fn both_scripts_leave_the_protected_home_directory_first() {
        assert!(install_script(Path::new("/tmp/break-daemon"), Path::new("/tmp/x.plist"))
            .starts_with("cd / &&"));
        assert!(uninstall_script(Path::new("/tmp/rules.json")).starts_with("cd / &&"));
    }

    #[test]
    fn the_uninstall_script_clears_hosts_before_deleting_the_binary() {
        let script = uninstall_script(Path::new("/tmp/rules.json"));
        let clear = script.find("--clear").expect("clear step");
        let remove = script.rfind("rm -f").expect("remove step");
        assert!(clear < remove);
    }

    #[test]
    fn applescript_escaping_keeps_quotes_from_closing_the_literal() {
        assert_eq!(escape_applescript(r#"echo "hi""#), r#"echo \"hi\""#);
        assert_eq!(escape_applescript(r"a\b"), r"a\\b");
    }

    #[test]
    fn a_cancelled_password_prompt_is_not_a_failure() {
        assert!(is_cancelled("execution error: 사용자가 취소했습니다. (-128)"));
        assert!(!is_cancelled("execution error: something else (1)"));
    }
}
