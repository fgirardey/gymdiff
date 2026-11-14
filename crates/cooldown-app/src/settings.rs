//! The player's preferences, one `settings.json` in the app's data folder
//! ([`crate::platform::data_dir`]). See `docs/Decisions/ADR-0007 Local persistence.md`.
//!
//! The consent to send error reports is **not** here on purpose: it lives in
//! `diagnostics.json` ([`cooldown_diagnostics::ConsentStore`]) so that a damaged
//! settings file can never switch reporting on.

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Every preference, with its default. Unknown or missing fields fall back to
/// the default, so an older or newer file never blocks the app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Schema version, to migrate the file when a field changes meaning.
    pub version: u32,
    /// The player has gone through the welcome screen (see `docs/Product/Features/First launch.md`).
    pub welcome_seen: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            welcome_seen: false,
        }
    }
}

/// Reads and writes [`Settings`].
#[derive(Debug, Clone)]
pub struct SettingsStore {
    path: PathBuf,
    legacy_welcome_marker: PathBuf,
}

impl SettingsStore {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            path: data_dir.join(FILE_NAME),
            legacy_welcome_marker: data_dir.join(LEGACY_WELCOME_FILE),
        }
    }

    /// The stored settings. A missing or unreadable file gives the defaults
    /// (the welcome is shown again rather than blocking the app). Without a
    /// settings file, the old `welcome-seen` marker still counts.
    pub fn load(&self) -> Settings {
        match std::fs::read_to_string(&self.path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_else(|error| {
                tracing::warn!(%error, "settings.json is unreadable, using the defaults");
                Settings::default()
            }),
            Err(_) => Settings {
                welcome_seen: self.legacy_welcome_marker.is_file(),
                ..Settings::default()
            },
        }
    }

    /// Applies `change` to the stored settings and saves them.
    pub fn update(&self, change: impl FnOnce(&mut Settings)) -> io::Result<Settings> {
        let mut settings = self.load();
        change(&mut settings);
        settings.version = CURRENT_VERSION;
        self.save(&settings)?;
        Ok(settings)
    }

    /// Writes a temporary file then renames it, so a crash never leaves a half-written file.
    fn save(&self, settings: &Settings) -> io::Result<()> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let temporary = self.path.with_extension("json.tmp");
        std::fs::write(&temporary, serde_json::to_string_pretty(settings)?)?;
        std::fs::rename(&temporary, &self.path)
    }
}

const CURRENT_VERSION: u32 = 1;

/// File name, in the data folder.
const FILE_NAME: &str = "settings.json";

/// Marker file of earlier versions, replaced by `Settings::welcome_seen`.
const LEGACY_WELCOME_FILE: &str = "welcome-seen";

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("cooldown-settings-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_fresh_folder_gives_the_defaults() {
        let store = SettingsStore::new(&temp_dir("fresh"));
        assert_eq!(store.load(), Settings::default());
        assert!(!store.load().welcome_seen);
    }

    #[test]
    fn an_update_persists_and_creates_the_folder() {
        let dir = temp_dir("update").join("nested");
        let store = SettingsStore::new(&dir);
        store.update(|s| s.welcome_seen = true).unwrap();
        assert!(SettingsStore::new(&dir).load().welcome_seen);
        assert!(!dir.join("settings.json.tmp").exists());
        std::fs::remove_dir_all(dir.parent().unwrap()).unwrap();
    }

    #[test]
    fn a_corrupt_file_gives_the_defaults() {
        let dir = temp_dir("corrupt");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(FILE_NAME), "{not json").unwrap();
        assert_eq!(SettingsStore::new(&dir).load(), Settings::default());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn missing_fields_fall_back_to_their_default() {
        let dir = temp_dir("partial");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(FILE_NAME), r#"{"future_field": 1}"#).unwrap();
        assert_eq!(SettingsStore::new(&dir).load(), Settings::default());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn the_legacy_welcome_marker_is_still_honoured() {
        let dir = temp_dir("legacy");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(LEGACY_WELCOME_FILE), b"").unwrap();
        let store = SettingsStore::new(&dir);
        assert!(store.load().welcome_seen);
        store.update(|_| {}).unwrap();
        assert!(store.load().welcome_seen, "migrated into settings.json");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
