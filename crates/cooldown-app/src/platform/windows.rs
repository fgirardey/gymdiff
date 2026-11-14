//! Windows: the target platform.

use std::path::PathBuf;

pub const NAME: &str = "windows";

/// Windows: a left click on the notification area icon opens the window; the menu is on the right click.
pub const TRAY_MENU_ON_LEFT_CLICK: bool = false;

/// `%LOCALAPPDATA%\cooldown.lol\logs`
pub fn log_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map_or_else(|| PathBuf::from("logs"), PathBuf::from)
        .join("cooldown.lol")
        .join("logs")
}

/// `%APPDATA%\cooldown.lol`
pub fn data_dir() -> PathBuf {
    std::env::var_os("APPDATA")
        .map_or_else(|| PathBuf::from("data"), PathBuf::from)
        .join("cooldown.lol")
}

/// Windows-specific window setup.
///
/// TODO(windows): overlay behaviour (always on top, click-through, positioning
/// over the game window). See docs/Roadmap.md.
pub fn configure_main_window(_window: &slint::Window) {
    tracing::debug!("no Windows-specific window configuration yet");
}
