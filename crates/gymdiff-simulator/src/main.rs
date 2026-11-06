//! # gymdiff-simulator
//!
//! A fake League of Legends client for development on macOS (or anywhere).
//!
//! It replays a scenario (see `fixtures/scenarios/`) and serves:
//! - a fake **LCU API** over HTTP, advertised through a fake `lockfile`,
//! - a fake **Live Client Data API** over HTTP.
//!
//! The app discovers it exactly like the real client, through
//! `GYMDIFF_LOCKFILE` and `GYMDIFF_LIVE_CLIENT_URL`.
//!
//! Not simulated yet: HTTPS with Riot's certificate and the LCU WebSocket
//! event stream (see `docs/Roadmap.md`).

mod scenario;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Context;
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use clap::Parser;
use gymdiff_lol::Lockfile;
use tracing_subscriber::EnvFilter;

use crate::scenario::Scenario;

#[derive(Debug, Parser)]
#[command(version, about = "Fake League of Legends client replaying a scenario")]
struct Args {
    /// Scenario file to replay.
    #[arg(
        long,
        env = "GYMDIFF_SCENARIO",
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

struct Simulator {
    scenario: Scenario,
    started: Instant,
    speed: f64,
    looping: bool,
    password: String,
}

impl Simulator {
    fn scenario_time(&self) -> f64 {
        let t = self.started.elapsed().as_secs_f64() * self.speed;
        let duration = self.scenario.duration();
        if self.looping && duration > 0.0 {
            // Keep the last step visible for a while before restarting.
            t % (duration + 10.0)
        } else {
            t
        }
    }
}

#[derive(Clone, Copy)]
enum Api {
    Lcu,
    Live,
}

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
        password: format!("gymdiff-sim-{}", std::process::id()),
        protocol: "http".into(),
    };
    write_lockfile(&args.lockfile, &lockfile)?;

    let simulator = Arc::new(Simulator {
        scenario,
        started: Instant::now(),
        speed: args.speed,
        looping: args.looping,
        password: lockfile.password.clone(),
    });

    let lcu = serve(Api::Lcu, args.lcu_port, simulator.clone());
    let live = serve(Api::Live, args.live_port, simulator);

    let lockfile_path = std::path::absolute(&args.lockfile)?;
    println!();
    println!("Simulator running. Point the app at it with:");
    println!("  export GYMDIFF_LOCKFILE=\"{}\"", lockfile_path.display());
    println!(
        "  export GYMDIFF_LIVE_CLIENT_URL=\"http://127.0.0.1:{}\"",
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

fn write_lockfile(path: &PathBuf, lockfile: &Lockfile) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, lockfile.to_line())
        .with_context(|| format!("writing lockfile {}", path.display()))
}

async fn serve(api: Api, port: u16, simulator: Arc<Simulator>) -> anyhow::Result<()> {
    let app = Router::new()
        .fallback(move |state, headers, uri| handle(api, state, headers, uri))
        .with_state(simulator);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding {addr}"))?;
    tracing::info!(
        api = match api {
            Api::Lcu => "LCU",
            Api::Live => "Live Client Data",
        },
        %addr,
        "listening"
    );
    axum::serve(listener, app).await?;
    Ok(())
}

// axum fallback handlers must be async even without awaits.
#[allow(clippy::unused_async)]
async fn handle(
    api: Api,
    State(simulator): State<Arc<Simulator>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    if matches!(api, Api::Lcu) && !is_authorized(&headers, &simulator.password) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let snapshot = simulator.scenario.snapshot_at(simulator.scenario_time());
    let endpoints = match api {
        Api::Lcu => &snapshot.lcu,
        Api::Live => &snapshot.live,
    };
    match endpoints.get(uri.path()) {
        Some(body) => (
            [(header::CONTENT_TYPE, "application/json")],
            body.to_string(),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Checks HTTP Basic auth `riot:<password>`, like the real LCU API.
fn is_authorized(headers: &HeaderMap, password: &str) -> bool {
    let expected = format!(
        "Basic {}",
        base64_encode(format!("riot:{password}").as_bytes())
    );
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == expected)
}

/// Minimal standard base64 encoder (avoids a dependency for one call).
fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(TABLE[(n >> (18 - 6 * i)) as usize & 63] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::base64_encode;

    #[test]
    fn base64_matches_rfc4648_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"riot:pw"), "cmlvdDpwdw==");
    }
}
