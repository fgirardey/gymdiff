//! What the Settings screen shows about updates
//! (`docs/Product/Features/Auto-update.md`, Behaviour).

/// State of the updater, as shown under "Alpha channel" in Settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateState {
    /// No check yet since launch.
    Idle,
    Checking,
    /// No newer build; `checked_at` is the local time, `14:05`.
    UpToDate {
        checked_at: String,
    },
    Downloading {
        build: u32,
    },
    /// The build is installed and starts at the next launch.
    Ready {
        build: u32,
    },
    Failed {
        reason: UpdateError,
    },
}

impl UpdateState {
    /// The status line under "Alpha channel".
    pub fn text(&self) -> String {
        match self {
            Self::Idle => "Checks for updates every hour.".to_owned(),
            Self::Checking => "Checking for updates…".to_owned(),
            Self::UpToDate { checked_at } => format!("Up to date. Last checked at {checked_at}."),
            Self::Downloading { build } => format!("Downloading build {build}…"),
            Self::Ready { build } => {
                format!("Build {build} is ready. It starts the next time you open GymDiff.")
            }
            Self::Failed { reason } => {
                format!("Couldn't update: {}. Next try in an hour.", reason.reason())
            }
        }
    }

    /// True while a check or a download runs: "Check now" is disabled.
    pub fn busy(&self) -> bool {
        matches!(self, Self::Checking | Self::Downloading { .. })
    }
}

/// Why an update failed, in words the player understands. The technical
/// detail goes to the log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateError {
    /// The request did not complete: offline, DNS, timeout, server error.
    NoConnection,
    /// No alpha build is published yet (no release with a manifest).
    NotPublished,
    /// GitHub refuses more requests for now (60 an hour without a token).
    RateLimited,
    /// The download does not match the manifest's size or checksum.
    Incomplete,
    /// The executable's folder is not writable (e.g. Program Files).
    CantWrite,
}

impl UpdateError {
    pub fn reason(self) -> &'static str {
        match self {
            Self::NoConnection => "no connection",
            Self::NotPublished => "no alpha build published yet",
            Self::RateLimited => "GitHub is limiting requests",
            Self::Incomplete => "download incomplete",
            Self::CantWrite => "can't write next to GymDiff",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texts_match_the_spec() {
        assert_eq!(UpdateState::Idle.text(), "Checks for updates every hour.");
        assert_eq!(UpdateState::Checking.text(), "Checking for updates…");
        let up_to_date = UpdateState::UpToDate {
            checked_at: "14:05".to_owned(),
        };
        assert_eq!(up_to_date.text(), "Up to date. Last checked at 14:05.");
        assert_eq!(
            UpdateState::Downloading { build: 58 }.text(),
            "Downloading build 58…"
        );
        assert_eq!(
            UpdateState::Ready { build: 58 }.text(),
            "Build 58 is ready. It starts the next time you open GymDiff."
        );
        let failed = UpdateState::Failed {
            reason: UpdateError::NoConnection,
        };
        assert_eq!(
            failed.text(),
            "Couldn't update: no connection. Next try in an hour."
        );
    }

    #[test]
    fn only_checking_and_downloading_are_busy() {
        assert!(UpdateState::Checking.busy());
        assert!(UpdateState::Downloading { build: 1 }.busy());
        assert!(!UpdateState::Idle.busy());
        assert!(!UpdateState::Ready { build: 1 }.busy());
    }
}
