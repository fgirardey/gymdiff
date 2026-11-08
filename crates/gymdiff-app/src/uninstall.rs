//! Settings → Uninstall: starts Velopack's uninstaller, the one Windows' Apps
//! settings run, then quits; and cleans up from Velopack's uninstall hook
//! (`docs/Product/Features/Installation.md`, ADR-0013).
//!
//! "Also delete my settings" cannot delete the data folder from the running
//! app (files are open, and a failed uninstall would leave GymDiff without
//! them): it leaves a marker that the uninstall hook acts on once Velopack has
//! stopped GymDiff.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use gymdiff_shortcuts::{Shortcut, StartWithWindows};
use slint::ComponentHandle;

use crate::AppWindow;

/// Shows the section when GymDiff is installed and handles the dialog's
/// Uninstall button.
pub fn bind(window: &AppWindow, data_dir: &Path, installed: bool) {
    window.set_uninstall_visible(installed);
    if !installed {
        return;
    }
    let data_dir = data_dir.to_owned();
    let weak = window.as_weak();
    window.on_uninstall(move |delete_data| match start(&data_dir, delete_data) {
        Ok(()) => {
            tracing::info!(delete_data, "uninstaller started: quitting");
            let _ = slint::quit_event_loop();
        }
        Err(error) => {
            tracing::warn!(%error, "cannot start the uninstaller");
            if let Some(window) = weak.upgrade() {
                window.set_uninstall_failed(true);
            }
        }
    });
}

/// Before Velopack removes GymDiff (its uninstall hook, in a GymDiff of its
/// own after the running one was stopped): removes what GymDiff added to
/// Windows, and the data folder when the player asked for it. Never fails:
/// uninstalling must go on, and there is nowhere to log to.
// Velopack's uninstall hook exists on Windows only; tested everywhere.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn before_uninstall(data_dir: &Path) {
    for shortcut in Shortcut::ALL {
        let _ = shortcut.remove();
    }
    let _ = StartWithWindows::set(false);
    if marker(data_dir).exists() {
        let _ = std::fs::remove_dir_all(data_dir);
    }
}

/// Marks the data folder for deletion if asked, then starts
/// `Update.exe --uninstall`; nothing is left marked when it cannot start.
fn start(data_dir: &Path, delete_data: bool) -> io::Result<()> {
    let update_exe = gymdiff_shortcuts::update_exe()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Unsupported, "GymDiff is not installed"))?;
    if delete_data {
        std::fs::create_dir_all(data_dir)?;
        std::fs::write(marker(data_dir), b"")?;
    }
    let started = Command::new(update_exe).arg("--uninstall").spawn();
    if started.is_err() {
        let _ = std::fs::remove_file(marker(data_dir));
    }
    started.map(drop)
}

/// The marker that asks the uninstall hook to delete the data folder.
fn marker(data_dir: &Path) -> PathBuf {
    data_dir.join(DELETE_MARKER)
}

const DELETE_MARKER: &str = "delete-on-uninstall";

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("gymdiff-uninstall-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("settings.json"), b"{}").unwrap();
        dir
    }

    #[test]
    fn the_hook_keeps_the_data_without_the_marker() {
        let dir = temp_dir("keep");
        before_uninstall(&dir);
        assert!(dir.join("settings.json").exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn the_hook_deletes_the_data_when_marked() {
        let dir = temp_dir("delete");
        std::fs::write(marker(&dir), b"").unwrap();
        before_uninstall(&dir);
        assert!(!dir.exists());
    }

    #[cfg(not(windows))]
    #[test]
    fn nothing_is_marked_when_the_uninstaller_cannot_start() {
        let dir = temp_dir("not-installed");
        assert!(start(&dir, true).is_err());
        assert!(!marker(&dir).exists());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
