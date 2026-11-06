//! macOS: the development platform. Windows-only features are stubbed here.

use std::path::PathBuf;

pub const NAME: &str = "macos";

/// `~/Library/Logs/GymDiff`
pub fn log_dir() -> PathBuf {
    std::env::var_os("HOME").map_or_else(
        || PathBuf::from("logs"),
        |home| {
            PathBuf::from(home)
                .join("Library")
                .join("Logs")
                .join("GymDiff")
        },
    )
}

pub fn configure_main_window(_window: &slint::Window) {
    tracing::debug!("macOS: overlay features are stubbed (development platform)");
}
