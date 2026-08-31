use break_core::rules::{AppTarget, SiteTarget};

use crate::{is_dry_run, log_dry_run_apps, log_dry_run_sites, Enforcer, Error};

#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsEnforcer;

impl WindowsEnforcer {
    pub fn new() -> Self {
        Self
    }
}

impl Enforcer for WindowsEnforcer {
    fn apply_sites(&self, sites: &[SiteTarget]) -> Result<(), Error> {
        if is_dry_run() {
            log_dry_run_sites(sites);
            return Ok(());
        }
        Err(Error::NotPrivileged)
    }

    fn clear_sites(&self) -> Result<(), Error> {
        if is_dry_run() {
            log_dry_run_sites(&[]);
            return Ok(());
        }
        Err(Error::NotPrivileged)
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
