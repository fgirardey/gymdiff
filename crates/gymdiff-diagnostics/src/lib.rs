//! # gymdiff-diagnostics
//!
//! Building blocks for the opt-in error reports described in
//! `docs/Decisions/ADR-0006 Opt-in error reports.md`.
//!
//! Nothing here touches the network. Everything is scrubbed *when captured*,
//! so raw text never sits in a report.

mod capture;
mod consent;
mod report;
mod scrub;

pub use capture::{CaptureLayer, ErrorBuffer};
pub use consent::{Consent, ConsentStore};
pub use report::{Entry, Report};
pub use scrub::scrub;
