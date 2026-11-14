//! Models for the LCU match history of the active summoner, and their conversion to
//! [`gymdiff_core::Game`].
//!
//! `GET /lol-match-history/v1/products/lol/current-summoner/matches?begIndex=&endIndex=`.
//! Only the fields GymDiff uses are modelled. Each game lists the other players too, but only the
//! first participant (the active summoner) is read: nobody else is ever kept.

use gymdiff_core::{Game, GameResult, GameSource, Role};
use serde::Deserialize;

/// Response of the match history endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MatchHistory {
    #[serde(default)]
    games: GamesPage,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct GamesPage {
    #[serde(default)]
    games: Vec<RawGame>,
}

impl MatchHistory {
    /// The page's games as imported [`Game`]s, newest first as the client sorts them. A game the
    /// payload does not describe well enough (no participant) is skipped.
    #[must_use]
    pub fn into_games(self) -> Vec<Game> {
        self.games
            .games
            .into_iter()
            .filter_map(RawGame::into_game)
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawGame {
    game_id: u64,
    /// Milliseconds since the Unix epoch.
    game_creation: i64,
    /// Seconds.
    game_duration: u32,
    #[serde(default)]
    queue_id: u32,
    #[serde(default)]
    participants: Vec<Participant>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Participant {
    #[serde(default)]
    champion_id: u32,
    #[serde(default)]
    stats: Stats,
    #[serde(default)]
    timeline: Timeline,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Stats {
    #[serde(default)]
    win: bool,
    #[serde(default)]
    kills: u32,
    #[serde(default)]
    deaths: u32,
    #[serde(default)]
    assists: u32,
    #[serde(default)]
    game_ended_in_early_surrender: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct Timeline {
    #[serde(default)]
    lane: String,
    #[serde(default)]
    role: String,
}

impl RawGame {
    fn into_game(self) -> Option<Game> {
        let participant = self.participants.into_iter().next()?;
        let stats = participant.stats;
        let result = if stats.game_ended_in_early_surrender {
            GameResult::Remake
        } else if stats.win {
            GameResult::Win
        } else {
            GameResult::Loss
        };
        Some(Game {
            game_id: self.game_id,
            source: GameSource::Imported,
            started_at: self.game_creation.div_euclid(1_000),
            duration_secs: self.game_duration,
            queue_id: self.queue_id,
            champion_id: participant.champion_id,
            role: role_of(&participant.timeline),
            result,
            kills: stats.kills,
            deaths: stats.deaths,
            assists: stats.assists,
        })
    }
}

/// Riot reports the bottom lane's two roles in `role`: `DUO_CARRY` and `DUO_SUPPORT`.
fn role_of(timeline: &Timeline) -> Option<Role> {
    match timeline.lane.as_str() {
        "TOP" => Some(Role::Top),
        "JUNGLE" => Some(Role::Jungle),
        "MIDDLE" | "MID" => Some(Role::Mid),
        "BOTTOM" | "BOT" => match timeline.role.as_str() {
            "DUO_SUPPORT" | "SUPPORT" => Some(Role::Support),
            _ => Some(Role::Bottom),
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"{
        "accountId": 1,
        "games": {
            "gameBeginDate": "2026-11-06",
            "gameCount": 3,
            "games": [
                {
                    "gameId": 7001,
                    "gameCreation": 1762300000000,
                    "gameDuration": 1830,
                    "queueId": 420,
                    "mapId": 11,
                    "participantIdentities": [{"participantId": 1, "player": {"summonerName": "someone"}}],
                    "participants": [
                        {
                            "participantId": 1,
                            "championId": 412,
                            "stats": {"win": true, "kills": 2, "deaths": 4, "assists": 17},
                            "timeline": {"lane": "BOTTOM", "role": "DUO_SUPPORT"}
                        }
                    ],
                    "someFutureField": true
                },
                {
                    "gameId": 7002,
                    "gameCreation": 1762290000000,
                    "gameDuration": 210,
                    "queueId": 420,
                    "participants": [
                        {
                            "championId": 103,
                            "stats": {"win": false, "gameEndedInEarlySurrender": true},
                            "timeline": {"lane": "NONE", "role": "NONE"}
                        }
                    ]
                },
                {"gameId": 7003, "gameCreation": 1762280000000, "gameDuration": 900, "participants": []}
            ]
        }
    }"#;

    #[test]
    fn parses_games_and_ignores_unknown_fields() {
        let games = serde_json::from_str::<MatchHistory>(FIXTURE)
            .unwrap()
            .into_games();
        // The third game has no participant: skipped.
        assert_eq!(games.len(), 2);
        assert_eq!(
            games[0],
            Game {
                game_id: 7001,
                source: GameSource::Imported,
                started_at: 1_762_300_000,
                duration_secs: 1_830,
                queue_id: 420,
                champion_id: 412,
                role: Some(Role::Support),
                result: GameResult::Win,
                kills: 2,
                deaths: 4,
                assists: 17,
            }
        );
    }

    #[test]
    fn an_early_surrender_is_a_remake_without_role() {
        let games = serde_json::from_str::<MatchHistory>(FIXTURE)
            .unwrap()
            .into_games();
        assert_eq!(games[1].result, GameResult::Remake);
        assert_eq!(games[1].role, None);
    }

    #[test]
    fn an_empty_or_missing_page_gives_no_games() {
        for json in ["{}", r#"{"games": {}}"#] {
            let games = serde_json::from_str::<MatchHistory>(json)
                .unwrap()
                .into_games();
            assert_eq!(games, Vec::<Game>::new());
        }
    }

    #[test]
    fn maps_lanes_to_roles() {
        let role = |lane: &str, role: &str| {
            role_of(&Timeline {
                lane: lane.to_owned(),
                role: role.to_owned(),
            })
        };
        assert_eq!(role("TOP", "SOLO"), Some(Role::Top));
        assert_eq!(role("JUNGLE", "NONE"), Some(Role::Jungle));
        assert_eq!(role("MIDDLE", "SOLO"), Some(Role::Mid));
        assert_eq!(role("BOTTOM", "DUO_CARRY"), Some(Role::Bottom));
        assert_eq!(role("BOTTOM", "DUO_SUPPORT"), Some(Role::Support));
        assert_eq!(role("NONE", "NONE"), None);
    }
}
