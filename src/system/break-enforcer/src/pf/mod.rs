pub mod conf;
pub mod resolvers;
pub mod rules;

use std::fs;
use std::io;
use std::net::IpAddr;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{atomic, is_dry_run, Error};

pub const TOKEN_PATH: &str = "/Library/Application Support/Break/pf.token";
pub const STATUS_PATH: &str = "/Library/Application Support/Break/pf.status";
pub const STATUS_OK: &str = "ok";

const PFCTL: &str = "/sbin/pfctl";
const TOKEN_LABEL: &str = "Token";

static LOADED: AtomicBool = AtomicBool::new(false);

pub fn apply(nameservers: &[IpAddr]) -> Result<(), Error> {
    let anchor = rules::render_anchor(nameservers);
    if anchor.is_empty() {
        return clear();
    }

    if is_dry_run() {
        crate::log_dry_run_guard(nameservers);
        return Ok(());
    }

    record(load(&anchor))
}

pub fn clear() -> Result<(), Error> {
    if is_dry_run() {
        crate::log_dry_run_guard(&[]);
        return Ok(());
    }

    LOADED.store(false, Ordering::Relaxed);
    match unload() {
        Ok(()) => {
            let _ = fs::remove_file(STATUS_PATH);
            Ok(())
        }
        Err(err) => {
            let _ = write_if_different(Path::new(STATUS_PATH), &err.to_string());
            Err(err)
        }
    }
}

fn load(anchor: &str) -> Result<(), Error> {
    let first_run = !LOADED.swap(true, Ordering::Relaxed);
    let anchor_changed = write_if_different(Path::new(rules::ANCHOR_PATH), anchor)?;
    let conf_changed = conf::read_and_write(Path::new(conf::PF_CONF_PATH), true)?;

    if conf_changed || first_run {
        pfctl(&["-f", conf::PF_CONF_PATH])?;
    } else if anchor_changed {
        pfctl(&["-a", rules::ANCHOR_NAME, "-f", rules::ANCHOR_PATH])?;
    }

    enable(first_run)
}

fn unload() -> Result<(), Error> {
    let anchor_path = Path::new(rules::ANCHOR_PATH);
    if fs::read_to_string(anchor_path).is_ok_and(|body| !body.trim().is_empty()) {
        atomic::write(anchor_path, "")?;
        let _ = pfctl(&["-a", rules::ANCHOR_NAME, "-F", "rules"]);
    }

    if conf::read_and_write(Path::new(conf::PF_CONF_PATH), false)? {
        pfctl(&["-f", conf::PF_CONF_PATH])?;
    }

    release();
    Ok(())
}

fn enable(force: bool) -> Result<(), Error> {
    let held = held_token();
    if held.is_some() && !force {
        return Ok(());
    }
    if let Some(token) = held {
        let _ = pfctl(&["-X", &token]);
    }

    let output = run(&["-E"])?;
    match read_token(&output) {
        Some(token) => {
            write_if_different(Path::new(TOKEN_PATH), &token)?;
        }
        None => {
            let _ = fs::remove_file(TOKEN_PATH);
        }
    }
    Ok(())
}

fn release() {
    if let Some(token) = held_token() {
        let _ = pfctl(&["-X", &token]);
    }
    let _ = fs::remove_file(TOKEN_PATH);
}

fn held_token() -> Option<String> {
    let token = fs::read_to_string(TOKEN_PATH).ok()?.trim().to_string();
    (!token.is_empty()).then_some(token)
}

pub fn read_token(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let (label, value) = line.trim().split_once(':')?;
        (label.trim() == TOKEN_LABEL).then(|| value.trim())?;
        let token = value.trim();
        (!token.is_empty() && token.bytes().all(|byte| byte.is_ascii_digit()))
            .then(|| token.to_string())
    })
}

fn record(result: Result<(), Error>) -> Result<(), Error> {
    let body = match &result {
        Ok(()) => STATUS_OK.to_string(),
        Err(err) => err.to_string(),
    };
    let _ = write_if_different(Path::new(STATUS_PATH), &body);
    result
}

fn write_if_different(path: &Path, content: &str) -> io::Result<bool> {
    let current = fs::read_to_string(path).unwrap_or_default();
    if current == content {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    atomic::write(path, content)?;
    Ok(true)
}

fn pfctl(args: &[&str]) -> Result<(), Error> {
    run(args).map(|_| ())
}

fn run(args: &[&str]) -> Result<String, Error> {
    let output = Command::new(PFCTL).args(args).output()?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if output.status.success() {
        return Ok(combined);
    }

    Err(Error::Pfctl {
        args: args.join(" "),
        message: combined.trim().lines().next().unwrap_or("").to_string(),
    })
}
