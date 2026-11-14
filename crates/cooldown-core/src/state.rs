//! The companion's application state and the events that change it.
//!
//! The state is a plain value updated by a pure reducer ([`CompanionState::apply`]).
//! Data sources (real client, simulator, fixtures) only produce [`Event`]s;
//! the UI only renders a [`CompanionState`].

use crate::{Event, GamePhase};

/// Everything the UI needs to render.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompanionState {
    pub connection: ConnectionStatus,
    pub phase: GamePhase,
    /// In-game clock in seconds. `None` outside of a game.
    pub game_time: Option<f64>,
}

impl CompanionState {
    /// Pure reducer: returns the next state for a given event.
    #[must_use]
    pub fn apply(mut self, event: Event) -> Self {
        match event {
            Event::ClientConnected => self.connection = ConnectionStatus::Connected,
            Event::ClientDisconnected => {
                self = Self::default();
            }
            Event::ClientError(message) => self.connection = ConnectionStatus::Error(message),
            Event::PhaseChanged(phase) => {
                if !phase.is_in_game() {
                    self.game_time = None;
                }
                self.phase = phase;
            }
            Event::GameTimeUpdated(seconds) => {
                if self.phase.is_in_game() {
                    self.game_time = Some(seconds);
                }
            }
        }
        self
    }

    /// Game clock formatted as `mm:ss`, or `--:--` outside of a game.
    #[must_use]
    pub fn game_time_label(&self) -> String {
        match self.game_time {
            Some(seconds) if seconds.is_finite() && seconds >= 0.0 => {
                // Truncation is intended: we display whole seconds.
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let total = seconds as u64;
                format!("{:02}:{:02}", total / 60, total % 60)
            }
            _ => "--:--".to_owned(),
        }
    }
}

/// Whether cooldown.lol can currently talk to the League client.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    /// No League client found (not running, or lockfile not found).
    #[default]
    Disconnected,
    /// League client found and responding.
    Connected,
    /// League client found but requests are failing.
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn in_game() -> CompanionState {
        CompanionState::default()
            .apply(Event::ClientConnected)
            .apply(Event::PhaseChanged(GamePhase::InProgress))
    }

    #[test]
    fn tracks_game_time_only_in_game() {
        let state = CompanionState::default()
            .apply(Event::ClientConnected)
            .apply(Event::PhaseChanged(GamePhase::ChampSelect))
            .apply(Event::GameTimeUpdated(42.0));
        assert_eq!(state.game_time, None);

        let state = in_game().apply(Event::GameTimeUpdated(125.7));
        assert_eq!(state.game_time_label(), "02:05");
    }

    #[test]
    fn leaving_the_game_clears_the_clock() {
        let state = in_game()
            .apply(Event::GameTimeUpdated(600.0))
            .apply(Event::PhaseChanged(GamePhase::EndOfGame));
        assert_eq!(state.game_time, None);
        assert_eq!(state.game_time_label(), "--:--");
    }

    #[test]
    fn disconnect_resets_everything() {
        let state = in_game().apply(Event::ClientDisconnected);
        assert_eq!(state, CompanionState::default());
    }
}
