use std::fs;
use std::io;
use std::path::Path;

use crate::atomic;
use crate::marker;
use crate::pf::rules::{ANCHOR_NAME, ANCHOR_PATH};

pub const PF_CONF_PATH: &str = "/etc/pf.conf";

pub fn render_lines() -> String {
    format!("anchor \"{ANCHOR_NAME}\"\nload anchor \"{ANCHOR_NAME}\" from \"{ANCHOR_PATH}\"\n")
}

pub fn apply_to_content(content: &str, linked: bool) -> String {
    let block = if linked {
        marker::wrap(&render_lines())
    } else {
        String::new()
    };
    marker::apply(content, &block)
}

pub fn is_linked(content: &str) -> bool {
    marker::contains_block(content) && content.contains(&format!("anchor \"{ANCHOR_NAME}\""))
}

pub fn read_and_write(path: &Path, linked: bool) -> io::Result<bool> {
    let current = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(err),
    };

    let next = apply_to_content(&current, linked);
    if next == current {
        return Ok(false);
    }

    atomic::write(path, &next)?;
    Ok(true)
}
