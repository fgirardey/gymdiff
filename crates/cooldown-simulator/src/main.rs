//! # cooldown-simulator
//!
//! A fake League of Legends client for development on macOS (or anywhere).
//!
//! It replays a scenario (see `fixtures/scenarios/`) and serves:
//! - a fake **LCU API** over HTTP, advertised through a fake `lockfile`,
//! - a fake **Live Client Data API** over HTTP.
//!
//! The app discovers it exactly like the real client, through
//! `COOLDOWN_LOCKFILE` and `COOLDOWN_LIVE_CLIENT_URL`.
//!
//! Not simulated yet: HTTPS with Riot's certificate and the LCU WebSocket
//! event stream (see `docs/Roadmap.md`).

mod base64;
mod scenario;
mod server;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Context;
use clap::Parser;
use cooldown_lol::Lockfile;
use tracing_subscriber::EnvFilter;

use crate::scenario::Scenario;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    let scenario = Scenario::load(&args.scenario)?;
    tracing::info!(
        description = %scenario.description,
        synthetic = scenario.synthetic,
        steps = scenario.steps.len(),
        "scenario loaded"
    );
    for step in &scenario.steps {
        if let Some(note) = &step.note {
            tracing::info!("  t={:>6.1}s  {note}", step.at);
        }
    }

    let lockfile = Lockfile {
        process_name: "LeagueClient".into(),
        pid: std::process::id(),
        port: args.lcu_port,
        password: format!("cooldown-sim-{}", std::process::id()),
        protocol: "http".into(),
    };
    write_lockfile(&args.lockfile, &lockfile)?;

    let simulator = Arc::new(server::Simulator {
        scenario,
        started: Instant::now(),
        speed: args.speed,
        looping: args.looping,
        password: lockfile.password.clone(),
    });

    let lcu = server::serve(server::Api::Lcu, args.lcu_port, simulator.clone());
    let live = server::serve(server::Api::Live, args.live_port, simulator);

    let lockfile_path = std::path::absolute(&args.lockfile)?;
    println!();
    println!("Simulator running. Point the app at it with:");
    println!("  export COOLDOWN_LOCKFILE=\"{}\"", lockfile_path.display());
    println!(
        "  export COOLDOWN_LIVE_CLIENT_URL=\"http://127.0.0.1:{}\"",
        args.live_port
    );
    println!("(`just dev` does this for you.) Press Ctrl+C to stop.");
    println!();

    tokio::select! {
        result = lcu => result?,
        result = live => result?,
        _ = tokio::signal::ctrl_c() => tracing::info!("stopping"),
    }

    // Like the real client, remove the lockfile on exit.
    let _ = std::fs::remove_file(&args.lockfile);
    Ok(())
}

#[derive(Debug, Parser)]
#[command(version, about = "Fake League of Legends client replaying a scenario")]
struct Args {
    /// Scenario file to replay.
    #[arg(
        long,
        env = "COOLDOWN_SCENARIO",
        default_value = "fixtures/scenarios/synthetic-full-game/scenario.json"
    )]
    scenario: PathBuf,

    /// Playback speed (2.0 = twice as fast).
    #[arg(long, default_value_t = 1.0)]
    speed: f64,

    /// Restart the scenario when it ends.
    #[arg(long = "loop")]
    looping: bool,

    /// Port of the fake LCU API.
    #[arg(long, default_value_t = 51_337)]
    lcu_port: u16,

    /// Port of the fake Live Client Data API (the real one uses 2999).
    #[arg(long, default_value_t = 12_999)]
    live_port: u16,

    /// Where to write the fake lockfile.
    #[arg(long, default_value = "target/simulator/lockfile")]
    lockfile: PathBuf,
}

fn write_lockfile(path: &PathBuf, lockfile: &Lockfile) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, lockfile.to_line())
        .with_context(|| format!("writing lockfile {}", path.display()))
}
