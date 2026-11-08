//! Installer integration and auto-update (`docs/Product/Features/Auto-update.md`,
//! ADR-0011), on Velopack.

mod service;
mod state;

pub use service::{Updater, run_installer_hooks};
