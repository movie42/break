use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn write(path: &Path, content: &str) -> io::Result<()> {
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

fn temp_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "break".to_string());
    path.with_file_name(format!(".{name}.break-tmp"))
}
