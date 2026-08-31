pub mod install;
pub mod paths;
pub mod plist;
pub mod status;

#[derive(Debug, thiserror::Error)]
pub enum DaemonError {
    #[error("설치를 취소했습니다.")]
    Cancelled,
    #[error("차단 프로그램 파일을 찾지 못했습니다: {0}")]
    BinaryNotFound(String),
    #[error("{0}")]
    Script(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}
