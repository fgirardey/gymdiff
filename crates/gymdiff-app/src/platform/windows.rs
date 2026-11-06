//! Windows: the target platform.

use std::path::PathBuf;

pub const NAME: &str = "windows";

/// `%LOCALAPPDATA%\GymDiff\logs`
pub fn log_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map_or_else(|| PathBuf::from("logs"), PathBuf::from)
        .join("GymDiff")
        .join("logs")
}

/// Windows-specific window setup.
///
/// TODO(windows): overlay behaviour (always on top, click-through, positioning
/// over the game window). See docs/Roadmap.md.
pub fn configure_main_window(_window: &slint::Window) {
    tracing::debug!("no Windows-specific window configuration yet");
}
