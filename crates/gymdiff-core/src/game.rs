//! A finished game of the active summoner, as [`crate::session`] groups and History lists it.

use crate::workout::Workout;

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
    /// What the [`crate::workout`] formula needs besides the fields above. `None` when the
    /// game's details could not be read.
    pub stats: Option<PlayerStats>,
    /// The amounts due, computed with the formula when the game was stored (a later change of
    /// the formula does not rewrite them). `None` when the game has no workout, see
    /// [`Game::no_workout_reason`].
    pub workout: Option<Workout>,
}

/// The stats of a game behind the workout, for the active summoner and their team.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerStats {
    /// Minions plus neutral monsters killed.
    pub cs: u32,
    pub vision_score: u32,
    pub damage_to_champions: u32,
    pub team_kills: u32,
    pub team_damage_to_champions: u32,
}

impl Game {
    /// Whether History shows this game: games of the queues that give a workout (see
    /// [`queue_in_scope`]) only. The others are not shown at all, not even as a card without
    /// workout.
    #[must_use]
    pub fn is_in_scope(&self) -> bool {
        queue_in_scope(self.queue_id)
    }

    /// When the game ended.
    #[must_use]
    pub fn ended_at(&self) -> i64 {
        self.started_at + i64::from(self.duration_secs)
    }

    /// Why this game has no workout, `None` when it has (or will have) one.
    ///
    /// Only imported games are decided here: a tracked game's workout comes from the post-game
    /// workout, not from this model.
    #[must_use]
    pub fn no_workout_reason(&self) -> Option<NoWorkoutReason> {
        if self.source == GameSource::Tracked {
            return None;
        }
        if self.result == GameResult::Remake {
            Some(NoWorkoutReason::Remake)
        } else if self.role.is_none() {
            Some(NoWorkoutReason::RoleNotDetected)
        } else if self.workout.is_none() {
            Some(NoWorkoutReason::StatsUnreadable)
        } else {
            None
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
    Remake,
    RoleNotDetected,
    StatsUnreadable,
}

impl NoWorkoutReason {
    /// The sentence shown on the card.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Remake => "Remake",
            Self::RoleNotDetected => "Couldn't detect your role",
            Self::StatsUnreadable => "Couldn't read the game's stats",
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
            stats: Some(PlayerStats {
                cs: 200,
                vision_score: 20,
                damage_to_champions: 20_000,
                team_kills: 25,
                team_damage_to_champions: 80_000,
            }),
            workout: Some(Workout::default()),
        }
    }

    #[test]
    fn ended_at_adds_the_duration() {
        assert_eq!(game(1, 1_000, 1_800).ended_at(), 2_800);
    }

    #[test]
    fn an_imported_game_with_a_workout_has_no_reason() {
        assert_eq!(game(1, 0, 1_800).no_workout_reason(), None);
    }

    #[test]
    fn only_the_queues_that_give_a_workout_are_in_scope() {
        for queue_id in [400, 420, 430, 440, 480] {
            assert!(
                Game {
                    queue_id,
                    ..game(1, 0, 1_800)
                }
                .is_in_scope(),
                "{queue_id}"
            );
        }
        // ARAM, ARAM: Mayhem, Arena, practice tool, custom.
        for queue_id in [450, 2400, 1700, 3140, 0] {
            assert!(
                !Game {
                    queue_id,
                    ..game(1, 0, 1_800)
                }
                .is_in_scope(),
                "{queue_id}"
            );
        }
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
    fn without_a_role_the_workout_cannot_be_computed() {
        let game = Game {
            role: None,
            workout: None,
            ..game(1, 0, 1_800)
        };
        assert_eq!(
            game.no_workout_reason(),
            Some(NoWorkoutReason::RoleNotDetected)
        );
    }

    #[test]
    fn without_stats_the_workout_cannot_be_computed() {
        let game = Game {
            stats: None,
            workout: None,
            ..game(1, 0, 1_800)
        };
        assert_eq!(
            game.no_workout_reason(),
            Some(NoWorkoutReason::StatsUnreadable)
        );
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
