//! Game flow phases as reported by the League client.

use serde::{Deserialize, Serialize};

/// High-level phase of the League client.
///
/// Mirrors the values returned by the LCU endpoint
/// `GET /lol-gameflow/v1/gameflow-phase`. Unknown values are preserved in
/// [`GamePhase::Unknown`] so that a client update never crashes the app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(from = "String", into = "String")]
pub enum GamePhase {
    #[default]
    None,
    Lobby,
    Matchmaking,
    ReadyCheck,
    ChampSelect,
    GameStart,
    InProgress,
    Reconnect,
    WaitingForStats,
    PreEndOfGame,
    EndOfGame,
    Unknown(String),
}

impl GamePhase {
    /// Whether a game (the actual match) is currently running.
    #[must_use]
    pub fn is_in_game(&self) -> bool {
        matches!(self, Self::GameStart | Self::InProgress | Self::Reconnect)
    }

    /// Short human-readable label, used by the UI.
    #[must_use]
    pub fn label(&self) -> &str {
        match self {
            Self::None => "Idle",
            Self::Lobby => "Lobby",
            Self::Matchmaking => "Matchmaking",
            Self::ReadyCheck => "Ready check",
            Self::ChampSelect => "Champion select",
            Self::GameStart => "Game starting",
            Self::InProgress => "In game",
            Self::Reconnect => "Reconnecting",
            Self::WaitingForStats => "Waiting for stats",
            Self::PreEndOfGame => "Game ending",
            Self::EndOfGame => "End of game",
            Self::Unknown(raw) => raw,
        }
    }
}

impl From<String> for GamePhase {
    fn from(raw: String) -> Self {
        match raw.as_str() {
            "None" => Self::None,
            "Lobby" => Self::Lobby,
            "Matchmaking" => Self::Matchmaking,
            "ReadyCheck" => Self::ReadyCheck,
            "ChampSelect" => Self::ChampSelect,
            "GameStart" => Self::GameStart,
            "InProgress" => Self::InProgress,
            "Reconnect" => Self::Reconnect,
            "WaitingForStats" => Self::WaitingForStats,
            "PreEndOfGame" => Self::PreEndOfGame,
            "EndOfGame" => Self::EndOfGame,
            _ => Self::Unknown(raw),
        }
    }
}

impl From<GamePhase> for String {
    fn from(phase: GamePhase) -> Self {
        match phase {
            GamePhase::None => "None".into(),
            GamePhase::Lobby => "Lobby".into(),
            GamePhase::Matchmaking => "Matchmaking".into(),
            GamePhase::ReadyCheck => "ReadyCheck".into(),
            GamePhase::ChampSelect => "ChampSelect".into(),
            GamePhase::GameStart => "GameStart".into(),
            GamePhase::InProgress => "InProgress".into(),
            GamePhase::Reconnect => "Reconnect".into(),
            GamePhase::WaitingForStats => "WaitingForStats".into(),
            GamePhase::PreEndOfGame => "PreEndOfGame".into(),
            GamePhase::EndOfGame => "EndOfGame".into(),
            GamePhase::Unknown(raw) => raw,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_phases_from_lcu_json() {
        let phase: GamePhase = serde_json::from_str("\"ChampSelect\"").unwrap();
        assert_eq!(phase, GamePhase::ChampSelect);
    }

    #[test]
    fn keeps_unknown_phases() {
        let phase: GamePhase = serde_json::from_str("\"SomethingNew\"").unwrap();
        assert_eq!(phase, GamePhase::Unknown("SomethingNew".into()));
        assert_eq!(phase.label(), "SomethingNew");
    }

    #[test]
    fn round_trips_through_json() {
        let json = serde_json::to_string(&GamePhase::InProgress).unwrap();
        assert_eq!(json, "\"InProgress\"");
    }

    #[test]
    fn in_game_detection() {
        assert!(GamePhase::InProgress.is_in_game());
        assert!(!GamePhase::ChampSelect.is_in_game());
    }
}
