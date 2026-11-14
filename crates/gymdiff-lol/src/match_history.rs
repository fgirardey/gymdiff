//! Models for the LCU match history, and their conversion to [`gymdiff_core::Game`].
//!
//! Two payloads, both read leniently (only the fields GymDiff uses):
//! - the list of the active summoner's recent games ([`MatchHistory`]), which describes the
//!   player alone: one participant per game;
//! - the detail of one game ([`GameDetail`]), with its ten participants, of which only the two
//!   sums of the player's team are kept. No other player's stats or name ever leave this module.

use gymdiff_core::{Game, GameResult, GameSource, PlayerStats, Role};
use serde::Deserialize;

/// The summoner spell Smite: what tells a jungler from a top laner, which Riot's lane
/// detection can both call `JUNGLE`.
const SMITE: u32 = 11;

/// Response of `GET /lol-match-history/v1/products/lol/current-summoner/matches`.
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
    /// The page's games, newest first as the client sorts them. A game the payload does not
    /// describe well enough (no participant) is skipped.
    #[must_use]
    pub fn into_games(self) -> Vec<PastGame> {
        self.games
            .games
            .into_iter()
            .filter_map(RawGame::into_past_game)
            .collect()
    }
}

/// A game of the match history, still missing its team totals (see [`GameDetail`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PastGame {
    game: Game,
    team_id: u32,
    cs: u32,
    vision_score: u32,
    damage_to_champions: u32,
}

impl PastGame {
    /// The game without its stats: enough to know whether it is stored already.
    #[must_use]
    pub fn game(&self) -> &Game {
        &self.game
    }

    /// The player's team in the game's detail.
    #[must_use]
    pub fn team_id(&self) -> u32 {
        self.team_id
    }

    /// The game with its stats, from the sums of the player's team (`None` when the detail could
    /// not be read: the game then has no workout).
    #[must_use]
    pub fn with_team_totals(self, totals: Option<TeamTotals>) -> Game {
        Game {
            stats: totals.map(|totals| PlayerStats {
                cs: self.cs,
                vision_score: self.vision_score,
                damage_to_champions: self.damage_to_champions,
                team_kills: totals.kills,
                team_damage_to_champions: totals.damage_to_champions,
            }),
            ..self.game
        }
    }
}

/// The sums of one team in a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeamTotals {
    pub kills: u32,
    pub damage_to_champions: u32,
}

/// Response of `GET /lol-match-history/v1/games/{gameId}`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct GameDetail {
    #[serde(default)]
    participants: Vec<DetailParticipant>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DetailParticipant {
    #[serde(default)]
    team_id: u32,
    #[serde(default)]
    stats: DetailStats,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DetailStats {
    #[serde(default)]
    kills: u32,
    #[serde(default)]
    total_damage_dealt_to_champions: u32,
}

impl GameDetail {
    /// The kills and damage to champions of `team_id`; `None` when nobody played for it.
    #[must_use]
    pub fn team_totals(&self, team_id: u32) -> Option<TeamTotals> {
        let team: Vec<&DetailParticipant> = self
            .participants
            .iter()
            .filter(|participant| participant.team_id == team_id)
            .collect();
        if team.is_empty() {
            return None;
        }
        Some(TeamTotals {
            kills: team.iter().map(|p| p.stats.kills).sum(),
            damage_to_champions: team
                .iter()
                .map(|p| p.stats.total_damage_dealt_to_champions)
                .sum(),
        })
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
    team_id: u32,
    #[serde(default)]
    spell1_id: u32,
    #[serde(default)]
    spell2_id: u32,
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
    #[serde(default)]
    total_minions_killed: u32,
    #[serde(default)]
    neutral_minions_killed: u32,
    #[serde(default)]
    total_damage_dealt_to_champions: u32,
    #[serde(default)]
    vision_score: u32,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct Timeline {
    #[serde(default)]
    lane: String,
    #[serde(default)]
    role: String,
}

impl RawGame {
    fn into_past_game(self) -> Option<PastGame> {
        let participant = self.participants.into_iter().next()?;
        let stats = &participant.stats;
        let result = if stats.game_ended_in_early_surrender {
            GameResult::Remake
        } else if stats.win {
            GameResult::Win
        } else {
            GameResult::Loss
        };
        let has_smite = participant.spell1_id == SMITE || participant.spell2_id == SMITE;
        Some(PastGame {
            team_id: participant.team_id,
            cs: stats.total_minions_killed + stats.neutral_minions_killed,
            vision_score: stats.vision_score,
            damage_to_champions: stats.total_damage_dealt_to_champions,
            game: Game {
                game_id: self.game_id,
                source: GameSource::Imported,
                started_at: self.game_creation.div_euclid(1_000),
                duration_secs: self.game_duration,
                queue_id: self.queue_id,
                champion_id: participant.champion_id,
                role: role_of(&participant.timeline, has_smite),
                result,
                kills: stats.kills,
                deaths: stats.deaths,
                assists: stats.assists,
                stats: None,
                workout: None,
            },
        })
    }
}

/// Riot reports the bottom lane's two roles in `role`: `CARRY` and `SUPPORT`. Its lane detection
/// can call a top laner `JUNGLE`, so a `JUNGLE` without Smite is a top laner.
fn role_of(timeline: &Timeline, has_smite: bool) -> Option<Role> {
    match timeline.lane.as_str() {
        "JUNGLE" if has_smite => Some(Role::Jungle),
        "TOP" | "JUNGLE" => Some(Role::Top),
        "MIDDLE" | "MID" => Some(Role::Mid),
        "BOTTOM" | "BOT" => match timeline.role.as_str() {
            "SUPPORT" | "DUO_SUPPORT" => Some(Role::Support),
            _ => Some(Role::Bottom),
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = r#"{
        "accountId": 1,
        "games": {
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
                            "participantId": 9,
                            "teamId": 200,
                            "championId": 412,
                            "spell1Id": 4,
                            "spell2Id": 3,
                            "stats": {
                                "win": true, "kills": 2, "deaths": 4, "assists": 17,
                                "totalMinionsKilled": 30, "neutralMinionsKilled": 2,
                                "totalDamageDealtToChampions": 9000, "visionScore": 80
                            },
                            "timeline": {"lane": "BOTTOM", "role": "SUPPORT"}
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

    const DETAIL: &str = r#"{
        "gameId": 7001,
        "participantIdentities": [{"participantId": 1, "player": {"gameName": "Somebody Else"}}],
        "participants": [
            {"participantId": 1, "teamId": 100, "stats": {"kills": 3, "totalDamageDealtToChampions": 20000}},
            {"participantId": 2, "teamId": 100, "stats": {"kills": 7, "totalDamageDealtToChampions": 25000}},
            {"participantId": 9, "teamId": 200, "stats": {"kills": 2, "totalDamageDealtToChampions": 9000}},
            {"participantId": 10, "teamId": 200, "stats": {"kills": 11, "totalDamageDealtToChampions": 31000}}
        ]
    }"#;

    fn past_games() -> Vec<PastGame> {
        serde_json::from_str::<MatchHistory>(LIST)
            .unwrap()
            .into_games()
    }

    #[test]
    fn parses_games_and_ignores_unknown_fields() {
        let games = past_games();
        // The third game has no participant: skipped.
        assert_eq!(games.len(), 2);
        assert_eq!(
            games[0].game(),
            &Game {
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
                stats: None,
                workout: None,
            }
        );
        assert_eq!(games[0].team_id(), 200);
    }

    #[test]
    fn an_early_surrender_is_a_remake_without_role() {
        let games = past_games();
        assert_eq!(games[1].game().result, GameResult::Remake);
        assert_eq!(games[1].game().role, None);
    }

    #[test]
    fn an_empty_or_missing_page_gives_no_games() {
        for json in ["{}", r#"{"games": {}}"#] {
            let games = serde_json::from_str::<MatchHistory>(json)
                .unwrap()
                .into_games();
            assert_eq!(games, Vec::<PastGame>::new());
        }
    }

    #[test]
    fn sums_the_players_team_in_the_detail() {
        let detail: GameDetail = serde_json::from_str(DETAIL).unwrap();
        assert_eq!(
            detail.team_totals(200),
            Some(TeamTotals {
                kills: 13,
                damage_to_champions: 40_000
            })
        );
        assert_eq!(detail.team_totals(300), None);
    }

    #[test]
    fn the_stats_combine_the_player_and_the_team_totals() {
        let detail: GameDetail = serde_json::from_str(DETAIL).unwrap();
        let past = past_games().remove(0);
        let totals = detail.team_totals(past.team_id());
        assert_eq!(
            past.clone().with_team_totals(totals).stats,
            Some(PlayerStats {
                cs: 32,
                vision_score: 80,
                damage_to_champions: 9_000,
                team_kills: 13,
                team_damage_to_champions: 40_000,
            })
        );
        // Without the detail the game has no stats, so no workout.
        assert_eq!(past.with_team_totals(None).stats, None);
    }

    #[test]
    fn maps_lanes_to_roles() {
        let role = |lane: &str, role: &str, smite: bool| {
            role_of(
                &Timeline {
                    lane: lane.to_owned(),
                    role: role.to_owned(),
                },
                smite,
            )
        };
        assert_eq!(role("TOP", "SOLO", false), Some(Role::Top));
        assert_eq!(role("JUNGLE", "NONE", true), Some(Role::Jungle));
        // Riot sometimes calls a top laner a jungler: no Smite, no jungle.
        assert_eq!(role("JUNGLE", "NONE", false), Some(Role::Top));
        assert_eq!(role("MIDDLE", "SOLO", false), Some(Role::Mid));
        assert_eq!(role("BOTTOM", "CARRY", false), Some(Role::Bottom));
        assert_eq!(role("BOTTOM", "SUPPORT", false), Some(Role::Support));
        assert_eq!(role("NONE", "DUO", false), None);
    }
}
