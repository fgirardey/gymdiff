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

    /// Whether a game just ended: from the stats screen to the end-of-game screen.
    #[must_use]
    pub fn is_after_game(&self) -> bool {
        matches!(
            self,
            Self::WaitingForStats | Self::PreEndOfGame | Self::EndOfGame
        )
    }

    /// Whether going from `previous` to this phase may have added a game to the client's match
    /// history: on reaching the end-of-game screen, and again on leaving the post-game phases,
    /// since the match history can list the game a little after that screen shows.
    #[must_use]
    pub fn may_list_a_new_game(&self, previous: &Self) -> bool {
        let reached_end = *self == Self::EndOfGame && *previous != Self::EndOfGame;
        let left_post_game = previous.is_after_game() && !self.is_after_game();
        reached_end || left_post_game
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

    #[test]
    fn a_new_game_may_be_listed_at_the_end_of_game_and_after_it() {
        use GamePhase::{ChampSelect, EndOfGame, InProgress, Lobby, PreEndOfGame, WaitingForStats};
        assert!(EndOfGame.may_list_a_new_game(&PreEndOfGame));
        assert!(Lobby.may_list_a_new_game(&EndOfGame));
        // A game ended without its end-of-game screen.
        assert!(GamePhase::None.may_list_a_new_game(&WaitingForStats));

        assert!(!PreEndOfGame.may_list_a_new_game(&WaitingForStats));
        assert!(!WaitingForStats.may_list_a_new_game(&InProgress));
        assert!(!ChampSelect.may_list_a_new_game(&Lobby));
        assert!(!EndOfGame.may_list_a_new_game(&EndOfGame));
    }
}
