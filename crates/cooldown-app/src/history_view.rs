//! What the History screen shows, from the stored games: sessions with their labels, and one
//! row per game with its stats against the role's targets and the workout they turned into.
//! Plain data (no Slint types), so it is built off the UI thread and unit-tested;
//! [`SessionView::to_slint`] hands it to the window.

use std::path::PathBuf;

use chrono::{Datelike, NaiveDate, NaiveDateTime};
use cooldown_core::workout::{
    Amount, BURPEES_CAP, PLANK_CAP, PLANK_SECS_PER_DEATH, SQUATS_CAP, StatKind, Target,
};
use cooldown_core::{
    Game, GameResult, NoWorkoutReason, PlayerStats, Role, Session, group_into_sessions,
    role_targets,
};
use slint::{ComponentHandle, Image, ModelRc, SharedString, VecModel};

use crate::champion_catalog::{ChampionInfo, initials};
use crate::{AppWindow, ExerciseData, GameRowData, RoleIcons, SessionData, StatData, TermData};

/// One session: its header and its games.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionView {
    pub label: String,
    pub subtitle: String,
    /// Games from before cooldown.lol only: the header has no workout progress.
    pub neutral: bool,
    pub wins: u32,
    pub losses: u32,
    pub games: Vec<GameView>,
}

/// One game row.
#[derive(Debug, Clone, PartialEq)]
pub struct GameView {
    pub champion_initials: String,
    pub champion_name: String,
    pub champion_image: Option<PathBuf>,
    pub role: Option<Role>,
    pub queue: &'static str,
    pub result: GameResult,
    /// "34:12".
    pub length: String,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    /// "1.1 KDA".
    pub kda_ratio: String,
    /// CS per minute, damage share, kill participation, vision per minute; empty when the stats
    /// could not be read.
    pub stats: Vec<StatView>,
    pub workout: WorkoutView,
}

/// A stat of the Games view.
#[derive(Debug, Clone, PartialEq)]
pub struct StatView {
    pub value: String,
    pub under: bool,
    pub fill: f32,
    /// Where the role's target sits on the gauge, when the formula judges this stat.
    pub target: Option<f32>,
}

/// The workout part of a row.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkoutView {
    /// Games from before cooldown.lol were never offered their workout: burpees, squats, plank.
    Skipped(Vec<ExerciseView>),
    /// Why there is no workout.
    NoWorkout { reason: String },
}

/// One exercise: the stat behind it, the amount due and the formula.
#[derive(Debug, Clone, PartialEq)]
pub struct ExerciseView {
    pub name: &'static str,
    pub pays_for: String,
    /// The stat behind it; `None` for the plank, paid by deaths.
    pub stat: Option<StatKind>,
    pub stat_label: &'static str,
    pub value: String,
    pub target: String,
    pub gap: String,
    pub under: bool,
    pub due: u32,
    pub unit: &'static str,
    pub unit_label: &'static str,
    pub capped: bool,
    pub cap: u32,
    pub terms: Vec<Term>,
}

/// A piece of the formula left of the "=".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Operator(&'static str),
    Number {
        text: String,
        caption: String,
        tone: Tone,
    },
}

/// How a number of the formula is coloured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Plain,
    /// The player's own value, under target.
    Under,
    /// The formula's constant.
    Factor,
}

/// The sessions of `games`, newest first, ready to show.
///
/// `local` turns a Unix time into the player's local date and time (it hides the time zone, so
/// the labels are testable); `champion` looks a champion up by id.
pub fn session_views(
    games: &[Game],
    now: i64,
    local: &impl Fn(i64) -> NaiveDateTime,
    champion: &impl Fn(u32) -> Option<ChampionInfo>,
) -> Vec<SessionView> {
    // Games of the other modes are not shown, and must not bridge two sessions either.
    let shown: Vec<Game> = games
        .iter()
        .filter(|game| game.is_in_scope())
        .cloned()
        .collect();
    let sessions = group_into_sessions(&shown);
    let today = local(now).date();
    // The rank of a session within its day counts from the first one of the day.
    let days: Vec<NaiveDate> = sessions
        .iter()
        .map(|session| local(session.started_at()).date())
        .collect();

    sessions
        .iter()
        .enumerate()
        .map(|(index, session)| {
            let day = days[index];
            // Sessions are newest first: the ones of the same day after this one are older.
            let rank = days[index + 1..]
                .iter()
                .filter(|other| **other == day)
                .count()
                + 1;
            let count = |result| {
                let games = session.games().iter();
                u32::try_from(games.filter(|game| game.result == result).count())
                    .unwrap_or(u32::MAX)
            };
            SessionView {
                label: session_label(day, today, rank),
                subtitle: session_subtitle(session),
                neutral: session.is_before_cooldown(),
                wins: count(GameResult::Win),
                losses: count(GameResult::Loss),
                games: session
                    .games()
                    .iter()
                    .map(|game| game_view(game, champion))
                    .collect(),
            }
        })
        .collect()
}

impl SessionView {
    /// The model for the window. Call on the UI thread (images and the role symbols belong to it).
    pub fn to_slint(views: &[Self], window: &AppWindow) -> ModelRc<SessionData> {
        let role_icons = window.global::<RoleIcons>();
        let sessions: Vec<SessionData> = views
            .iter()
            .map(|session| SessionData {
                label: session.label.clone().into(),
                subtitle: session.subtitle.clone().into(),
                neutral: session.neutral,
                wins: int(session.wins),
                losses: int(session.losses),
                games: model(session.games.iter().map(|game| game.to_slint(&role_icons))),
            })
            .collect();
        ModelRc::new(VecModel::from(sessions))
    }
}

impl GameView {
    fn to_slint(&self, role_icons: &RoleIcons<'_>) -> GameRowData {
        // An image that cannot be read is the same as one not downloaded yet: the initials stay.
        let image = self
            .champion_image
            .as_deref()
            .and_then(|path| Image::load_from_path(path).ok());
        let (status, reason, exercises): (&str, &str, &[ExerciseView]) = match &self.workout {
            WorkoutView::Skipped(exercises) => ("SKIPPED", "", exercises),
            WorkoutView::NoWorkout { reason } => ("NO WORKOUT", reason, &[]),
        };
        GameRowData {
            has_champion_image: image.is_some(),
            champion_image: image.unwrap_or_default(),
            champion_initials: self.champion_initials.clone().into(),
            champion_name: self.champion_name.clone().into(),
            has_role: self.role.is_some(),
            role_icon: self
                .role
                .map(|role| role_icon(role_icons, role))
                .unwrap_or_default(),
            role_name: self.role.map_or("Role not detected", role_name).into(),
            queue: self.queue.into(),
            result_label: result_label(self.result).into(),
            win: self.result == GameResult::Win,
            remake: self.result == GameResult::Remake,
            length: self.length.clone().into(),
            kills: int(self.kills),
            deaths: int(self.deaths),
            assists: int(self.assists),
            kda_ratio: self.kda_ratio.clone().into(),
            stats: model(self.stats.iter().map(|stat| StatData {
                value: stat.value.clone().into(),
                under: stat.under,
                fill: stat.fill,
                has_target: stat.target.is_some(),
                target: stat.target.unwrap_or_default(),
            })),
            status: status.into(),
            has_workout: matches!(self.workout, WorkoutView::Skipped(_)),
            reason: reason.into(),
            exercises: model(exercises.iter().map(ExerciseView::to_slint)),
        }
    }
}

impl ExerciseView {
    fn to_slint(&self) -> ExerciseData {
        ExerciseData {
            name: self.name.into(),
            pays_for: self.pays_for.clone().into(),
            stat: match self.stat {
                Some(StatKind::CsPerMinute) => crate::StatKind::CsPerMinute,
                Some(StatKind::DamageShare) => crate::StatKind::DamageShare,
                Some(StatKind::KillParticipation) => crate::StatKind::KillParticipation,
                Some(StatKind::VisionPerMinute) => crate::StatKind::VisionPerMinute,
                None => crate::StatKind::Deaths,
            },
            stat_label: self.stat_label.into(),
            value: self.value.clone().into(),
            target: self.target.clone().into(),
            gap: self.gap.clone().into(),
            under: self.under,
            // Games from before cooldown.lol: nothing was done.
            done: 0,
            due: int(self.due),
            unit: self.unit.into(),
            unit_label: self.unit_label.into(),
            capped: self.capped,
            cap_label: format!("max {}{}", self.cap, self.unit).into(),
            terms: model(self.terms.iter().map(|term| match term {
                Term::Operator(text) => TermData {
                    operator: true,
                    text: (*text).into(),
                    caption: SharedString::new(),
                    under: false,
                    factor: false,
                },
                Term::Number {
                    text,
                    caption,
                    tone,
                } => TermData {
                    operator: false,
                    text: text.clone().into(),
                    caption: caption.clone().into(),
                    under: *tone == Tone::Under,
                    factor: *tone == Tone::Factor,
                },
            })),
        }
    }
}

/// "TODAY · #1", "YESTERDAY · #2", "SUN 1 NOV · #1".
fn session_label(day: NaiveDate, today: NaiveDate, rank: usize) -> String {
    let name = if day == today {
        "Today".to_owned()
    } else if today.pred_opt() == Some(day) {
        "Yesterday".to_owned()
    } else {
        format!("{} {} {}", weekday(day), day.day(), month(day))
    };
    format!("{name} · #{rank}").to_uppercase()
}

fn weekday(day: NaiveDate) -> &'static str {
    ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][day.weekday().num_days_from_monday() as usize]
}

fn month(day: NaiveDate) -> &'static str {
    [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ][day.month0() as usize]
}

/// "5 games · before cooldown.lol".
fn session_subtitle(session: &Session) -> String {
    let count = session.games().len();
    let games = if count == 1 { "game" } else { "games" };
    if session.is_before_cooldown() {
        format!("{count} {games} · before cooldown.lol")
    } else {
        format!("{count} {games}")
    }
}

fn game_view(game: &Game, champion: &impl Fn(u32) -> Option<ChampionInfo>) -> GameView {
    let info = champion(game.champion_id);
    let targets = game.role.map(role_targets);
    let stats = game
        .stats
        .map(|stats| stat_views(game, &stats, targets))
        .unwrap_or_default();
    let workout = match (game.workout, game.stats, targets) {
        (Some(workout), Some(stats), Some((main, secondary))) => WorkoutView::Skipped(vec![
            stat_exercise(Exercise::Burpees, main, workout.burpees, game, &stats),
            stat_exercise(Exercise::Squats, secondary, workout.squats, game, &stats),
            plank_exercise(workout.plank, game.deaths),
        ]),
        _ => WorkoutView::NoWorkout {
            reason: no_workout_reason(game),
        },
    };
    GameView {
        champion_initials: initials(info.as_ref().map(|info| info.name.as_str())),
        champion_name: info
            .as_ref()
            .map_or_else(|| "Unknown champion".to_owned(), |info| info.name.clone()),
        champion_image: info.and_then(|info| info.image),
        role: game.role,
        queue: queue_name(game.queue_id),
        result: game.result,
        length: format!("{}:{:02}", game.duration_secs / 60, game.duration_secs % 60),
        kills: game.kills,
        deaths: game.deaths,
        assists: game.assists,
        kda_ratio: format!(
            "{:.1} KDA",
            f64::from(game.kills + game.assists) / f64::from(game.deaths.max(1))
        ),
        stats,
        workout,
    }
}

/// The Games view's four stats, each against the role's target when the formula judges it.
fn stat_views(
    game: &Game,
    stats: &PlayerStats,
    targets: Option<(Target, Target)>,
) -> Vec<StatView> {
    [
        StatKind::CsPerMinute,
        StatKind::DamageShare,
        StatKind::KillParticipation,
        StatKind::VisionPerMinute,
    ]
    .into_iter()
    .map(|kind| {
        let value = kind.value(game, stats);
        let target = targets
            .into_iter()
            .flat_map(|(main, secondary)| [main, secondary])
            .find(|target| target.kind == kind);
        let scale = gauge_scale(kind);
        #[allow(clippy::cast_possible_truncation)] // A fraction between 0 and 1.
        let fraction = |value: f64| (value / scale).clamp(0.0, 1.0) as f32;
        StatView {
            value: value_label(kind, value),
            under: target.is_some_and(|target| value < target.value),
            fill: fraction(value),
            target: target.map(|target| fraction(target.value)),
        }
    })
    .collect()
}

/// Burpees or squats.
#[derive(Clone, Copy)]
enum Exercise {
    Burpees,
    Squats,
}

/// The exercise paid by a stat: value, target, gap and the formula.
fn stat_exercise(
    exercise: Exercise,
    target: Target,
    amount: Amount,
    game: &Game,
    stats: &PlayerStats,
) -> ExerciseView {
    let kind = target.kind;
    let value = kind.value(game, stats);
    let gap = target.gap(game, stats);
    let under = gap > 0.0;
    let (name, rank, singular, cap, per_unit) = match exercise {
        Exercise::Burpees => (
            "BURPEES",
            "Main",
            "burpee",
            BURPEES_CAP,
            kind.burpees_per_unit(),
        ),
        Exercise::Squats => (
            "SQUATS",
            "Secondary",
            "squat",
            SQUATS_CAP,
            kind.squats_per_unit(),
        ),
    };
    let unit = gap_unit(kind);
    let terms = formula_terms(target, game, stats, under, per_unit, singular);
    ExerciseView {
        name,
        pays_for: format!("{rank} stat · {}", stat_label(kind)),
        stat: Some(kind),
        stat_label: stat_label(kind),
        value: value_label(kind, value),
        target: format!("/ {}", value_label(kind, target.value)),
        gap: if under {
            format!("−{} {unit}", gap.ceil())
        } else {
            String::new()
        },
        under,
        due: amount.due,
        unit: "",
        unit_label: match exercise {
            Exercise::Burpees => "burpees",
            Exercise::Squats => "squats",
        },
        capped: amount.capped,
        cap,
        terms,
    }
}

/// The formula of a stat's exercise, left of the "=": "( target × minutes − total ) ÷ factor" for
/// the per-minute stats, "( target − value ) × factor" for the shares.
fn formula_terms(
    target: Target,
    game: &Game,
    stats: &PlayerStats,
    under: bool,
    per_unit: f64,
    singular: &str,
) -> Vec<Term> {
    let kind = target.kind;
    let tone = if under { Tone::Under } else { Tone::Plain };
    let number = |text: String, caption: String, tone| Term::Number {
        text,
        caption,
        tone,
    };
    let unit = gap_unit(kind);
    let value = kind.value(game, stats);
    let mut terms = vec![Term::Operator("(")];
    match kind {
        StatKind::CsPerMinute | StatKind::VisionPerMinute => {
            let (short, total) = match kind {
                StatKind::CsPerMinute => ("CS", f64::from(stats.cs)),
                _ => ("vision", f64::from(stats.vision_score)),
            };
            terms.extend([
                number(
                    format!("{:.1}", target.value),
                    format!("target {short} / min"),
                    Tone::Plain,
                ),
                Term::Operator("×"),
                number(minutes_label(game), "minutes".to_owned(), Tone::Plain),
                Term::Operator("−"),
                number(
                    format!("{total:.0}"),
                    format!("your {}", total_name(kind)),
                    tone,
                ),
            ]);
        }
        StatKind::DamageShare | StatKind::KillParticipation => terms.extend([
            number(
                format!("{:.0} %", target.value),
                "target".to_owned(),
                Tone::Plain,
            ),
            Term::Operator("−"),
            number(
                format!("{value:.0} %"),
                format!("your {}", stat_label(kind).to_lowercase()),
                tone,
            ),
        ]),
    }
    terms.push(Term::Operator(")"));
    if per_unit < 1.0 {
        terms.extend([
            Term::Operator("÷"),
            number(
                format!("{:.0}", 1.0 / per_unit),
                format!("{unit} per {singular}"),
                Tone::Factor,
            ),
        ]);
    } else {
        terms.extend([
            Term::Operator("×"),
            number(
                format!("{per_unit:.0}"),
                format!("{singular}s per {}", singular_unit(unit)),
                Tone::Factor,
            ),
        ]);
    }
    terms
}

/// The plank: every death costs the same.
fn plank_exercise(amount: Amount, deaths: u32) -> ExerciseView {
    ExerciseView {
        name: "PLANK",
        pays_for: "Every death".to_owned(),
        stat: None,
        stat_label: "Deaths",
        value: deaths.to_string(),
        target: format!("× {PLANK_SECS_PER_DEATH} s"),
        gap: String::new(),
        under: deaths > 0,
        due: amount.due,
        unit: "s",
        unit_label: "plank",
        capped: amount.capped,
        cap: PLANK_CAP,
        terms: vec![
            Term::Number {
                text: deaths.to_string(),
                caption: "deaths".to_owned(),
                tone: if deaths > 0 { Tone::Under } else { Tone::Plain },
            },
            Term::Operator("×"),
            Term::Number {
                text: format!("{PLANK_SECS_PER_DEATH} s"),
                caption: "per death".to_owned(),
                tone: Tone::Factor,
            },
        ],
    }
}

/// "CS per minute", as in the stat captions of the design.
fn stat_label(kind: StatKind) -> &'static str {
    match kind {
        StatKind::CsPerMinute => "CS per minute",
        StatKind::VisionPerMinute => "Vision per minute",
        StatKind::DamageShare => "Damage share",
        StatKind::KillParticipation => "Kill participation",
    }
}

/// "8.5" for the per-minute stats, "24 %" for the shares.
fn value_label(kind: StatKind, value: f64) -> String {
    match kind {
        StatKind::CsPerMinute | StatKind::VisionPerMinute => format!("{value:.1}"),
        StatKind::DamageShare | StatKind::KillParticipation => format!("{value:.0} %"),
    }
}

/// The value at which a gauge is full.
fn gauge_scale(kind: StatKind) -> f64 {
    match kind {
        StatKind::CsPerMinute => 10.0,
        StatKind::VisionPerMinute => 3.0,
        StatKind::DamageShare | StatKind::KillParticipation => 100.0,
    }
}

/// The unit the gap is counted in: "CS", "vision", "pts".
fn gap_unit(kind: StatKind) -> &'static str {
    match kind {
        StatKind::CsPerMinute => "CS",
        StatKind::VisionPerMinute => "vision",
        StatKind::DamageShare | StatKind::KillParticipation => "pts",
    }
}

fn singular_unit(unit: &str) -> &str {
    if unit == "pts" { "point" } else { unit }
}

/// "CS", "vision score": the total the per-minute target is compared with.
fn total_name(kind: StatKind) -> &'static str {
    match kind {
        StatKind::VisionPerMinute => "vision score",
        _ => "CS",
    }
}

/// "28" or "34.2".
fn minutes_label(game: &Game) -> String {
    let minutes = f64::from(game.duration_secs) / 60.0;
    if game.duration_secs.is_multiple_of(60) {
        format!("{minutes:.0}")
    } else {
        format!("{minutes:.1}")
    }
}

/// The queues History shows (see `cooldown_core::game::queue_in_scope`).
fn queue_name(queue_id: u32) -> &'static str {
    match queue_id {
        420 => "Ranked Solo/Duo",
        440 => "Ranked Flex",
        400 => "Normal Draft",
        430 => "Normal Blind",
        480 => "Swiftplay",
        _ => "",
    }
}

fn result_label(result: GameResult) -> &'static str {
    match result {
        GameResult::Win => "WIN",
        GameResult::Loss => "LOSS",
        GameResult::Remake => "REMAKE",
    }
}

fn role_name(role: Role) -> &'static str {
    match role {
        Role::Top => "Top",
        Role::Jungle => "Jungle",
        Role::Mid => "Mid",
        Role::Bottom => "Bottom",
        Role::Support => "Support",
    }
}

/// "Remake", "Couldn't detect your role".
fn no_workout_reason(game: &Game) -> String {
    game.no_workout_reason()
        .unwrap_or(NoWorkoutReason::StatsUnreadable)
        .label()
        .to_owned()
}

fn role_icon(icons: &RoleIcons<'_>, role: Role) -> Image {
    match role {
        Role::Top => icons.get_top(),
        Role::Jungle => icons.get_jungle(),
        Role::Mid => icons.get_middle(),
        Role::Bottom => icons.get_bottom(),
        Role::Support => icons.get_utility(),
    }
}

fn int(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

fn model<T: Clone + 'static>(items: impl Iterator<Item = T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(items.collect::<Vec<_>>()))
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use cooldown_core::workout::{Amount, Workout};
    use cooldown_core::{GameSource, PlayerStats};

    use super::*;

    const HOUR: i64 = 3_600;

    /// 2026-11-08 12:00 UTC, a Sunday: "now" of the tests, in a UTC "local" time.
    const NOW: i64 = 1_794_139_200;

    fn utc(seconds: i64) -> NaiveDateTime {
        DateTime::<Utc>::from_timestamp(seconds, 0)
            .unwrap()
            .naive_utc()
    }

    fn jinx(champion_id: u32) -> Option<ChampionInfo> {
        (champion_id == 222).then(|| ChampionInfo {
            name: "Jinx".to_owned(),
            image: Some(PathBuf::from("Jinx.png")),
        })
    }

    fn game(game_id: u64, started_at: i64) -> Game {
        Game {
            game_id,
            source: GameSource::Imported,
            started_at,
            duration_secs: 1_800,
            queue_id: 420,
            champion_id: 222,
            role: Some(Role::Bottom),
            result: GameResult::Win,
            kills: 9,
            deaths: 3,
            assists: 7,
            stats: Some(PlayerStats {
                cs: 200,
                vision_score: 20,
                damage_to_champions: 20_000,
                team_kills: 25,
                team_damage_to_champions: 80_000,
            }),
            workout: Some(Workout {
                burpees: Amount {
                    due: 5,
                    capped: false,
                },
                squats: Amount {
                    due: 0,
                    capped: false,
                },
                plank: Amount {
                    due: 30,
                    capped: false,
                },
            }),
        }
    }

    fn views(games: &[Game]) -> Vec<SessionView> {
        session_views(games, NOW, &utc, &jinx)
    }

    #[test]
    fn labels_the_day_and_the_rank_of_the_session_in_it() {
        // Today 02:00 and 06:00 (two sessions), yesterday 20:00, 3 November 10:00.
        let games = [
            game(4, NOW - 6 * HOUR),
            game(3, NOW - 10 * HOUR),
            game(2, NOW - 16 * HOUR),
            game(1, NOW - 5 * 24 * HOUR),
        ];
        let labels: Vec<String> = views(&games).into_iter().map(|s| s.label).collect();
        assert_eq!(
            labels,
            [
                "TODAY · #2",
                "TODAY · #1",
                "YESTERDAY · #1",
                "TUE 3 NOV · #1"
            ]
        );
    }

    #[test]
    fn the_subtitle_counts_the_games_and_says_they_are_from_before() {
        let one = views(&[game(1, NOW - HOUR)]);
        assert_eq!(one[0].subtitle, "1 game · before cooldown.lol");
        let two = views(&[game(1, NOW - 2 * HOUR), game(2, NOW - HOUR + 600)]);
        assert_eq!(two[0].subtitle, "2 games · before cooldown.lol");
        assert!(two[0].neutral);
    }

    #[test]
    fn a_row_shows_the_game() {
        let view = &views(&[game(1, NOW - HOUR)])[0].games[0];
        assert_eq!(view.champion_initials, "JI");
        assert_eq!(view.champion_name, "Jinx");
        assert_eq!(view.champion_image, Some(PathBuf::from("Jinx.png")));
        assert_eq!(view.role, Some(Role::Bottom));
        assert_eq!(view.queue, "Ranked Solo/Duo");
        assert_eq!(view.result, GameResult::Win);
        assert_eq!(view.length, "30:00");
        assert_eq!((view.kills, view.deaths, view.assists), (9, 3, 7));
        assert_eq!(view.kda_ratio, "5.3 KDA");
    }

    #[test]
    fn stats_are_red_under_the_target_of_the_role_only() {
        // Bottom: CS per minute (8.0) pays burpees, damage share (28 %) pays squats.
        let stats = &views(&[game(1, NOW - HOUR)])[0].games[0].stats;
        let values: Vec<&str> = stats.iter().map(|stat| stat.value.as_str()).collect();
        assert_eq!(values, ["6.7", "25 %", "64 %", "0.7"]);
        let under: Vec<bool> = stats.iter().map(|stat| stat.under).collect();
        assert_eq!(under, [true, true, false, false]);
        let targets: Vec<bool> = stats.iter().map(|stat| stat.target.is_some()).collect();
        assert_eq!(targets, [true, true, false, false]);
        assert!((stats[0].target.unwrap() - 0.8).abs() < 1e-6);
        assert!((stats[2].fill - 0.64).abs() < 1e-6);
    }

    #[test]
    fn each_exercise_shows_its_stat_and_formula() {
        let WorkoutView::Skipped(exercises) = &views(&[game(1, NOW - HOUR)])[0].games[0].workout
        else {
            panic!("a workout");
        };
        let [burpees, squats, plank] = exercises.as_slice() else {
            panic!("three exercises");
        };
        assert_eq!(burpees.pays_for, "Main stat · CS per minute");
        assert_eq!(
            (
                burpees.value.as_str(),
                burpees.target.as_str(),
                burpees.gap.as_str()
            ),
            ("6.7", "/ 8.0", "−40 CS")
        );
        assert_eq!(burpees.due, 5);
        let texts = |exercise: &ExerciseView| -> Vec<String> {
            exercise
                .terms
                .iter()
                .map(|term| match term {
                    Term::Operator(text) => (*text).to_owned(),
                    Term::Number { text, .. } => text.clone(),
                })
                .collect()
        };
        assert_eq!(
            texts(burpees),
            ["(", "8.0", "×", "30", "−", "200", ")", "÷", "5"]
        );
        assert_eq!(squats.gap, "−3 pts");
        assert_eq!(texts(squats), ["(", "28 %", "−", "25 %", ")", "×", "5"]);
        assert!(matches!(
            squats.terms.last(),
            Some(Term::Number { caption, tone: Tone::Factor, .. }) if caption == "squats per point"
        ));
        assert_eq!(
            (plank.value.as_str(), plank.target.as_str()),
            ("3", "× 10 s")
        );
        assert_eq!((plank.due, plank.unit), (30, "s"));
    }

    #[test]
    fn the_session_counts_its_wins_and_losses() {
        let loss = Game {
            result: GameResult::Loss,
            ..game(2, NOW - HOUR + 600)
        };
        let session = &views(&[game(1, NOW - 2 * HOUR), loss])[0];
        assert_eq!((session.wins, session.losses), (1, 1));
    }

    #[test]
    fn an_unknown_champion_shows_a_question_mark() {
        let unknown = Game {
            champion_id: 9_999,
            ..game(1, NOW - HOUR)
        };
        let view = &views(&[unknown])[0].games[0];
        assert_eq!(view.champion_initials, "?");
        assert_eq!(view.champion_image, None);
    }

    #[test]
    fn a_game_without_a_workout_says_why() {
        let remake = Game {
            result: GameResult::Remake,
            workout: None,
            ..game(1, NOW - 3 * HOUR)
        };
        let norole = Game {
            role: None,
            workout: None,
            ..game(2, NOW - HOUR)
        };
        // Newest first, across the two sessions (the break is over an hour).
        let cards: Vec<WorkoutView> = views(&[remake, norole])
            .into_iter()
            .flat_map(|session| session.games)
            .map(|game| game.workout)
            .collect();
        let no_workout = |reason: &str| WorkoutView::NoWorkout {
            reason: reason.to_owned(),
        };
        assert_eq!(
            cards,
            [
                no_workout("Couldn't detect your role"),
                no_workout("Remake")
            ]
        );
        let remake = &views(&[Game {
            result: GameResult::Remake,
            ..game(9, NOW)
        }])[0]
            .games[0];
        assert_eq!(remake.result, GameResult::Remake);
    }

    #[test]
    fn games_of_the_other_modes_are_not_shown_and_do_not_bridge_sessions() {
        // Two ranked games 105 minutes apart, with an ARAM game that would join them: two
        // sessions, and the ARAM game is nowhere.
        let ranked = [game(1, NOW - 6 * HOUR), game(3, NOW - 6 * HOUR + 8_100)];
        let aram = Game {
            queue_id: 450,
            ..game(2, NOW - 6 * HOUR + HOUR)
        };
        let sessions = views(&[ranked[0].clone(), aram, ranked[1].clone()]);
        assert_eq!(sessions.len(), 2);
        assert!(sessions.iter().all(|session| session.games.len() == 1));
        let only_aram = views(&[Game {
            queue_id: 450,
            ..game(4, NOW - HOUR)
        }]);
        assert_eq!(only_aram, Vec::<SessionView>::new());
    }
}
