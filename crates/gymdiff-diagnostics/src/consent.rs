//! The player's choice about sending reports, stored locally.

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The stored choice. The default is **off**, with no install id yet.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Consent {
    pub enabled: bool,
    /// Random id, created only when the player opts in; removed on opt-out.
    pub install_id: Option<String>,
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

    /// Turns sending on (creating an install id if needed) or off (forgetting it).
    pub fn set_enabled(&self, enabled: bool) -> io::Result<Consent> {
        let consent = if enabled {
            Consent {
                enabled: true,
                install_id: self
                    .load()
                    .install_id
                    .or_else(|| Some(uuid::Uuid::new_v4().to_string())),
            }
        } else {
            Consent::default()
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
        assert!(!ConsentStore::new(dir.path()).load().enabled);
    }

    #[test]
    fn opt_in_creates_a_stable_id_and_opt_out_forgets_it() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConsentStore::new(dir.path());
        let first = store.set_enabled(true).unwrap();
        assert!(first.enabled && first.install_id.is_some());
        assert_eq!(store.set_enabled(true).unwrap(), first);
        assert_eq!(store.load(), first);

        let off = store.set_enabled(false).unwrap();
        assert_eq!(off, Consent::default());
        assert_eq!(store.load(), off);
    }
}
