use std::fs;
use std::io;
use std::path::Path;

use break_core::rules::SiteTarget;

use crate::atomic;
use crate::marker;

pub use crate::marker::{strip as strip_block, BEGIN_MARKER, END_MARKER};

pub const HOSTS_PATH: &str = "/etc/hosts";

const BLOCK_ADDRESSES: [&str; 2] = ["127.0.0.1", "::1"];

pub fn render_block(sites: &[SiteTarget]) -> String {
    if sites.is_empty() {
        return String::new();
    }

    let mut body = String::new();
    for address in BLOCK_ADDRESSES {
        for site in sites {
            body.push_str(address);
            body.push('\t');
            body.push_str(&site.host);
            body.push('\n');
            body.push_str(address);
            body.push_str("\twww.");
            body.push_str(&site.host);
            body.push('\n');
        }
    }
    marker::wrap(&body)
}

pub fn apply_to_content(content: &str, sites: &[SiteTarget]) -> String {
    marker::apply(content, &render_block(sites))
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

    atomic::write(path, &next)?;
    Ok(true)
}
