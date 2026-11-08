// The installer is a GUI app on Windows: no console window next to it.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! # GymDiff installer
//!
//! A small window in GymDiff's look (`ui/installer-window.slint` in the app
//! crate): the player picks the Start menu and Desktop shortcuts, the installer
//! runs Velopack's setup it embeds (`--silent`: no Velopack window, no launch),
//! creates the chosen shortcuts and launches GymDiff if asked
//! (`docs/Product/Features/Installation.md`, ADR-0012).

use std::path::Path;
use std::process::Command;

use gymdiff_shortcuts::Shortcut;
use slint::ComponentHandle;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let window = InstallerWindow::new()?;
    window.set_version(VERSION.into());

    let weak = window.as_weak();
    window.on_install(move || {
        let Some(window) = weak.upgrade() else { return };
        let choices = [
            (Shortcut::StartMenu, window.get_start_menu()),
            (Shortcut::Desktop, window.get_desktop()),
        ];
        window.set_step(InstallerStep::Installing);
        let weak = weak.clone();
        let spawned = std::thread::Builder::new()
            .name("install".into())
            .spawn(move || {
                let result = install().map(|()| apply_shortcuts(&choices));
                let _ = weak.upgrade_in_event_loop(move |window| match result {
                    Ok(()) => window.set_step(InstallerStep::Done),
                    Err(failure) => fail(&window, failure),
                });
            });
        if spawned.is_err() {
            fail(&window, Failure::Prepare);
        }
    });

    window.on_quit(|| {
        let _ = slint::quit_event_loop();
    });

    let weak = window.as_weak();
    window.on_finish(move || {
        if weak.upgrade().is_some_and(|window| window.get_launch()) {
            launch();
        }
        let _ = slint::quit_event_loop();
    });

    window.run()
}

/// Velopack's `GymDiff-<channel>-Setup.exe`, embedded by `build.rs`; empty in
/// local builds.
const PAYLOAD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/payload.exe"));
const VERSION: &str = env!("GYMDIFF_INSTALLER_VERSION");

/// Why installing failed, as shown on the Failed step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Failure {
    /// A local build: no Velopack setup inside.
    NoPayload,
    /// The setup could not be written or started.
    Prepare,
    /// The setup ran and reported an error.
    Interrupted,
}

impl Failure {
    fn reason(self) -> &'static str {
        match self {
            Self::NoPayload => "this installer was built without GymDiff inside",
            Self::Prepare => "the installation could not start",
            Self::Interrupted => "the installation was interrupted",
        }
    }
}

/// Writes Velopack's setup to a temporary folder and runs it silently.
fn install() -> Result<(), Failure> {
    if PAYLOAD.is_empty() {
        return Err(Failure::NoPayload);
    }
    let folder = std::env::temp_dir().join(format!("gymdiff-setup-{}", std::process::id()));
    let result = run_setup(&folder);
    // Best effort: the temporary folder is the system's to clean otherwise.
    let _ = std::fs::remove_dir_all(&folder);
    result
}

fn run_setup(folder: &Path) -> Result<(), Failure> {
    std::fs::create_dir_all(folder).map_err(|_| Failure::Prepare)?;
    let setup = folder.join("GymDiff-Setup.exe");
    std::fs::write(&setup, PAYLOAD).map_err(|_| Failure::Prepare)?;
    let status = Command::new(&setup)
        .arg("--silent")
        .status()
        .map_err(|_| Failure::Prepare)?;
    if status.success() {
        Ok(())
    } else {
        Err(Failure::Interrupted)
    }
}

/// Creates the chosen shortcuts and removes the others (a reinstall applies
/// the new choices). A shortcut that fails is not worth failing the install:
/// Settings can add it later.
fn apply_shortcuts(choices: &[(Shortcut, bool)]) {
    for &(shortcut, wanted) in choices {
        let _ = shortcut.set(wanted);
    }
}

fn fail(window: &InstallerWindow, failure: Failure) {
    window.set_failure_reason(failure.reason().into());
    window.set_step(InstallerStep::Failed);
}

/// Starts the installed GymDiff through Velopack's launcher; it keeps running
/// after the installer closes.
fn launch() {
    if let Some(launcher) = gymdiff_shortcuts::launcher() {
        let _ = Command::new(launcher).spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reasons_fit_the_failed_sentence() {
        for failure in [Failure::NoPayload, Failure::Prepare, Failure::Interrupted] {
            let reason = failure.reason();
            assert!(reason.chars().next().is_some_and(char::is_lowercase));
            assert!(!reason.ends_with('.'));
        }
    }
}
