use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;

use crate::rules::{Rules, RULES_VERSION};

pub const APP_DIR_NAME: &str = "com.movie42.break";
pub const RULES_FILE_NAME: &str = "rules.json";

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("규칙 파일 경로를 찾을 수 없습니다")]
    NoRulesDir,
    #[error("규칙 파일 입출력에 실패했습니다: {0}")]
    Io(#[from] std::io::Error),
    #[error("규칙을 JSON으로 변환하지 못했습니다: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("규칙 파일이 이 프로그램보다 새 버전입니다 (파일 {found}, 아는 버전 {known}). 차단 프로그램을 다시 설치하세요")]
    UnsupportedVersion { found: u32, known: u32 },
}

#[cfg(target_os = "macos")]
pub fn rules_dir() -> Result<PathBuf, StoreError> {
    let base = directories::BaseDirs::new().ok_or(StoreError::NoRulesDir)?;
    Ok(base
        .home_dir()
        .join("Library")
        .join("Application Support")
        .join(APP_DIR_NAME))
}

#[cfg(target_os = "windows")]
pub fn rules_dir() -> Result<PathBuf, StoreError> {
    let base = directories::BaseDirs::new().ok_or(StoreError::NoRulesDir)?;
    Ok(base.data_dir().join("Break"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn rules_dir() -> Result<PathBuf, StoreError> {
    let base = directories::BaseDirs::new().ok_or(StoreError::NoRulesDir)?;
    Ok(base.data_dir().join(APP_DIR_NAME))
}

pub fn rules_path() -> Result<PathBuf, StoreError> {
    Ok(rules_dir()?.join(RULES_FILE_NAME))
}

pub fn load() -> Result<Rules, StoreError> {
    load_from(&rules_path()?)
}

pub fn save(rules: &Rules) -> Result<(), StoreError> {
    save_to(&rules_path()?, rules)
}

pub fn load_from(path: &Path) -> Result<Rules, StoreError> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Rules::default()),
        Err(err) => return Err(StoreError::Io(err)),
    };

    match serde_json::from_str::<Rules>(&raw) {
        Ok(rules) if rules.version > RULES_VERSION => Err(StoreError::UnsupportedVersion {
            found: rules.version,
            known: RULES_VERSION,
        }),
        Ok(mut rules) => {
            rules.migrate();
            Ok(rules)
        }
        Err(_) => {
            quarantine(path)?;
            Ok(Rules::default())
        }
    }
}

pub fn save_to(path: &Path, rules: &Rules) -> Result<(), StoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(rules)?;
    fs::write(path, json)?;
    Ok(())
}

fn quarantine(path: &Path) -> Result<PathBuf, StoreError> {
    let stamp = Local::now().format("%Y%m%d%H%M%S");
    let mut name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| RULES_FILE_NAME.to_string());
    name.push_str(&format!(".corrupt-{stamp}"));

    let target = path.with_file_name(name);
    fs::rename(path, &target)?;
    Ok(target)
}
