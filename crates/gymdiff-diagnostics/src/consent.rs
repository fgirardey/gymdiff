//! The player's choices about sending reports, stored locally.

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The stored choices, one per Settings switch. The default is **all off**,
/// with no install id yet.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Consent {
    /// "Send crash reports": errors and panics. Named `enabled` in files
    /// written before the logs switch existed.
    #[serde(alias = "enabled")]
    pub crash_reports: bool,
    /// "Send all logs": everyday logs (`INFO` and above).
    pub logs: bool,
    /// Random id, created when the player turns a switch on; removed when
    /// both are off.
    pub install_id: Option<String>,
}

impl Consent {
    /// True when at least one kind of report may be sent.
    #[must_use]
    pub fn any(&self) -> bool {
        self.crash_reports || self.logs
    }
}

/// Reads and writes [`Consent`] in the app's data folder.
#[derive(Debug, Clone)]
pub struct ConsentStore {
    path: PathBuf,
}

impl ConsentStore {
    #[must_use]
    pub fn new(data_dir: &Path) -> Self {
        Self {
            path: data_dir.join(FILE_NAME),
        }
    }

    /// Any read or parse error means "no consent": never send on doubt.
    #[must_use]
    pub fn load(&self) -> Consent {
        std::fs::read_to_string(&self.path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    /// Turns crash reports on or off.
    pub fn set_crash_reports(&self, enabled: bool) -> io::Result<Consent> {
        self.change(|consent| consent.crash_reports = enabled)
    }

    /// Turns sending all logs on or off.
    pub fn set_logs(&self, enabled: bool) -> io::Result<Consent> {
        self.change(|consent| consent.logs = enabled)
    }

    /// Applies `change`, creates the install id when a switch is on and
    /// forgets it when both are off, then saves.
    fn change(&self, change: impl FnOnce(&mut Consent)) -> io::Result<Consent> {
        let mut consent = self.load();
        change(&mut consent);
        consent.install_id = if consent.any() {
            consent
                .install_id
                .or_else(|| Some(uuid::Uuid::new_v4().to_string()))
        } else {
            None
        };
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&self.path, serde_json::to_string_pretty(&consent)?)?;
        Ok(consent)
    }
}

/// File name, in the data folder.
const FILE_NAME: &str = "diagnostics.json";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_off_without_a_file() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(ConsentStore::new(dir.path()).load(), Consent::default());
    }

    #[test]
    fn corrupt_file_means_off() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(FILE_NAME), "{not json").unwrap();
        assert!(!ConsentStore::new(dir.path()).load().any());
    }

    #[test]
    fn reads_files_from_before_the_logs_switch() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(FILE_NAME),
            r#"{"enabled": true, "install_id": "abc"}"#,
        )
        .unwrap();
        let consent = ConsentStore::new(dir.path()).load();
        assert!(consent.crash_reports && !consent.logs);
        assert_eq!(consent.install_id.as_deref(), Some("abc"));
    }

    #[test]
    fn opt_in_creates_a_stable_id_and_opt_out_forgets_it() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConsentStore::new(dir.path());
        let first = store.set_crash_reports(true).unwrap();
        assert!(first.crash_reports && first.install_id.is_some());
        assert_eq!(store.set_crash_reports(true).unwrap(), first);
        assert_eq!(store.load(), first);

        let off = store.set_crash_reports(false).unwrap();
        assert_eq!(off, Consent::default());
        assert_eq!(store.load(), off);
    }

    #[test]
    fn the_id_lives_while_either_switch_is_on() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConsentStore::new(dir.path());
        let id = store.set_logs(true).unwrap().install_id;
        assert!(id.is_some());
        assert_eq!(store.set_crash_reports(true).unwrap().install_id, id);
        assert_eq!(store.set_logs(false).unwrap().install_id, id);
        assert_eq!(store.set_crash_reports(false).unwrap().install_id, None);
    }
}
