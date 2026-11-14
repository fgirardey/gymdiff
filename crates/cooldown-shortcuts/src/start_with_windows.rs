//! Start cooldown.lol when the player signs in to Windows: the value `cooldown.lol` in
//! the user's `Run` key opens Velopack's launcher with [`BACKGROUND_ARG`]
//! (`docs/Product/Features/Running in the background.md`, ADR-0013).
//!
//! Task Manager's *Startup apps* can disable it without removing it, in the
//! `StartupApproved\Run` key: that counts as off, and turning it on clears it.
//! Per user, no administrator rights. Off Windows it is never on and cannot
//! be turned on.

use std::io;
use std::path::Path;

/// cooldown.lol's startup entry.
pub struct StartWithWindows;

impl StartWithWindows {
    /// True when Windows will start cooldown.lol at sign-in.
    #[must_use]
    pub fn is_enabled() -> bool {
        registry::read().is_some_and(|entry| entry.approved.as_deref().is_none_or(approved))
    }

    /// Turns starting with Windows on or off.
    pub fn set(enabled: bool) -> io::Result<()> {
        if enabled {
            let launcher = crate::launcher().ok_or_else(crate::install::unsupported)?;
            registry::write(&command_line(&launcher))
        } else {
            registry::remove()
        }
    }
}

/// Argument of the startup entry: cooldown.lol starts with only its notification
/// area icon, no window.
pub const BACKGROUND_ARG: &str = "--background";

/// What the `Run` value holds.
fn command_line(launcher: &Path) -> String {
    format!("\"{}\" {BACKGROUND_ARG}", launcher.display())
}

/// Task Manager's state: an odd first byte (`03`, `07`…) means disabled,
/// even (`02`, `06`…) or nothing means enabled.
fn approved(bytes: &[u8]) -> bool {
    bytes.first().is_none_or(|first| first & 1 == 0)
}

/// The entry as read from the registry.
struct Entry {
    /// Task Manager's override, when there is one.
    approved: Option<Vec<u8>>,
}

#[cfg(windows)]
mod registry {
    use std::io;

    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};

    use super::Entry;

    /// Name of the value, in both keys.
    const VALUE_NAME: &str = "cooldown.lol";
    const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const APPROVED: &str =
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";

    pub(super) fn read() -> Option<Entry> {
        let user = RegKey::predef(HKEY_CURRENT_USER);
        let run = user.open_subkey_with_flags(RUN, KEY_READ).ok()?;
        let _: String = run.get_value(VALUE_NAME).ok()?;
        let approved = user
            .open_subkey_with_flags(APPROVED, KEY_READ)
            .and_then(|key| key.get_raw_value(VALUE_NAME))
            .ok()
            .map(|value| value.bytes.into_owned());
        Some(Entry { approved })
    }

    pub(super) fn write(command_line: &str) -> io::Result<()> {
        let user = RegKey::predef(HKEY_CURRENT_USER);
        let (run, _) = user.create_subkey(RUN)?;
        run.set_value(VALUE_NAME, &command_line)?;
        clear_approval(&user)
    }

    pub(super) fn remove() -> io::Result<()> {
        let user = RegKey::predef(HKEY_CURRENT_USER);
        match user.open_subkey_with_flags(RUN, KEY_SET_VALUE) {
            Ok(run) => ignore_missing(run.delete_value(VALUE_NAME))?,
            Err(error) => ignore_missing(Err(error))?,
        }
        clear_approval(&user)
    }

    /// Forgets Task Manager's choice, so that "on" really starts cooldown.lol.
    fn clear_approval(user: &RegKey) -> io::Result<()> {
        match user.open_subkey_with_flags(APPROVED, KEY_SET_VALUE) {
            Ok(approved) => ignore_missing(approved.delete_value(VALUE_NAME)),
            Err(error) => ignore_missing(Err(error)),
        }
    }

    fn ignore_missing(result: io::Result<()>) -> io::Result<()> {
        match result {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }
}

#[cfg(not(windows))]
mod registry {
    use std::io;

    use super::Entry;

    pub(super) fn read() -> Option<Entry> {
        None
    }

    pub(super) fn write(_command_line: &str) -> io::Result<()> {
        Err(crate::install::unsupported())
    }

    #[allow(clippy::unnecessary_wraps)] // Same signature as the Windows version.
    pub(super) fn remove() -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_command_line_quotes_the_launcher() {
        let line = command_line(Path::new(
            r"C:\Users\Me Too\AppData\Local\cooldown.lol\cooldown.exe",
        ));
        assert_eq!(
            line,
            r#""C:\Users\Me Too\AppData\Local\cooldown.lol\cooldown.exe" --background"#
        );
    }

    #[test]
    fn task_manager_disables_with_an_odd_first_byte() {
        assert!(approved(&[]));
        assert!(approved(&[0x02, 0, 0, 0]));
        assert!(approved(&[0x06]));
        assert!(!approved(&[0x03, 0, 0, 0]));
        assert!(!approved(&[0x07]));
    }

    #[cfg(not(windows))]
    #[test]
    fn off_windows_it_is_off_and_stays_off() {
        assert!(!StartWithWindows::is_enabled());
        assert!(StartWithWindows::set(false).is_ok());
        let error = StartWithWindows::set(true).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
    }
}
