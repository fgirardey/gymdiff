//! A finished game of the active summoner, as [`crate::session`] groups and History lists it.

/// One finished game.
///
/// Times are seconds since the Unix epoch (UTC): turning them into days and clock times is the
/// UI's job, so this crate stays free of time zones.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub game_id: u64,
    pub source: GameSource,
    pub started_at: i64,
    pub duration_secs: u32,
    pub queue_id: u32,
    pub champion_id: u32,
    /// `None` when Riot did not detect a role (or the mode has none).
    pub role: Option<Role>,
    pub result: GameResult,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
}

impl Game {
    /// When the game ended.
    #[must_use]
    pub fn ended_at(&self) -> i64 {
        self.started_at + i64::from(self.duration_secs)
    }

    /// Why this game has no workout, `None` when it has (or will have) one.
    ///
    /// Only imported games are decided here: a tracked game's workout status comes from the
    /// post-game workout, not from this model.
    #[must_use]
    pub fn no_workout_reason(&self) -> Option<NoWorkoutReason> {
        match self.source {
            GameSource::Tracked => None,
            GameSource::Imported if self.result == GameResult::Remake => {
                Some(NoWorkoutReason::Remake)
            }
            GameSource::Imported if !queue_in_scope(self.queue_id) => {
                Some(NoWorkoutReason::ModeNotSupported)
            }
            GameSource::Imported => Some(NoWorkoutReason::PlayedBeforeGymDiff),
        }
    }
}

/// How GymDiff learned about a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameSource {
    /// Seen while GymDiff was open: its end-of-game stats were read.
    Tracked,
    /// Read from the client's match history: no workout was possible.
    Imported,
}

/// Position played, as detected by Riot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Top,
    Jungle,
    Mid,
    Bottom,
    Support,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    Win,
    Loss,
    /// Ended before it counted (early surrender).
    Remake,
}

/// Why a game shows the muted "No workout" card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoWorkoutReason {
    PlayedBeforeGymDiff,
    ModeNotSupported,
    Remake,
}

impl NoWorkoutReason {
    /// The sentence shown on the card.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::PlayedBeforeGymDiff => "Played before GymDiff",
            Self::ModeNotSupported => "Mode not supported",
            Self::Remake => "Remake",
        }
    }
}

/// Queues that give a workout: ranked solo/duo, ranked flex, normal draft, normal blind, swiftplay.
#[must_use]
pub fn queue_in_scope(queue_id: u32) -> bool {
    matches!(queue_id, 420 | 440 | 400 | 430 | 480)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn game(game_id: u64, started_at: i64, duration_secs: u32) -> Game {
        Game {
            game_id,
            source: GameSource::Imported,
            started_at,
            duration_secs,
            queue_id: 420,
            champion_id: 103,
            role: Some(Role::Mid),
            result: GameResult::Win,
            kills: 5,
            deaths: 3,
            assists: 4,
        }
    }

    #[test]
    fn ended_at_adds_the_duration() {
        assert_eq!(game(1, 1_000, 1_800).ended_at(), 2_800);
    }

    #[test]
    fn an_imported_game_in_scope_was_played_before_gymdiff() {
        assert_eq!(
            game(1, 0, 1_800).no_workout_reason(),
            Some(NoWorkoutReason::PlayedBeforeGymDiff)
        );
    }

    #[test]
    fn an_imported_game_out_of_scope_is_not_supported() {
        let aram = Game {
            queue_id: 450,
            ..game(1, 0, 1_800)
        };
        assert_eq!(
            aram.no_workout_reason(),
            Some(NoWorkoutReason::ModeNotSupported)
        );
    }

    #[test]
    fn a_remake_is_a_remake_whatever_the_queue() {
        let remake = Game {
            result: GameResult::Remake,
            queue_id: 450,
            ..game(1, 0, 200)
        };
        assert_eq!(remake.no_workout_reason(), Some(NoWorkoutReason::Remake));
    }

    #[test]
    fn a_tracked_game_has_no_reason_here() {
        let tracked = Game {
            source: GameSource::Tracked,
            ..game(1, 0, 1_800)
        };
        assert_eq!(tracked.no_workout_reason(), None);
    }
}
