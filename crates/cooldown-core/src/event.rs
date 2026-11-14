//! Events: everything a data source can report to the reducer
//! ([`crate::CompanionState::apply`]).

use crate::GamePhase;

/// Something that happened in the outside world.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    ClientConnected,
    ClientDisconnected,
    ClientError(String),
    PhaseChanged(GamePhase),
    /// In-game clock, in seconds, from the Live Client Data API.
    GameTimeUpdated(f64),
}
