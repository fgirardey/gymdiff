//! The update loop of alpha builds: checks shortly after launch, every hour
//! and on "Check now", installs newer builds for the next launch, and pushes
//! its state to the Settings screen (`docs/Product/Features/Auto-update.md`).

use std::time::Duration;

use tokio::sync::mpsc;

use super::install;
use super::manifest::Manifest;
use super::state::{UpdateError, UpdateState};
use crate::AppWindow;
use crate::build_info::{BUILD, Channel};

/// Handle on the update loop, which runs on its own thread.
pub struct Updater {
    checks: mpsc::UnboundedSender<()>,
}

impl Updater {
    /// Starts the update loop if this build updates itself: alpha builds, or
    /// any build when `GYMDIFF_UPDATE_URL` points at a test manifest.
    pub fn start(window: slint::Weak<AppWindow>) -> Option<Self> {
        let feed = feed_url()?;
        let (checks, requests) = mpsc::unbounded_channel();
        let spawned = std::thread::Builder::new()
            .name("updater".into())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        tracing::error!(%error, "cannot build the updater runtime");
                        return;
                    }
                };
                runtime.block_on(run(window, requests, feed));
            });
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

/// Where alpha builds look for updates (ADR-0010).
const ALPHA_FEED: &str = "https://github.com/fgirardey/gymdiff/releases/download/alpha/latest.json";
/// Lets the first check wait until the app has settled.
const FIRST_CHECK_DELAY: Duration = Duration::from_secs(5);
const CHECK_INTERVAL: Duration = Duration::from_hours(1);
/// Whole download included: an executable is about 20 MB.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

fn feed_url() -> Option<String> {
    match std::env::var("GYMDIFF_UPDATE_URL") {
        Ok(url) if !url.is_empty() => Some(url),
        _ => (BUILD.channel == Channel::Alpha).then(|| ALPHA_FEED.to_owned()),
    }
}

async fn run(
    window: slint::Weak<AppWindow>,
    mut requests: mpsc::UnboundedReceiver<()>,
    feed: String,
) {
    let client = match reqwest::Client::builder()
        .user_agent(format!("GymDiff/{}", BUILD.full_version()))
        .timeout(REQUEST_TIMEOUT)
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            tracing::error!(%error, "cannot build the update HTTP client");
            return;
        }
    };
    tracing::info!(%feed, build = BUILD.number, "updates on");
    let mut state = UpdateState::Idle;
    push(&window, &state);
    // The newest build on disk: the running one, or one installed since.
    let mut installed = BUILD.number;
    let mut wait = FIRST_CHECK_DELAY;

    loop {
        tokio::select! {
            () = tokio::time::sleep(wait) => {}
            request = requests.recv() => {
                if request.is_none() {
                    return;
                }
            }
        }
        // Clicks queued during the previous check would only repeat it.
        while requests.try_recv().is_ok() {}

        let next = check(&client, &feed, &mut installed, &window).await;
        if next != state {
            log(&next);
        }
        state = next;
        push(&window, &state);
        wait = CHECK_INTERVAL;
    }
}

/// One check: reads the manifest, installs a newer build.
async fn check(
    client: &reqwest::Client,
    feed: &str,
    installed: &mut u32,
    window: &slint::Weak<AppWindow>,
) -> UpdateState {
    push(window, &UpdateState::Checking);
    let body = match install::download(client, feed).await {
        Ok(body) => body,
        Err(reason) => return failed(reason),
    };
    let Some(manifest) = Manifest::parse(&body) else {
        tracing::warn!(%feed, "the update URL does not serve a manifest");
        return failed(UpdateError::NotPublished);
    };
    if !manifest.is_newer_than(*installed, BUILD.target) {
        return UpdateState::UpToDate {
            checked_at: chrono::Local::now().format("%H:%M").to_string(),
        };
    }

    push(
        window,
        &UpdateState::Downloading {
            build: manifest.build,
        },
    );
    match install::install(client, &manifest).await {
        Ok(()) => {
            *installed = manifest.build;
            UpdateState::Ready {
                build: manifest.build,
            }
        }
        Err(reason) => failed(reason),
    }
}

fn failed(reason: UpdateError) -> UpdateState {
    UpdateState::Failed { reason }
}

/// State changes at `INFO`, failures at `WARN` (never `ERROR`: a network
/// failure is not a bug, see `docs/Workflow/Logging.md`).
fn log(state: &UpdateState) {
    match state {
        UpdateState::UpToDate { .. } => tracing::info!(build = BUILD.number, "up to date"),
        UpdateState::Ready { build } => {
            tracing::info!(build, "update installed, starts at the next launch");
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
