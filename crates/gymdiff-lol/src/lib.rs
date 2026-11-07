//! # gymdiff-lol
//!
//! Access to the two local APIs exposed by League of Legends:
//!
//! - **LCU API** (League Client Update): REST + WebSocket, exposed by the
//!   launcher/client on a random port. Credentials come from the `lockfile`.
//! - **Live Client Data API**: read-only REST API exposed by the game process
//!   on `https://127.0.0.1:2999` while a match is running.
//!
//! See `docs/Architecture/League APIs.md` for details.
//!
//! Everything else in the app talks to the game through the [`LolClient`]
//! trait, so the real client and the simulator are interchangeable.

mod client;
mod error;
mod http_client;
pub mod live_client;
pub mod lockfile;

pub use client::LolClient;
pub use error::LolError;
pub use http_client::{DEFAULT_LIVE_CLIENT_URL, HttpLolClient};
pub use lockfile::Lockfile;
