use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to write {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid project configuration in {path}: {source}")]
    Config {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("required tool `{0}` was not found; run `kpdk doctor`")]
    ToolMissing(String),
    #[error("`{program}` exited with status {status}")]
    Process { program: String, status: i32 },
}

pub type Result<T> = std::result::Result<T, Error>;
