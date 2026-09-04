use std::fs;
use std::net::IpAddr;
use std::process::Command;

pub const RESOLV_CONF_PATH: &str = "/etc/resolv.conf";

pub fn system_nameservers() -> Vec<IpAddr> {
    let from_scutil = Command::new("/usr/sbin/scutil")
        .arg("--dns")
        .output()
        .map(|output| parse_nameservers(&String::from_utf8_lossy(&output.stdout)))
        .unwrap_or_default();

    if !from_scutil.is_empty() {
        return from_scutil;
    }

    fs::read_to_string(RESOLV_CONF_PATH)
        .map(|content| parse_resolv_conf(&content))
        .unwrap_or_default()
}

pub fn parse_resolv_conf(content: &str) -> Vec<IpAddr> {
    let mut found: Vec<IpAddr> = Vec::new();
    for line in content.lines() {
        let Some(value) = line.trim().strip_prefix("nameserver") else {
            continue;
        };
        let Some(address) = parse_address(value.trim()) else {
            continue;
        };
        if !found.contains(&address) {
            found.push(address);
        }
    }
    found
}

pub fn parse_nameservers(output: &str) -> Vec<IpAddr> {
    let mut found: Vec<IpAddr> = Vec::new();
    for line in output.lines() {
        let Some(address) = nameserver_line(line) else {
            continue;
        };
        if !found.contains(&address) {
            found.push(address);
        }
    }
    found
}

pub fn blockable(nameservers: &[IpAddr]) -> Vec<IpAddr> {
    nameservers
        .iter()
        .copied()
        .filter(|address| !address.is_loopback() && !address.is_unspecified())
        .collect()
}

fn nameserver_line(line: &str) -> Option<IpAddr> {
    let rest = line.trim().strip_prefix("nameserver[")?;
    let (index, after) = rest.split_once(']')?;
    if index.is_empty() || !index.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    parse_address(after.trim_start().strip_prefix(':')?.trim())
}

fn parse_address(value: &str) -> Option<IpAddr> {
    value.split('%').next()?.parse().ok()
}
