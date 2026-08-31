use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use break_core::rules::SiteTarget;

pub const HOSTS_PATH: &str = "/etc/hosts";
pub const BEGIN_MARKER: &str = "# BEGIN Break — 이 블록은 Break가 관리합니다. 직접 고치지 마세요.";
pub const END_MARKER: &str = "# END Break";

const BLOCK_ADDRESSES: [&str; 2] = ["127.0.0.1", "::1"];

pub fn render_block(sites: &[SiteTarget]) -> String {
    if sites.is_empty() {
        return String::new();
    }

    let mut out = String::from(BEGIN_MARKER);
    out.push('\n');
    for address in BLOCK_ADDRESSES {
        for site in sites {
            out.push_str(address);
            out.push('\t');
            out.push_str(&site.host);
            out.push('\n');
            out.push_str(address);
            out.push_str("\twww.");
            out.push_str(&site.host);
            out.push('\n');
        }
    }
    out.push_str(END_MARKER);
    out.push('\n');
    out
}

pub fn strip_block(content: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    let mut inside = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == BEGIN_MARKER {
            inside = true;
            continue;
        }
        if inside {
            if trimmed == END_MARKER {
                inside = false;
            }
            continue;
        }
        kept.push(line);
    }

    while kept.last().is_some_and(|line| line.trim().is_empty()) {
        kept.pop();
    }

    if kept.is_empty() {
        return String::new();
    }

    let mut out = kept.join("\n");
    out.push('\n');
    out
}

pub fn apply_to_content(content: &str, sites: &[SiteTarget]) -> String {
    let base = strip_block(content);
    let block = render_block(sites);
    if block.is_empty() {
        return base;
    }
    if base.is_empty() {
        return block;
    }
    format!("{base}\n{block}")
}

pub fn read_and_write(path: &Path, sites: &[SiteTarget]) -> io::Result<bool> {
    let current = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(err),
    };

    let next = apply_to_content(&current, sites);
    if next == current {
        return Ok(false);
    }

    write_atomically(path, &next)?;
    Ok(true)
}

fn temp_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "hosts".to_string());
    path.with_file_name(format!(".{name}.break-tmp"))
}

fn write_atomically(path: &Path, content: &str) -> io::Result<()> {
    let temp = temp_path(path);
    fs::write(&temp, content)?;

    if let Ok(meta) = fs::metadata(path) {
        if let Err(err) = fs::set_permissions(&temp, meta.permissions()) {
            let _ = fs::remove_file(&temp);
            return Err(err);
        }
    }

    match fs::rename(&temp, path) {
        Ok(()) => Ok(()),
        Err(err) => {
            let _ = fs::remove_file(&temp);
            Err(err)
        }
    }
}
