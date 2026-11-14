//! cooldown.lol's Start menu and Desktop shortcuts on Windows, created by the
//! installer and by Settings, removed by Settings and on uninstall
//! (`docs/Decisions/ADR-0012 Installer window in Slint over Velopack.md`).
//!
//! Shortcuts point at Velopack's launcher ([`crate::launcher`]), which stays
//! in place across updates. On other systems there is no shortcut:
//! [`Shortcut::path`] is `None` and changes fail with
//! [`ErrorKind::Unsupported`](std::io::ErrorKind::Unsupported).

use std::io;
use std::path::{Path, PathBuf};

use crate::install::{current_executable, launcher, unsupported};

/// Where a cooldown.lol shortcut can live.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shortcut {
    /// `%APPDATA%\Microsoft\Windows\Start Menu\Programs\cooldown.lol.lnk`: listed in
    /// the Start menu and found by Windows Search.
    StartMenu,
    /// `cooldown.lol.lnk` on the user's Desktop (wherever Windows redirects it).
    Desktop,
}

impl Shortcut {
    /// Both locations, in display order.
    pub const ALL: [Self; 2] = [Self::StartMenu, Self::Desktop];

    /// Where this shortcut is, or would be; `None` off Windows or when the
    /// folder is unknown.
    #[must_use]
    pub fn path(self) -> Option<PathBuf> {
        if !cfg!(windows) {
            return None;
        }
        let folder = match self {
            Self::StartMenu => dirs::data_dir()?.join(START_MENU_PROGRAMS),
            Self::Desktop => dirs::desktop_dir()?,
        };
        Some(folder.join(FILE_NAME))
    }

    /// True when the shortcut exists.
    #[must_use]
    pub fn exists(self) -> bool {
        self.path().is_some_and(|path| path.exists())
    }

    /// Creates the shortcut to the installed cooldown.lol, replacing an older one.
    pub fn create(self) -> io::Result<()> {
        let path = self.path().ok_or_else(unsupported)?;
        let (Some(target), Some(icon_source)) = (launcher(), current_executable()) else {
            return Err(unsupported());
        };
        if let Some(folder) = path.parent() {
            std::fs::create_dir_all(folder)?;
        }
        write_link(&path, &target, &icon_source)
    }

    /// Removes the shortcut; removing a missing one is fine.
    pub fn remove(self) -> io::Result<()> {
        let Some(path) = self.path() else {
            return Ok(());
        };
        match std::fs::remove_file(path) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        }
    }

    /// Creates the shortcut when `wanted`, removes it otherwise.
    pub fn set(self, wanted: bool) -> io::Result<()> {
        if wanted { self.create() } else { self.remove() }
    }
}

/// Name of the shortcut file in both locations.
const FILE_NAME: &str = "cooldown.lol.lnk";
/// The Start menu's programs folder, under the roaming application data.
const START_MENU_PROGRAMS: &str = "Microsoft\\Windows\\Start Menu\\Programs";

/// Writes a `.lnk` at `path` that opens `target` with the icon of
/// `icon_source`. `target` must exist.
#[cfg(windows)]
fn write_link(path: &Path, target: &Path, icon_source: &Path) -> io::Result<()> {
    // mslnk unwraps these conversions: check them instead of panicking.
    let (Some(_), Some(icon)) = (target.to_str(), icon_source.to_str()) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "shortcut paths must be valid Unicode",
        ));
    };
    let mut link = mslnk::ShellLink::new(target).map_err(|error| link_error(&error))?;
    link.set_name(Some("cooldown.lol".to_owned()));
    link.set_icon_location(Some(icon.to_owned()));
    link.create_lnk(path).map_err(|error| link_error(&error))
}

#[cfg(windows)]
fn link_error(error: &mslnk::MSLinkError) -> io::Error {
    io::Error::other(error.to_string())
}

#[cfg(not(windows))]
fn write_link(_path: &Path, _target: &Path, _icon_source: &Path) -> io::Result<()> {
    Err(unsupported())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn writes_a_shell_link() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("cooldown.exe");
        std::fs::write(&target, b"MZ").unwrap();
        let link = dir.path().join("cooldown.lol.lnk");

        write_link(&link, &target, &target).unwrap();

        let bytes = std::fs::read(&link).unwrap();
        // Shell link header: size 0x4C, then the class id 00021401-….
        assert_eq!(&bytes[..4], &[0x4C, 0, 0, 0]);
        assert_eq!(&bytes[4..8], &[0x01, 0x14, 0x02, 0x00]);
    }

    #[cfg(windows)]
    #[test]
    fn a_missing_target_is_an_error_not_a_panic() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.exe");
        assert!(write_link(&dir.path().join("x.lnk"), &missing, &missing).is_err());
    }

    #[test]
    fn removing_a_missing_shortcut_is_fine() {
        for shortcut in Shortcut::ALL {
            if shortcut.path().is_none() {
                assert!(shortcut.remove().is_ok());
                assert!(!shortcut.exists());
            }
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn there_are_no_shortcuts_off_windows() {
        assert_eq!(Shortcut::StartMenu.path(), None);
        let error = Shortcut::Desktop.create().unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
    }
}
