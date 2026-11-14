//! What the History screen shows, from the stored games: sessions with their labels, cards
//! with their champion and amounts. Plain data (no Slint types), so it is built off the UI
//! thread and unit-tested; [`SessionView::to_slint`] hands it to the window.

use std::path::PathBuf;

use chrono::{Datelike, NaiveDate, NaiveDateTime};
use gymdiff_core::{Game, GameResult, NoWorkoutReason, Role, Session, group_into_sessions};
use slint::{ComponentHandle, Image, ModelRc, VecModel};

use crate::champion_catalog::{ChampionInfo, initials};
use crate::{AppWindow, GameCardData, RoleIcons, SessionData};

/// One session panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionView {
    pub label: String,
    pub subtitle: String,
    /// Games from before GymDiff only: the header has no ring and no totals.
    pub neutral: bool,
    pub games: Vec<GameView>,
}

/// One card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameView {
    pub champion_initials: String,
    pub champion_image: Option<PathBuf>,
    pub role: Option<Role>,
    pub win: bool,
    pub result_label: &'static str,
    pub kda: String,
    pub card: Card,
}

/// The two looks of a card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Card {
    /// A workout: the amounts due. Games from before GymDiff were skipped, so none is done.
    Skipped {
        burpees: u32,
        squats: u32,
        plank: u32,
    },
    /// The muted card: why there is no workout.
    NoWorkout { reason: String },
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
            SessionView {
                label: session_label(day, today, rank),
                subtitle: session_subtitle(session),
                neutral: session.is_before_gymdiff(),
                games: session
                    .games()
                    .iter()
                    .map(|game| game_view(game, champion))
                    .collect(),
            }
        })
        .collect()
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

/// "5 games · before GymDiff".
fn session_subtitle(session: &Session) -> String {
    let count = session.games().len();
    let games = if count == 1 { "game" } else { "games" };
    if session.is_before_gymdiff() {
        format!("{count} {games} · before GymDiff")
    } else {
        format!("{count} {games}")
    }
}

fn game_view(game: &Game, champion: &impl Fn(u32) -> Option<ChampionInfo>) -> GameView {
    let info = champion(game.champion_id);
    let card = match game.workout {
        Some(workout) => Card::Skipped {
            burpees: workout.burpees.due,
            squats: workout.squats.due,
            plank: workout.plank.due,
        },
        None => Card::NoWorkout {
            reason: no_workout_reason(game),
        },
    };
    GameView {
        champion_initials: initials(info.as_ref().map(|info| info.name.as_str())),
        champion_image: info.and_then(|info| info.image),
        role: game.role,
        win: game.result == GameResult::Win,
        result_label: match game.result {
            GameResult::Win => "WIN",
            GameResult::Loss => "LOSS",
            GameResult::Remake => "REMAKE",
        },
        kda: format!("{}/{}/{}", game.kills, game.deaths, game.assists),
        card,
    }
}

/// "Remake", "Couldn't detect your role".
fn no_workout_reason(game: &Game) -> String {
    game.no_workout_reason()
        .unwrap_or(NoWorkoutReason::StatsUnreadable)
        .label()
        .to_owned()
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
                games: ModelRc::new(VecModel::from(
                    session
                        .games
                        .iter()
                        .map(|game| game.to_slint(&role_icons))
                        .collect::<Vec<_>>(),
                )),
            })
            .collect();
        ModelRc::new(VecModel::from(sessions))
    }
}

impl GameView {
    fn to_slint(&self, role_icons: &RoleIcons<'_>) -> GameCardData {
        // An image that cannot be read is the same as one not downloaded yet: the initials stay.
        let image = self
            .champion_image
            .as_deref()
            .and_then(|path| Image::load_from_path(path).ok());
        let (burpees, squats, plank) = match self.card {
            Card::Skipped {
                burpees,
                squats,
                plank,
            } => (burpees, squats, plank),
            Card::NoWorkout { .. } => (0, 0, 0),
        };
        let amount = |value: u32| i32::try_from(value).unwrap_or(i32::MAX);
        GameCardData {
            has_champion_image: image.is_some(),
            champion_image: image.unwrap_or_default(),
            champion_initials: self.champion_initials.clone().into(),
            has_role: self.role.is_some(),
            role_icon: self
                .role
                .map(|role| role_icon(role_icons, role))
                .unwrap_or_default(),
            result_label: self.result_label.into(),
            win: self.win,
            kda: self.kda.clone().into(),
            has_workout: matches!(self.card, Card::Skipped { .. }),
            burpees_done: 0,
            burpees_due: amount(burpees),
            squats_done: 0,
            squats_due: amount(squats),
            plank_done: 0,
            plank_due: amount(plank),
            reason: match &self.card {
                Card::NoWorkout { reason } => reason.clone().into(),
                Card::Skipped { .. } => "".into(),
            },
        }
    }
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

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use gymdiff_core::workout::{Amount, Workout};
    use gymdiff_core::{GameSource, PlayerStats};

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
        assert_eq!(one[0].subtitle, "1 game · before GymDiff");
        let two = views(&[game(1, NOW - 2 * HOUR), game(2, NOW - HOUR + 600)]);
        assert_eq!(two[0].subtitle, "2 games · before GymDiff");
        assert!(two[0].neutral);
    }

    #[test]
    fn a_card_shows_the_game_and_its_amounts() {
        let view = &views(&[game(1, NOW - HOUR)])[0].games[0];
        assert_eq!(view.champion_initials, "JI");
        assert_eq!(view.champion_image, Some(PathBuf::from("Jinx.png")));
        assert_eq!(view.role, Some(Role::Bottom));
        assert!(view.win);
        assert_eq!(view.result_label, "WIN");
        assert_eq!(view.kda, "9/3/7");
        assert_eq!(
            view.card,
            Card::Skipped {
                burpees: 5,
                squats: 0,
                plank: 30
            }
        );
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
        let cards: Vec<Card> = views(&[remake, norole])
            .into_iter()
            .flat_map(|session| session.games)
            .map(|game| game.card)
            .collect();
        let no_workout = |reason: &str| Card::NoWorkout {
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
        assert_eq!(remake.result_label, "REMAKE");
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
