//! # cooldown-diagnostics
//!
//! Building blocks for the opt-in reports described in
//! `docs/Decisions/ADR-0006 Opt-in error reports.md`: the player's consent and
//! the scrubber that removes personal data before anything leaves the PC.
//!
//! Nothing here touches the network.

mod consent;
mod scrub;

pub use consent::{Consent, ConsentStore};
pub use scrub::scrub;
