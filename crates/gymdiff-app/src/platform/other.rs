//! Other platforms (Linux CI, etc.): best-effort, not supported.

use std::path::PathBuf;

pub const NAME: &str = "other";

pub fn log_dir() -> PathBuf {
    PathBuf::from("logs")
}

pub fn data_dir() -> PathBuf {
    PathBuf::from("data")
}

pub fn configure_main_window(_window: &slint::Window) {}
