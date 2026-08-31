use std::path::PathBuf;

pub const LABEL: &str = "com.movie42.break.daemon";
pub const BINARY_NAME: &str = "break-daemon";
pub const INSTALL_DIR: &str = "/Library/Application Support/Break";
pub const PLIST_PATH: &str = "/Library/LaunchDaemons/com.movie42.break.daemon.plist";
pub const LOG_DIR: &str = "/Library/Logs/Break";
pub const LOG_PATH: &str = "/Library/Logs/Break/daemon.log";

pub fn installed_binary() -> PathBuf {
    PathBuf::from(INSTALL_DIR).join(BINARY_NAME)
}

pub fn source_binary() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;

    let candidates = [
        dir.join(BINARY_NAME),
        dir.join("../Resources").join(BINARY_NAME),
    ];

    candidates.into_iter().find(|path| path.is_file())
}
