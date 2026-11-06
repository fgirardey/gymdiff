//! # gymdiff-core
//!
//! Platform-agnostic domain logic for GymDiff.
//!
//! Rules for this crate (see `docs/Architecture/Overview.md`):
//! - no I/O (network, filesystem, processes),
//! - no OS-specific code (`#[cfg(target_os = ...)]` is not allowed here),
//! - everything must be unit-testable on any machine.

pub mod phase;
pub mod state;

pub use phase::GamePhase;
pub use state::{CompanionState, ConnectionStatus, Event};
