use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum LolError {
    #[error("League client lockfile not found (searched: {searched:?})")]
    LockfileNotFound { searched: Vec<PathBuf> },

    #[error("invalid lockfile: {0}")]
    LockfileInvalid(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("unexpected HTTP status {status} for {url}")]
    UnexpectedStatus { status: u16, url: String },
}
