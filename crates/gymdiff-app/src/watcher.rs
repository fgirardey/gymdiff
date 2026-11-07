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
    let status_label = status_label(state);
    if !source.is_empty() {
        tracing::trace!(%source, status = %status_label, "pushing state to the UI");
    }

    let result = window.upgrade_in_event_loop(move |window| {
        window.set_connected(connected);
        window.set_status_label(status_label.into());
    });
    if result.is_err() {
        tracing::debug!("UI event loop is gone");
    }
}

/// Text next to the connection dot in the top bar.
fn status_label(state: &CompanionState) -> String {
    match &state.connection {
        ConnectionStatus::Disconnected => "Looking for the League client…".to_owned(),
        ConnectionStatus::Connected => format!("Connected · {}", state.phase.label()),
        ConnectionStatus::Error(message) => format!("Error: {message}"),
    }
}

#[cfg(test)]
mod tests {
    use gymdiff_core::GamePhase;

    use super::*;

    #[test]
    fn status_label_shows_the_phase_once_connected() {
        let state = CompanionState::default();
        assert_eq!(status_label(&state), "Looking for the League client…");

        let state = state
            .apply(Event::ClientConnected)
            .apply(Event::PhaseChanged(GamePhase::Lobby));
        assert_eq!(status_label(&state), "Connected · Lobby");
    }
}
