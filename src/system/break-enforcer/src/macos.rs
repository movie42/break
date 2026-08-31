use std::path::Path;
use std::process::Command;

use break_core::rules::{AppTarget, SiteTarget};

use crate::hosts::{self, HOSTS_PATH};
use crate::{is_dry_run, log_dry_run_apps, log_dry_run_sites, Enforcer, Error};

#[derive(Debug, Default, Clone, Copy)]
pub struct MacosEnforcer;

impl MacosEnforcer {
    pub fn new() -> Self {
        Self
    }

    fn write_sites(&self, sites: &[SiteTarget]) -> Result<(), Error> {
        if is_dry_run() {
            log_dry_run_sites(sites);
            return Ok(());
        }

        let changed = hosts::read_and_write(Path::new(HOSTS_PATH), sites).map_err(to_error)?;
        if changed {
            flush_dns_cache();
        }
        Ok(())
    }
}

fn to_error(err: std::io::Error) -> Error {
    match err.kind() {
        std::io::ErrorKind::PermissionDenied => Error::NotPrivileged,
        _ => Error::Io(err),
    }
}

pub fn flush_dns_cache() {
    let _ = Command::new("dscacheutil").arg("-flushcache").status();
    let _ = Command::new("killall")
        .args(["-HUP", "mDNSResponder"])
        .status();
}

impl Enforcer for MacosEnforcer {
    fn apply_sites(&self, sites: &[SiteTarget]) -> Result<(), Error> {
        self.write_sites(sites)
    }

    fn clear_sites(&self) -> Result<(), Error> {
        self.write_sites(&[])
    }

    fn apply_apps(&self, apps: &[AppTarget]) -> Result<(), Error> {
        if is_dry_run() {
            log_dry_run_apps(apps);
            return Ok(());
        }
        Err(Error::NotPrivileged)
    }

    fn clear_apps(&self) -> Result<(), Error> {
        if is_dry_run() {
            log_dry_run_apps(&[]);
            return Ok(());
        }
        Err(Error::NotPrivileged)
    }
}
