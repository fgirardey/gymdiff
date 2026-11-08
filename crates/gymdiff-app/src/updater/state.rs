//! What the Settings screen shows about updates
//! (`docs/Product/Features/Auto-update.md`, Behaviour).

/// State of the updater, as shown under "Alpha channel" in Settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateState {
    /// No check yet since launch.
    Idle,
    Checking,
    /// No newer package; `checked_at` is the local time, `14:05`.
    UpToDate {
        checked_at: String,
    },
    Downloading {
        target: UpdateTarget,
    },
    /// Downloaded; Velopack applies it at the next launch.
    Ready {
        target: UpdateTarget,
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
            Self::Downloading { target } => format!("Downloading {}…", target.lower()),
            Self::Ready { target } => format!(
                "{} is ready. It starts the next time you open GymDiff.",
                target.capitalized()
            ),
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

/// The package being installed, as the player reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateTarget {
    /// An alpha package, `0.1.0-alpha.58` → build 58.
    Build(u32),
    /// Any other package, by its version.
    Version(String),
}

impl UpdateTarget {
    /// From a Velopack package version.
    pub fn from_version(version: &str) -> Self {
        version
            .split_once("-alpha.")
            .and_then(|(_, build)| build.parse().ok())
            .map_or_else(|| Self::Version(version.to_owned()), Self::Build)
    }

    fn lower(&self) -> String {
        match self {
            Self::Build(build) => format!("build {build}"),
            Self::Version(version) => format!("version {version}"),
        }
    }

    fn capitalized(&self) -> String {
        match self {
            Self::Build(build) => format!("Build {build}"),
            Self::Version(version) => format!("Version {version}"),
        }
    }
}

/// Why an update failed, in words the player understands. The technical
/// detail goes to the log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateError {
    /// Network error, GitHub down or limiting requests.
    NoConnection,
    /// No package on the channel's feed yet.
    NotPublished,
    /// Anything else (corrupted download, disk error…).
    Other,
}

impl UpdateError {
    pub fn reason(self) -> &'static str {
        match self {
            Self::NoConnection => "no connection",
            Self::NotPublished => "no alpha build published yet",
            Self::Other => "update error",
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
        let build = UpdateTarget::Build(58);
        let downloading = UpdateState::Downloading {
            target: build.clone(),
        };
        assert_eq!(downloading.text(), "Downloading build 58…");
        assert_eq!(
            UpdateState::Ready { target: build }.text(),
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
    fn reads_the_build_from_alpha_versions() {
        assert_eq!(
            UpdateTarget::from_version("0.1.0-alpha.58"),
            UpdateTarget::Build(58)
        );
        let stable = UpdateTarget::from_version("0.2.0");
        assert_eq!(stable, UpdateTarget::Version("0.2.0".to_owned()));
        assert_eq!(
            UpdateState::Ready { target: stable }.text(),
            "Version 0.2.0 is ready. It starts the next time you open GymDiff."
        );
    }

    #[test]
    fn only_checking_and_downloading_are_busy() {
        assert!(UpdateState::Checking.busy());
        let downloading = UpdateState::Downloading {
            target: UpdateTarget::Build(1),
        };
        assert!(downloading.busy());
        assert!(!UpdateState::Idle.busy());
    }
}
