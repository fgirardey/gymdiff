//! The update loop of alpha builds: checks shortly after launch, every hour
//! and on "Check now", installs newer builds for the next launch, and pushes
//! its state to the Settings screen (`docs/Product/Features/Auto-update.md`).

use std::time::Duration;

use tokio::sync::mpsc;

use super::install;
use super::manifest::Manifest;
use super::releases::{self, UpdateChannel};
use super::state::{UpdateError, UpdateState};
use crate::AppWindow;
use crate::build_info::{BUILD, Channel};

/// Handle on the update loop, which runs on its own thread.
pub struct Updater {
    checks: mpsc::UnboundedSender<()>,
}

impl Updater {
    /// Starts the update loop if this build updates itself: alpha builds, or
    /// any build when `GYMDIFF_UPDATE_URL` points at a test release list.
    pub fn start(window: slint::Weak<AppWindow>) -> Option<Self> {
        let feed = feed()?;
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

/// GymDiff's releases, newest first (ADR-0010). Without a token GitHub allows
/// 60 requests an hour, plenty for one check an hour.
const RELEASES_API: &str = "https://api.github.com/repos/fgirardey/gymdiff/releases?per_page=30";
/// Lets the first check wait until the app has settled.
const FIRST_CHECK_DELAY: Duration = Duration::from_secs(5);
const CHECK_INTERVAL: Duration = Duration::from_hours(1);
/// Whole download included: an executable is about 20 MB.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

/// Where this build looks for updates.
struct Feed {
    /// The release list, in the GitHub API's format.
    url: String,
    channel: UpdateChannel,
}

/// Stable releases update themselves once their manifest is signed (ADR-0010).
const STABLE_UPDATES: bool = false;

/// Release builds follow the stable releases, every other build the alpha
/// releases. Alpha builds always update; release builds once
/// [`STABLE_UPDATES`] is on; any build when `GYMDIFF_UPDATE_URL` points at a
/// test release list.
fn feed() -> Option<Feed> {
    let channel = if BUILD.channel == Channel::Release {
        UpdateChannel::Stable
    } else {
        UpdateChannel::Alpha
    };
    let updates_itself = match channel {
        UpdateChannel::Alpha => BUILD.channel == Channel::Alpha,
        UpdateChannel::Stable => STABLE_UPDATES,
    };
    let url = match std::env::var("GYMDIFF_UPDATE_URL") {
        Ok(url) if !url.is_empty() => url,
        _ if updates_itself => RELEASES_API.to_owned(),
        _ => return None,
    };
    Some(Feed { url, channel })
}

async fn run(
    window: slint::Weak<AppWindow>,
    mut requests: mpsc::UnboundedReceiver<()>,
    feed: Feed,
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
    tracing::info!(feed = %feed.url, channel = ?feed.channel, build = BUILD.number, "updates on");
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

/// One check: finds the channel's latest release, reads its manifest,
/// installs it when newer.
async fn check(
    client: &reqwest::Client,
    feed: &Feed,
    installed: &mut u32,
    window: &slint::Weak<AppWindow>,
) -> UpdateState {
    push(window, &UpdateState::Checking);
    let list = match install::download(client, &feed.url).await {
        Ok(body) => body,
        Err(reason) => return failed(reason),
    };
    let Some(releases) = releases::parse(&list) else {
        tracing::warn!(feed = %feed.url, "the update URL does not serve a release list");
        return failed(UpdateError::NotPublished);
    };
    let Some(manifest_url) =
        releases::latest(&releases, feed.channel).and_then(releases::Release::manifest_url)
    else {
        tracing::debug!(channel = ?feed.channel, "no release with a manifest in the channel");
        return failed(UpdateError::NotPublished);
    };
    let body = match install::download(client, manifest_url).await {
        Ok(body) => body,
        Err(reason) => return failed(reason),
    };
    let Some(manifest) = Manifest::parse(&body) else {
        tracing::warn!(manifest_url, "not a manifest");
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
