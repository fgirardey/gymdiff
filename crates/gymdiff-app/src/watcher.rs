//! Background loop: discovers the League client, polls it, reduces events into
//! a [`CompanionState`] and pushes it to the UI.
//!
//! Polling is a first step; the LCU WebSocket event stream will replace it for
//! client-side data (see `docs/Roadmap.md`).

use std::time::Duration;

use gymdiff_core::{CompanionState, ConnectionStatus, Event};
use gymdiff_lol::{HttpLolClient, Lockfile, LolClient};

use crate::AppWindow;

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const DISCOVERY_INTERVAL: Duration = Duration::from_secs(2);

pub async fn run(window: slint::Weak<AppWindow>) {
    let mut state = CompanionState::default();
    push(&window, &state, "");

    loop {
        let lockfile = match Lockfile::discover() {
            Ok(lockfile) => lockfile,
            Err(error) => {
                tracing::debug!(%error, "League client not found");
                tokio::time::sleep(DISCOVERY_INTERVAL).await;
                continue;
            }
        };
        let client = match HttpLolClient::new(lockfile) {
            Ok(client) => client,
            Err(error) => {
                tracing::error!(%error, "cannot build HTTP client");
                tokio::time::sleep(DISCOVERY_INTERVAL).await;
                continue;
            }
        };
        let source = describe_source(client.lockfile());
        tracing::info!(%source, "League client found");

        state = poll_until_disconnected(&client, state, &window, &source).await;
        state = state.apply(Event::ClientDisconnected);
        push(&window, &state, "");
    }
}

async fn poll_until_disconnected(
    client: &impl LolClient,
    mut state: CompanionState,
    window: &slint::Weak<AppWindow>,
    source: &str,
) -> CompanionState {
    let mut consecutive_errors = 0;
    loop {
        match client.gameflow_phase().await {
            Ok(phase) => {
                consecutive_errors = 0;
                if state.connection != ConnectionStatus::Connected {
                    state = state.apply(Event::ClientConnected);
                }
                if phase != state.phase {
                    tracing::info!(phase = phase.label(), "phase changed");
                    state = state.apply(Event::PhaseChanged(phase));
                }
            }
            Err(error) => {
                consecutive_errors += 1;
                tracing::warn!(%error, consecutive_errors, "LCU request failed");
                if consecutive_errors >= 3 {
                    // The client was probably closed (stale lockfile): rediscover.
                    return state;
                }
                state = state.apply(Event::ClientError(error.to_string()));
            }
        }

        if state.phase.is_in_game() {
            match client.game_stats().await {
                Ok(Some(live)) => state = state.apply(Event::GameTimeUpdated(live.game_time)),
                Ok(None) => {}
                Err(error) => tracing::warn!(%error, "Live Client Data request failed"),
            }
        }

        push(window, &state, source);
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

fn describe_source(lockfile: &Lockfile) -> String {
    if lockfile.protocol == "http" {
        format!("Simulator on port {}", lockfile.port)
    } else {
        format!("League client on port {}", lockfile.port)
    }
}

fn push(window: &slint::Weak<AppWindow>, state: &CompanionState, source: &str) {
    let connected = state.connection == ConnectionStatus::Connected;
    let connection_label = match &state.connection {
        ConnectionStatus::Disconnected => "Looking for the League client…".to_owned(),
        ConnectionStatus::Connected => "Connected".to_owned(),
        ConnectionStatus::Error(message) => format!("Error: {message}"),
    };
    let phase_label = state.phase.label().to_owned();
    let game_time = state.game_time_label();
    let source = source.to_owned();

    let result = window.upgrade_in_event_loop(move |window| {
        window.set_connected(connected);
        window.set_connection_label(connection_label.into());
        window.set_phase_label(phase_label.into());
        window.set_game_time(game_time.into());
        window.set_data_source(source.into());
    });
    if result.is_err() {
        tracing::debug!("UI event loop is gone");
    }
}
