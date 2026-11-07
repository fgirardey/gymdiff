//! The [`LolClient`] trait: the only way the app reads game data.

use std::future::Future;

use gymdiff_core::GamePhase;

use crate::LolError;
use crate::live_client::GameStats;

/// Read access to the League client. Implemented by [`crate::HttpLolClient`] for both
/// the real client and the simulator; tests can provide in-memory fakes.
pub trait LolClient: Send + Sync {
    /// `GET /lol-gameflow/v1/gameflow-phase` (LCU).
    fn gameflow_phase(&self) -> impl Future<Output = Result<GamePhase, LolError>> + Send;

    /// `GET /liveclientdata/gamestats`. `Ok(None)` when no game is running.
    fn game_stats(&self) -> impl Future<Output = Result<Option<GameStats>, LolError>> + Send;
}
