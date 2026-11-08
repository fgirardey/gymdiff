//! Where Velopack installed GymDiff on Windows
//! (`docs/Decisions/ADR-0011 Windows installer and updates with Velopack.md`):
//! the launcher that shortcuts and the startup entry open, and the
//! `Update.exe` that uninstalls. `None` off Windows.

use std::io;
use std::path::PathBuf;

/// Where Velopack installs GymDiff: `%LOCALAPPDATA%\GymDiff`.
#[must_use]
pub fn install_root() -> Option<PathBuf> {
    cfg!(windows)
        .then(dirs::data_local_dir)
        .flatten()
        .map(|folder| folder.join(PACK_ID))
}

/// Velopack's launcher, `%LOCALAPPDATA%\GymDiff\gymdiff.exe`: it starts the
/// current version with the arguments it gets, and stays in place across
/// updates.
#[must_use]
pub fn launcher() -> Option<PathBuf> {
    install_root().map(|root| root.join(LAUNCHER))
}

/// Velopack's `Update.exe`; `Update.exe --uninstall` is what Windows' Apps
/// settings run to uninstall GymDiff.
#[must_use]
pub fn update_exe() -> Option<PathBuf> {
    install_root().map(|root| root.join(UPDATE_EXE))
}

/// The real executable of the current version, which carries the crest icon.
pub(crate) fn current_executable() -> Option<PathBuf> {
    install_root().map(|root| root.join(CURRENT_EXECUTABLE))
}

/// The error of every change attempted where GymDiff is not installed.
pub(crate) fn unsupported() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "only an installed Windows GymDiff has shortcuts and a startup entry",
    )
}

/// Velopack's pack id (`vpk pack --packId`), also the install folder's name.
const PACK_ID: &str = "GymDiff";
/// Velopack's stub at the install root.
const LAUNCHER: &str = "gymdiff.exe";
const UPDATE_EXE: &str = "Update.exe";
const CURRENT_EXECUTABLE: &str = "current\\gymdiff.exe";

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(windows))]
    #[test]
    fn nothing_is_installed_off_windows() {
        assert_eq!(install_root(), None);
        assert_eq!(launcher(), None);
        assert_eq!(update_exe(), None);
    }

    #[cfg(windows)]
    #[test]
    fn everything_lives_in_the_install_root() {
        let root = install_root().unwrap();
        assert!(root.ends_with("GymDiff"));
        assert_eq!(launcher().unwrap(), root.join("gymdiff.exe"));
        assert_eq!(update_exe().unwrap(), root.join("Update.exe"));
    }
}
