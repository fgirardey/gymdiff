//! Auto-update of alpha builds (`docs/Product/Features/Auto-update.md`,
//! ADR-0010).

mod install;
mod manifest;
mod releases;
mod service;
mod state;

pub use service::Updater;
