//! The workout formula: how a finished game becomes burpees, squats and plank seconds
//! (`docs/Product/Workout formula.md`). Pure: game stats and role in, amounts due out.

use crate::game::{Game, GameResult, PlayerStats, Role, queue_in_scope};

/// What a game costs: one amount per exercise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Workout {
    pub burpees: Amount,
    pub squats: Amount,
    /// Seconds.
    pub plank: Amount,
}

/// An amount due for one exercise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Amount {
    pub due: u32,
    /// The game's cap applied: the player owed more than `due`.
    pub capped: bool,
}

/// The workout of `game`, or `None` when it has none: a remake, a queue out of scope, an unknown
/// role or unreadable stats.
#[must_use]
pub fn compute(game: &Game) -> Option<Workout> {
    if game.result == GameResult::Remake
        || game.duration_secs < REMAKE_SECS
        || !queue_in_scope(game.queue_id)
    {
        return None;
    }
    let role = game.role?;
    let stats = game.stats?;

    let (main, secondary) = role_targets(role);
    let burpees = main.gap(game, &stats) * main.kind.burpees_per_unit();
    let squats = secondary.gap(game, &stats) * secondary.kind.squats_per_unit();
    let plank = f64::from(game.deaths) * f64::from(PLANK_SECS_PER_DEATH);

    Some(Workout {
        burpees: amount(burpees, BURPEES_CAP),
        squats: amount(squats, SQUATS_CAP),
        plank: amount(plank, PLANK_CAP),
    })
}

/// A game shorter than this is a remake whatever the client says.
const REMAKE_SECS: u32 = 5 * 60;

/// The most burpees, squats and plank seconds one game can cost.
pub const BURPEES_CAP: u32 = 30;
pub const SQUATS_CAP: u32 = 50;
pub const PLANK_CAP: u32 = 180;
/// Plank seconds owed per death.
pub const PLANK_SECS_PER_DEATH: u32 = 10;

/// A stat the formula can judge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatKind {
    /// Creep score (minions plus monsters) per minute.
    CsPerMinute,
    /// Vision score per minute.
    VisionPerMinute,
    /// Share of the team's damage to champions, in percent.
    DamageShare,
    /// Kills plus assists over the team's kills, in percent.
    KillParticipation,
}

/// A stat and the value a role is expected to reach.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Target {
    pub kind: StatKind,
    /// In the unit of [`StatKind::value`].
    pub value: f64,
}

/// The (main, secondary) targets of a role: burpees pay for the first, squats for the second.
#[must_use]
pub fn role_targets(role: Role) -> (Target, Target) {
    use StatKind::{CsPerMinute, DamageShare, KillParticipation, VisionPerMinute};
    let target = |kind, value| Target { kind, value };
    match role {
        Role::Top => (target(CsPerMinute, 7.0), target(DamageShare, 22.0)),
        Role::Jungle => (target(KillParticipation, 60.0), target(CsPerMinute, 5.5)),
        Role::Mid => (target(DamageShare, 25.0), target(CsPerMinute, 7.5)),
        Role::Bottom => (target(CsPerMinute, 8.0), target(DamageShare, 28.0)),
        Role::Support => (
            target(VisionPerMinute, 2.0),
            target(KillParticipation, 55.0),
        ),
    }
}

impl StatKind {
    /// The player's value in `game`: per minute, or in percent.
    #[must_use]
    pub fn value(self, game: &Game, stats: &PlayerStats) -> f64 {
        let minutes = minutes(game);
        match self {
            Self::CsPerMinute => per_minute(stats.cs, minutes),
            Self::VisionPerMinute => per_minute(stats.vision_score, minutes),
            Self::DamageShare => percent(stats.damage_to_champions, stats.team_damage_to_champions),
            Self::KillParticipation => percent(game.kills + game.assists, stats.team_kills),
        }
    }

    /// Burpees owed per unit of gap (see [`Target::gap`]) when this is the main stat.
    #[must_use]
    pub fn burpees_per_unit(self) -> f64 {
        match self {
            Self::CsPerMinute => 1.0 / 5.0,
            Self::VisionPerMinute | Self::KillParticipation => 1.0 / 2.0,
            Self::DamageShare => 2.0,
        }
    }

    /// Squats owed per unit of gap (see [`Target::gap`]) when this is the secondary stat.
    #[must_use]
    pub fn squats_per_unit(self) -> f64 {
        match self {
            Self::CsPerMinute => 1.0 / 2.0,
            Self::DamageShare => 5.0,
            Self::KillParticipation | Self::VisionPerMinute => 1.0,
        }
    }
}

impl Target {
    /// How far under target the player is, never below 0: missed CS or vision points for the
    /// per-minute stats (the target times the game's minutes, minus the total), percentage points
    /// for the shares.
    #[must_use]
    pub fn gap(self, game: &Game, stats: &PlayerStats) -> f64 {
        let minutes = minutes(game);
        let gap = match self.kind {
            StatKind::CsPerMinute => self.value * minutes - f64::from(stats.cs),
            StatKind::VisionPerMinute => self.value * minutes - f64::from(stats.vision_score),
            StatKind::DamageShare | StatKind::KillParticipation => {
                self.value - self.kind.value(game, stats)
            }
        };
        gap.max(0.0)
    }
}

/// The game's length in minutes.
fn minutes(game: &Game) -> f64 {
    f64::from(game.duration_secs) / 60.0
}

/// `total` per minute, 0 for a game without length.
fn per_minute(total: u32, minutes: f64) -> f64 {
    if minutes > 0.0 {
        f64::from(total) / minutes
    } else {
        0.0
    }
}

/// `part` over `whole`, in percent. 0 when there is no whole (a team with no kill).
fn percent(part: u32, whole: u32) -> f64 {
    if whole == 0 {
        0.0
    } else {
        f64::from(part) * 100.0 / f64::from(whole)
    }
}

/// Rounds up, then applies the cap.
fn amount(value: f64, cap: u32) -> Amount {
    // Float arithmetic leaves dust (25 - 22 = 3.0000000000000004): without this, 6 would be 7.
    let rounded_up = (value - 1e-9).ceil().max(0.0);
    // Amounts are small (a few hundred at most): the conversion cannot truncate.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let due = rounded_up as u32;
    Amount {
        due: due.min(cap),
        capped: due > cap,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::GameSource;

    /// 28 minutes, `kills`/`deaths`/`assists`, and the stats the worked examples need.
    fn game(role: Role, kills: u32, deaths: u32, assists: u32, stats: PlayerStats) -> Game {
        Game {
            game_id: 1,
            source: GameSource::Imported,
            started_at: 0,
            duration_secs: 28 * 60,
            queue_id: 420,
            champion_id: 1,
            role: Some(role),
            result: GameResult::Win,
            kills,
            deaths,
            assists,
            stats: Some(stats),
            workout: None,
        }
    }

    fn stats(cs: u32, vision: u32, damage: u32, team_kills: u32, team_damage: u32) -> PlayerStats {
        PlayerStats {
            cs,
            vision_score: vision,
            damage_to_champions: damage,
            team_kills,
            team_damage_to_champions: team_damage,
        }
    }

    fn amounts(workout: Workout) -> (u32, u32, u32) {
        (workout.burpees.due, workout.squats.due, workout.plank.due)
    }

    #[test]
    fn the_worked_example_of_the_spec() {
        // Mid, 7/6/4, 168 CS, 22 % damage share, 28 minutes: 6 burpees, 21 squats, 60 s.
        let game = game(Role::Mid, 7, 6, 4, stats(168, 20, 22_000, 25, 100_000));
        assert_eq!(amounts(compute(&game).unwrap()), (6, 21, 60));
    }

    #[test]
    fn doing_better_than_the_target_costs_nothing() {
        // Bottom: 300 CS (target 224), 40 % damage share (target 28 %), no death.
        let game = game(Role::Bottom, 10, 0, 5, stats(300, 10, 40_000, 20, 100_000));
        assert_eq!(amounts(compute(&game).unwrap()), (0, 0, 0));
    }

    #[test]
    fn top_pays_cs_in_burpees_and_damage_in_squats() {
        // 28 min: 196 CS target, 150 CS = 46 missed / 5 = 9.2 -> 10 burpees.
        // 17 % damage share = 5 points under 22 x 5 = 25 squats. 3 deaths = 30 s.
        let game = game(Role::Top, 3, 3, 2, stats(150, 10, 17_000, 20, 100_000));
        assert_eq!(amounts(compute(&game).unwrap()), (10, 25, 30));
    }

    #[test]
    fn jungle_pays_kill_participation_in_burpees_and_cs_in_squats() {
        // KP 8/20 = 40 % = 20 points under 60 / 2 = 10 burpees.
        // CS target 5.5 x 28 = 154, 100 CS = 54 missed / 2 = 27 squats.
        let game = game(Role::Jungle, 3, 0, 5, stats(100, 10, 10_000, 20, 100_000));
        assert_eq!(amounts(compute(&game).unwrap()), (10, 27, 0));
    }

    #[test]
    fn support_pays_vision_in_burpees_and_kill_participation_in_squats() {
        // Vision target 2.0 x 28 = 56, 40 = 16 missed / 2 = 8 burpees.
        // KP 10/20 = 50 % = 5 points under 55 x 1 = 5 squats.
        let game = game(Role::Support, 0, 0, 10, stats(30, 40, 5_000, 20, 100_000));
        assert_eq!(amounts(compute(&game).unwrap()), (8, 5, 0));
    }

    #[test]
    fn a_team_with_no_kill_gives_no_kill_participation() {
        // 0 % participation: 60 points under target / 2 = 30 burpees, exactly the cap.
        let game = game(Role::Jungle, 0, 0, 0, stats(154, 10, 0, 0, 100_000));
        let workout = compute(&game).unwrap();
        assert_eq!(
            workout.burpees,
            Amount {
                due: 30,
                capped: false
            }
        );
    }

    #[test]
    fn amounts_are_capped_and_say_so() {
        let game = game(Role::Top, 0, 25, 0, stats(0, 0, 0, 20, 100_000));
        let workout = compute(&game).unwrap();
        assert_eq!(
            workout.burpees,
            Amount {
                due: 30,
                capped: true
            }
        );
        assert_eq!(
            workout.squats,
            Amount {
                due: 50,
                capped: true
            }
        );
        assert_eq!(
            workout.plank,
            Amount {
                due: 180,
                capped: true
            }
        );
    }

    #[test]
    fn an_amount_exactly_at_the_cap_is_not_capped() {
        // 18 deaths x 10 s = 180 s: at the cap, not above it.
        let game = game(Role::Mid, 0, 18, 0, stats(300, 0, 50_000, 20, 100_000));
        assert_eq!(
            compute(&game).unwrap().plank,
            Amount {
                due: 180,
                capped: false
            }
        );
    }

    #[test]
    fn a_remake_has_no_workout() {
        let remake = Game {
            result: GameResult::Remake,
            ..game(Role::Mid, 0, 0, 0, stats(0, 0, 0, 0, 0))
        };
        assert_eq!(compute(&remake), None);
    }

    #[test]
    fn a_game_under_five_minutes_has_no_workout() {
        let short = Game {
            duration_secs: 299,
            ..game(Role::Mid, 0, 0, 0, stats(0, 0, 0, 0, 0))
        };
        assert_eq!(compute(&short), None);
    }

    #[test]
    fn a_queue_out_of_scope_has_no_workout() {
        let aram = Game {
            queue_id: 450,
            ..game(Role::Mid, 0, 0, 0, stats(0, 0, 0, 0, 0))
        };
        assert_eq!(compute(&aram), None);
    }

    #[test]
    fn without_role_or_stats_there_is_no_workout() {
        let base = game(Role::Mid, 0, 0, 0, stats(0, 0, 0, 0, 0));
        assert_eq!(
            compute(&Game {
                role: None,
                ..base.clone()
            }),
            None
        );
        assert_eq!(
            compute(&Game {
                stats: None,
                ..base
            }),
            None
        );
    }

    #[test]
    fn values_are_per_minute_or_in_percent() {
        // 28 minutes, 168 CS, 14 vision, 22 % of the damage, 11 of 25 kills.
        let game = game(Role::Mid, 7, 6, 4, stats(168, 14, 22_000, 25, 100_000));
        let stats = game.stats.unwrap();
        assert!((StatKind::CsPerMinute.value(&game, &stats) - 6.0).abs() < 1e-9);
        assert!((StatKind::VisionPerMinute.value(&game, &stats) - 0.5).abs() < 1e-9);
        assert!((StatKind::DamageShare.value(&game, &stats) - 22.0).abs() < 1e-9);
        assert!((StatKind::KillParticipation.value(&game, &stats) - 44.0).abs() < 1e-9);
    }

    #[test]
    fn the_gap_is_in_missed_units_and_never_negative() {
        let game = game(Role::Mid, 7, 6, 4, stats(168, 14, 22_000, 25, 100_000));
        let stats = game.stats.unwrap();
        let (main, secondary) = role_targets(Role::Mid);
        // Damage share: 25 - 22 = 3 points. CS: 7.5 x 28 - 168 = 42 CS.
        assert!((main.gap(&game, &stats) - 3.0).abs() < 1e-9);
        assert!((secondary.gap(&game, &stats) - 42.0).abs() < 1e-9);
        let top = role_targets(Role::Top).1;
        assert_eq!(Target { value: 10.0, ..top }.gap(&game, &stats), 0.0);
    }
}
