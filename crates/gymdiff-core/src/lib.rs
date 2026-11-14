//! # gymdiff-core
//!
//! Platform-agnostic domain logic for GymDiff.
//!
//! Rules for this crate (see `docs/Architecture/Overview.md`):
//! - no I/O (network, filesystem, processes),
//! - no OS-specific code (`#[cfg(target_os = ...)]` is not allowed here),
//! - everything must be unit-testable on any machine.

pub mod event;
pub mod game;
pub mod phase;
pub mod session;
pub mod state;
pub mod workout;

pub use event::Event;
pub use game::{Game, GameResult, GameSource, NoWorkoutReason, PlayerStats, Role};
pub use phase::GamePhase;
pub use session::{Session, group_into_sessions};
pub use state::{CompanionState, ConnectionStatus};
pub use workout::{Amount, Workout};
