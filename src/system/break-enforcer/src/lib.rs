use break_core::rules::{AppTarget, SiteTarget};

pub mod hosts;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

pub const DRY_RUN_ENV: &str = "BREAK_DRY_RUN";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("차단을 적용할 권한이 없습니다")]
    NotPrivileged,
    #[error("차단 적용에 실패했습니다: {0}")]
    Io(#[from] std::io::Error),
}

pub fn is_dry_run() -> bool {
    matches!(std::env::var(DRY_RUN_ENV), Ok(value) if value == "1")
}

pub trait Enforcer {
    fn apply_sites(&self, sites: &[SiteTarget]) -> Result<(), Error>;
    fn clear_sites(&self) -> Result<(), Error>;
    fn apply_apps(&self, apps: &[AppTarget]) -> Result<(), Error>;
    fn clear_apps(&self) -> Result<(), Error>;
}

pub fn log_dry_run_sites(sites: &[SiteTarget]) {
    let hosts: Vec<&str> = sites.iter().map(|site| site.host.as_str()).collect();
    println!("[break dry-run] sites: {}", hosts.join(", "));
}

pub fn log_dry_run_apps(apps: &[AppTarget]) {
    let ids: Vec<&str> = apps.iter().map(|app| app.bundle_id.as_str()).collect();
    println!("[break dry-run] apps: {}", ids.join(", "));
}

#[cfg(target_os = "macos")]
pub fn platform_enforcer() -> impl Enforcer {
    macos::MacosEnforcer::new()
}

#[cfg(target_os = "windows")]
pub fn platform_enforcer() -> impl Enforcer {
    windows::WindowsEnforcer::new()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn platform_enforcer() -> impl Enforcer {
    unsupported::UnsupportedEnforcer::new()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub mod unsupported {
    use super::{AppTarget, Enforcer, Error, SiteTarget};

    #[derive(Debug, Default, Clone, Copy)]
    pub struct UnsupportedEnforcer;

    impl UnsupportedEnforcer {
        pub fn new() -> Self {
            Self
        }
    }

    impl Enforcer for UnsupportedEnforcer {
        fn apply_sites(&self, _sites: &[SiteTarget]) -> Result<(), Error> {
            Err(Error::NotPrivileged)
        }
        fn clear_sites(&self) -> Result<(), Error> {
            Err(Error::NotPrivileged)
        }
        fn apply_apps(&self, _apps: &[AppTarget]) -> Result<(), Error> {
            Err(Error::NotPrivileged)
        }
        fn clear_apps(&self) -> Result<(), Error> {
            Err(Error::NotPrivileged)
        }
    }
}
