//! Background loop: discovers the League client, polls it, reduces events into
//! a [`CompanionState`] and pushes it to the UI.
//!
//! Polling is a first step; the LCU WebSocket event stream will replace it for
//! client-side data (see `docs/Roadmap.md`).

use std::time::Duration;

use cooldown_core::{CompanionState, ConnectionStatus, Event};
use cooldown_lol::{HttpLolClient, Lockfile, LolClient};

use crate::AppWindow;
use crate::history_import;
use crate::history_screen::HistoryScreen;

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const DISCOVERY_INTERVAL: Duration = Duration::from_secs(2);
/// Failed LCU requests in a row before the client is considered gone.
const MAX_CONSECUTIVE_ERRORS: u32 = 3;

/// Logs follow `docs/Workflow/Logging.md`: state changes at `INFO`, each poll
/// at `DEBUG`/`TRACE`. A missing or closing client is normal, never a warning.
pub async fn run(window: slint::Weak<AppWindow>, history: Option<HistoryScreen>) {
    let mut state = CompanionState::default();
    let mut http_error_logged = false;
    push(&window, &state, "");
    // What was stored last time, before the client even answers.
    if let Some(history) = history.clone() {
        tokio::spawn(async move { history.show_last_player().await });
    }

    loop {
        let lockfile = match Lockfile::discover() {
            Ok(lockfile) => lockfile,
            Err(error) => {
                tracing::trace!(%error, "League client not found");
                tokio::time::sleep(DISCOVERY_INTERVAL).await;
                continue;
            }
        };
        let client = match HttpLolClient::new(lockfile) {
            Ok(client) => client,
            Err(error) => {
                // A bug (TLS setup), not a game state: report it once, then retry quietly.
                if http_error_logged {
                    tracing::debug!(%error, "cannot build HTTP client");
                } else {
                    tracing::error!(%error, "cannot build HTTP client");
                    http_error_logged = true;
                }
                tokio::time::sleep(DISCOVERY_INTERVAL).await;
                continue;
            }
        };
        let source = describe_source(client.lockfile());
        tracing::debug!(%source, "lockfile found");

        state = poll_until_disconnected(&client, state, &window, &source, history.as_ref()).await;
        state = state.apply(Event::ClientDisconnected);
        push(&window, &state, "");
    }
}

async fn poll_until_disconnected(
    client: &(impl LolClient + Clone + 'static),
    mut state: CompanionState,
    window: &slint::Weak<AppWindow>,
    source: &str,
    history: Option<&HistoryScreen>,
) -> CompanionState {
    let mut consecutive_errors = 0;
    // Whether the client answered at least once: until then, failures only
    // mean a stale lockfile or a client still starting.
    let mut answered = false;
    let mut payload_warned = false;
    // Dropped (so stopped) when the client goes away.
    let mut _import: Option<ImportTask> = None;
    loop {
        match client.gameflow_phase().await {
            Ok(phase) => {
                consecutive_errors = 0;
                if !answered {
                    answered = true;
                    tracing::info!(%source, "connected to the League client");
                    if let Some(history) = history {
                        _import = Some(spawn_past_games_import(client.clone(), history.clone()));
                    }
                }
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
                if error.is_unexpected_payload() && !payload_warned {
                    tracing::warn!(%error, "unexpected LCU payload");
                    payload_warned = true;
                } else if answered {
                    tracing::debug!(%error, consecutive_errors, "LCU request failed");
                } else {
                    tracing::trace!(%error, consecutive_errors, "LCU request failed");
                }
                if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                    if answered {
                        tracing::info!("League client lost");
                    }
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
                Err(error) if error.is_unexpected_payload() && !payload_warned => {
                    tracing::warn!(%error, "unexpected Live Client Data payload");
                    payload_warned = true;
                }
                Err(error) => tracing::debug!(%error, "Live Client Data request failed"),
            }
        }

        push(window, &state, source);
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

/// Imports the games the client already knows, without holding up the poll. It retries until the
/// player is logged in, and stops with the connection (when the returned task is dropped).
fn spawn_past_games_import(client: impl LolClient + 'static, history: HistoryScreen) -> ImportTask {
    ImportTask(tokio::spawn(async move {
        let imported = history_import::import_when_ready(&client, history.store()).await;
        tracing::info!(
            imported = imported.stored,
            "past games imported from the client"
        );
        history.show(&imported.puuid).await;
    }))
}

/// A running import, aborted when dropped.
struct ImportTask(tokio::task::JoinHandle<()>);

impl Drop for ImportTask {
    fn drop(&mut self) {
        self.0.abort();
    }
}

fn describe_source(lockfile: &Lockfile) -> String {
    format!("League client on port {}", lockfile.port)
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
    use cooldown_core::GamePhase;

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
