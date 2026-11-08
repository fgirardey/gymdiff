//! The update loop of installed builds: checks shortly after launch, every
//! hour and on "Check now", downloads newer packages for Velopack to apply at
//! the next launch, and pushes its state to the Settings screen
//! (`docs/Product/Features/Auto-update.md`, ADR-0011).

use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

#[cfg(windows)]
use gymdiff_shortcuts::Shortcut;
use velopack::sources::GithubSource;
use velopack::{UpdateCheck, UpdateManager, VelopackApp};

use super::state::{UpdateError, UpdateState, UpdateTarget};
use crate::AppWindow;
use crate::build_info::{BUILD, Channel};

/// Handle on the update loop, which runs on its own thread.
pub struct Updater {
    checks: mpsc::Sender<()>,
}

impl Updater {
    /// Starts the update loop if this build updates itself: installed by
    /// Velopack, on a channel that updates (alpha; stable once
    /// [`STABLE_UPDATES`] is on).
    pub fn start(window: slint::Weak<AppWindow>) -> Option<Self> {
        let prerelease = match BUILD.channel {
            Channel::Alpha => true,
            Channel::Release if STABLE_UPDATES => false,
            _ => return None,
        };
        let source = GithubSource::new(REPOSITORY, None, prerelease);
        let manager = match UpdateManager::new(source, None, None) {
            Ok(manager) => manager,
            Err(error) => {
                tracing::info!(%error, "not installed by the installer: updates off");
                return None;
            }
        };
        let (checks, requests) = mpsc::channel();
        let spawned = std::thread::Builder::new()
            .name("updater".into())
            .spawn(move || run(&window, &manager, &requests));
        if let Err(error) = spawned {
            tracing::error!(%error, "cannot start the updater thread");
            return None;
        }
        Some(Self { checks })
    }

    /// "Check now": checks right away, unless a check is already running.
    pub fn check_now(&self) {
        // The loop ends only with the app: a closed channel needs no handling.
        let _ = self.checks.send(());
    }
}

/// Velopack's install, update and uninstall hooks, and a downloaded update
/// applied at startup (it restarts GymDiff). Call it early in `main`, before
/// any window or log file; it returns at once when GymDiff runs normally.
/// Before uninstalling, removes the shortcuts the installer or Settings made
/// (ADR-0012): Velopack only removes its own.
pub fn run_installer_hooks() {
    let app = VelopackApp::build();
    // Velopack's install and uninstall hooks exist on Windows only.
    #[cfg(windows)]
    let app = app.on_before_uninstall_fast_callback(|_version| {
        for shortcut in Shortcut::ALL {
            // Nothing to log to yet: uninstalling must go on anyway.
            let _ = shortcut.remove();
        }
    });
    let mut app = app;
    app.run();
}

/// True when GymDiff was installed by the installer (Velopack), not run from
/// the portable zip or a local build.
pub fn is_installed() -> bool {
    UpdateManager::new(GithubSource::new(REPOSITORY, None, false), None, None).is_ok()
}

/// Where the packages are published (GitHub Releases).
const REPOSITORY: &str = "https://github.com/fgirardey/gymdiff";
/// Stable installs update themselves once updates are signed (ADR-0011).
const STABLE_UPDATES: bool = false;
/// Lets the first check wait until the app has settled.
const FIRST_CHECK_DELAY: Duration = Duration::from_secs(5);
const CHECK_INTERVAL: Duration = Duration::from_hours(1);

fn run(window: &slint::Weak<AppWindow>, manager: &UpdateManager, requests: &mpsc::Receiver<()>) {
    tracing::info!(
        installed = %manager.get_current_version_as_string(),
        "updates on"
    );
    let mut state = UpdateState::Idle;
    push(window, &state);
    let mut wait = FIRST_CHECK_DELAY;

    loop {
        match requests.recv_timeout(wait) {
            Ok(()) | Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        // Clicks queued during the previous check would only repeat it.
        while requests.try_recv().is_ok() {}

        let next = check(manager, window);
        if next != state {
            log(&next);
        }
        state = next;
        push(window, &state);
        wait = CHECK_INTERVAL;
    }
}

/// One check: asks the channel's feed, downloads a newer package.
fn check(manager: &UpdateManager, window: &slint::Weak<AppWindow>) -> UpdateState {
    push(window, &UpdateState::Checking);
    let update = match manager.check_for_updates() {
        Ok(UpdateCheck::UpdateAvailable(update)) => update,
        Ok(UpdateCheck::NoUpdateAvailable) => {
            return UpdateState::UpToDate {
                checked_at: chrono::Local::now().format("%H:%M").to_string(),
            };
        }
        Ok(UpdateCheck::RemoteIsEmpty) => return failed(UpdateError::NotPublished),
        Err(error) => return failed(classify(&error)),
    };

    let target = UpdateTarget::from_version(&update.TargetFullRelease.Version);
    push(
        window,
        &UpdateState::Downloading {
            target: target.clone(),
        },
    );
    match manager.download_updates(&update, None) {
        Ok(()) => UpdateState::Ready { target },
        Err(error) => failed(classify(&error)),
    }
}

/// Logs the detail, keeps a reason the player understands.
fn classify(error: &velopack::Error) -> UpdateError {
    tracing::warn!(%error, "update check or download failed");
    match error {
        velopack::Error::Network(_) => UpdateError::NoConnection,
        _ => UpdateError::Other,
    }
}

fn failed(reason: UpdateError) -> UpdateState {
    UpdateState::Failed { reason }
}

/// State changes at `INFO`, failures at `WARN` (never `ERROR`: a network
/// failure is not a bug, see `docs/Workflow/Logging.md`).
fn log(state: &UpdateState) {
    match state {
        UpdateState::UpToDate { .. } => tracing::info!("up to date"),
        UpdateState::Ready { target } => {
            tracing::info!(?target, "update downloaded, applied at the next launch");
        }
        UpdateState::Failed { reason } => {
            tracing::warn!(reason = reason.reason(), "update failed");
        }
        UpdateState::Idle | UpdateState::Checking | UpdateState::Downloading { .. } => {}
    }
}

fn push(window: &slint::Weak<AppWindow>, state: &UpdateState) {
    let text = state.text();
    let busy = state.busy();
    let result = window.upgrade_in_event_loop(move |window| {
        window.set_update_status(text.into());
        window.set_update_busy(busy);
    });
    if let Err(error) = result {
        tracing::debug!(%error, "cannot push the update state (window closed?)");
    }
}
