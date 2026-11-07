//! Local settings stored in the app's data folder ([`crate::platform::data_dir`]).
//!
//! For now only the "welcome seen" flag (see `docs/Product/Features/First launch.md`).

use std::io;
use std::path::{Path, PathBuf};

/// Whether the player has already gone through the welcome screen.
///
/// Stored as an empty marker file. Any error reading it counts as "not seen",
/// so the welcome is shown again rather than blocking the app.
#[derive(Debug, Clone)]
pub struct WelcomeFlag {
    path: PathBuf,
}

impl WelcomeFlag {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            path: data_dir.join(WELCOME_SEEN_FILE),
        }
    }

    pub fn is_seen(&self) -> bool {
        self.path.is_file()
    }

    pub fn mark_seen(&self) -> io::Result<()> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&self.path, b"")
    }
}

/// Marker file name, in the data folder.
const WELCOME_SEEN_FILE: &str = "welcome-seen";

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("gymdiff-settings-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn welcome_is_not_seen_in_a_fresh_folder() {
        let dir = temp_dir("fresh");
        assert!(!WelcomeFlag::new(&dir).is_seen());
    }

    #[test]
    fn marking_the_welcome_seen_persists_and_creates_the_folder() {
        let dir = temp_dir("mark").join("nested");
        WelcomeFlag::new(&dir).mark_seen().unwrap();
        assert!(WelcomeFlag::new(&dir).is_seen());
        std::fs::remove_dir_all(dir.parent().unwrap()).unwrap();
    }
}
