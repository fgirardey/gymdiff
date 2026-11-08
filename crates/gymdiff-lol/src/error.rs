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

impl LolError {
    /// True when the API answered but the payload did not match our models:
    /// Riot probably changed the API. Unlike a refused connection or an error
    /// status (normal while the client starts, closes or loads a game), this is
    /// worth a warning.
    #[must_use]
    pub fn is_unexpected_payload(&self) -> bool {
        matches!(self, Self::Http(error) if error.is_decode())
    }
}
