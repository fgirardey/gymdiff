//! The [`LolClient`] trait: the only way the app reads game data.

use std::future::Future;

use gymdiff_core::GamePhase;

use crate::LolError;
use crate::live_client::GameStats;
use crate::match_history::{PastGame, TeamTotals};
use crate::summoner::Summoner;

/// Read access to the League client. Implemented by [`crate::HttpLolClient`] for both
/// the real client and the simulator; tests can provide in-memory fakes.
pub trait LolClient: Send + Sync {
    /// `GET /lol-gameflow/v1/gameflow-phase` (LCU).
    fn gameflow_phase(&self) -> impl Future<Output = Result<GamePhase, LolError>> + Send;

    /// `GET /liveclientdata/gamestats`. `Ok(None)` when no game is running.
    fn game_stats(&self) -> impl Future<Output = Result<Option<GameStats>, LolError>> + Send;

    /// `GET /lol-summoner/v1/current-summoner` (LCU). `Ok(None)` while nobody is logged in.
    fn current_summoner(&self) -> impl Future<Output = Result<Option<Summoner>, LolError>> + Send;

    /// `GET /lol-match-history/v1/products/lol/current-summoner/matches` (LCU): the games at
    /// positions `beg_index..end_index` of the active summoner's history, newest first. An empty
    /// list means there are no more games (or no history yet).
    fn match_history(
        &self,
        beg_index: u32,
        end_index: u32,
    ) -> impl Future<Output = Result<Vec<PastGame>, LolError>> + Send;

    /// `GET /lol-match-history/v1/games/{gameId}` (LCU): the sums of the kills and damage of
    /// `team_id` in a game. `Ok(None)` when the client has no detail for it.
    fn team_totals(
        &self,
        game_id: u64,
        team_id: u32,
    ) -> impl Future<Output = Result<Option<TeamTotals>, LolError>> + Send;
}
