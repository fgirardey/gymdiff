//! macOS: the development platform. Windows-only features are stubbed here.

use std::path::PathBuf;

pub const NAME: &str = "macos";

/// macOS: menu bar icons open their menu on a click, like the system ones.
pub const TRAY_MENU_ON_LEFT_CLICK: bool = true;

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

/// `~/Library/Application Support/GymDiff`
pub fn data_dir() -> PathBuf {
    std::env::var_os("HOME").map_or_else(
        || PathBuf::from("data"),
        |home| {
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("GymDiff")
        },
    )
}

pub fn configure_main_window(_window: &slint::Window) {
    tracing::debug!("macOS: overlay features are stubbed (development platform)");
}
