//! # cooldown-recorder
//!
//! Records a real League session into a simulator scenario, so it can be
//! replayed on macOS with `just sim --scenario <file>`.
//!
//! It is **read-only**: it only issues `GET` requests to the two local APIs,
//! at a low rate, exactly like the app does. It never touches the game
//! process. See `docs/Workflow/Recorder.md` and `docs/Compliance/Riot policies.md`.
//!
//! Nothing personal reaches the disk: every file written is anonymised first.

mod anonymize;
mod recording;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Context;
use clap::Parser;
use cooldown_lol::{Api, HttpLolClient, Lockfile};
use tracing_subscriber::EnvFilter;

use crate::recording::{Observation, Recording};

/// Endpoints recorded by default. The app and the planned features read these.
const DEFAULT_LCU: &[&str] = &[
    "/lol-gameflow/v1/gameflow-phase",
    "/lol-champ-select/v1/session",
    "/lol-end-of-game/v1/eog-stats-block",
];
const DEFAULT_LIVE: &[&str] = &["/liveclientdata/gamestats", "/liveclientdata/allgamedata"];

#[derive(Debug, Parser)]
#[command(about = "Record a real League session into a simulator scenario (read-only).")]
struct Args {
    /// Scenario name; written to `<out-dir>/recorded-<name>/scenario.json`
    /// (e.g. `2026-11-07-ranked-support`).
    name: String,
    /// Directory holding the scenarios.
    #[arg(long, default_value = "fixtures/scenarios")]
    out_dir: PathBuf,
    /// Seconds between two polls. Keep it gentle (minimum 1).
    #[arg(long, default_value_t = 2.0)]
    interval: f64,
    /// Extra LCU endpoint to record (GET only), e.g. `/lol-lobby/v2/lobby`.
    #[arg(long = "lcu")]
    extra_lcu: Vec<String>,
    /// Extra Live Client Data endpoint to record (GET only).
    #[arg(long = "live")]
    extra_live: Vec<String>,
    /// Lockfile location; defaults to the usual discovery.
    #[arg(long, env = "COOLDOWN_LOCKFILE")]
    lockfile: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    let interval = Duration::from_secs_f64(args.interval.max(1.0));
    let target = args
        .out_dir
        .join(format!("recorded-{}", args.name))
        .join("scenario.json");

    let endpoints = endpoints(&args);
    let client = connect(args.lockfile.as_deref())?;
    let mut recording = Recording::default();
    let started = Instant::now();

    tracing::info!(file = %target.display(), ?interval, "recording; press Ctrl+C to stop");
    let mut ticker = tokio::time::interval(interval);
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let at = started.elapsed().as_secs_f64();
                let observations = poll(&client, &endpoints).await;
                if recording.observe(at, observations) {
                    // Saved on every change so a crash or a closed terminal loses nothing.
                    save(&recording, &target)?;
                }
            }
            _ = tokio::signal::ctrl_c() => break,
        }
    }
    save(&recording, &target)?;
    tracing::info!(steps = recording.step_count(), file = %target.display(), "saved");
    Ok(())
}

fn endpoints(args: &Args) -> Vec<(Api, String)> {
    let lcu = DEFAULT_LCU
        .iter()
        .map(|path| (*path).to_owned())
        .chain(args.extra_lcu.clone());
    let live = DEFAULT_LIVE
        .iter()
        .map(|path| (*path).to_owned())
        .chain(args.extra_live.clone());
    lcu.map(|path| (Api::Lcu, path))
        .chain(live.map(|path| (Api::Live, path)))
        .collect()
}

/// The client may not be running yet, in which case the lockfile does not exist.
fn connect(lockfile: Option<&std::path::Path>) -> anyhow::Result<HttpLolClient> {
    let lockfile = match lockfile {
        Some(path) => Lockfile::read(path),
        None => Lockfile::discover(),
    }
    .context("start the League client first, or pass --lockfile")?;
    Ok(HttpLolClient::new(lockfile)?)
}

async fn poll(client: &HttpLolClient, endpoints: &[(Api, String)]) -> Vec<Observation> {
    let mut observations = Vec::with_capacity(endpoints.len());
    for (api, path) in endpoints {
        let body = match client.fetch_json(*api, path).await {
            Ok(body) => body,
            Err(error) => {
                // Unexpected status or payload: treat as unavailable, keep recording.
                tracing::warn!(%path, %error, "endpoint not readable");
                None
            }
        };
        observations.push(Observation {
            api: *api,
            path: path.clone(),
            body,
        });
    }
    observations
}

fn save(recording: &Recording, target: &std::path::Path) -> anyhow::Result<()> {
    let mut scenario = recording.to_scenario();
    anonymize::anonymize(&mut scenario);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut json = serde_json::to_string_pretty(&scenario)?;
    json.push('\n');
    std::fs::write(target, json).with_context(|| format!("writing {}", target.display()))
}
