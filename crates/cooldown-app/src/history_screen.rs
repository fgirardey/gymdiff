//! Keeps the History screen up to date: reads the stored games of a player, builds the sessions
//! (`history_view`), shows them, then downloads the champion images that are missing and shows
//! them again once they are there (ADR-0004).

use chrono::{Local, NaiveDateTime, TimeZone};
use slint::Weak;

use crate::AppWindow;
use crate::champion_catalog::ChampionCatalog;
use crate::history_refresh::HistoryRefresh;
use crate::history_store::HistoryStore;
use crate::history_view::{SessionView, session_views};

/// What it takes to refresh the screen. Cheap to clone.
#[derive(Clone)]
pub struct HistoryScreen {
    store: HistoryStore,
    champions: ChampionCatalog,
    refresh: HistoryRefresh,
    window: Weak<AppWindow>,
}

impl HistoryScreen {
    pub fn new(store: HistoryStore, champions: ChampionCatalog, window: Weak<AppWindow>) -> Self {
        Self {
            store,
            champions,
            refresh: HistoryRefresh::new(window.clone()),
            window,
        }
    }

    pub fn store(&self) -> &HistoryStore {
        &self.store
    }

    /// The refresh button and the "Updated … ago" label.
    pub fn refresh(&self) -> &HistoryRefresh {
        &self.refresh
    }

    /// The history of the player of the most recent stored game: what to show before the client
    /// says who is logged in.
    pub async fn show_last_player(&self) {
        let store = self.store.clone();
        match tokio::task::spawn_blocking(move || store.last_puuid()).await {
            Ok(Ok(Some(puuid))) => self.show(&puuid).await,
            Ok(Ok(None)) => {}
            Ok(Err(error)) => tracing::warn!(%error, "cannot read the history"),
            Err(error) => tracing::debug!(%error, "history read interrupted"),
        }
    }

    /// Shows the games of `puuid`, with the images already cached, then fetches the missing
    /// champion names and images and shows them again when something new arrived.
    pub async fn show(&self, puuid: &str) {
        let Some(games) = self.read_games(puuid).await else {
            return;
        };
        self.push(&games);

        let ids: Vec<u32> = games.iter().map(|game| game.champion_id).collect();
        let missing = ids.iter().any(|id| {
            self.champions
                .info(*id)
                .is_none_or(|champion| champion.image.is_none())
        });
        if !missing {
            return;
        }
        match self.champions.refresh(&ids).await {
            Ok(true) => self.push(&games),
            Ok(false) => {}
            // Offline is normal: the initials stay until the next time.
            Err(error) => tracing::debug!(%error, "cannot download the champion images"),
        }
    }

    async fn read_games(&self, puuid: &str) -> Option<Vec<cooldown_core::Game>> {
        let (store, puuid) = (self.store.clone(), puuid.to_owned());
        match tokio::task::spawn_blocking(move || store.games(&puuid)).await {
            Ok(Ok(games)) => Some(games),
            Ok(Err(error)) => {
                tracing::warn!(%error, "cannot read the history");
                None
            }
            Err(error) => {
                tracing::debug!(%error, "history read interrupted");
                None
            }
        }
    }

    fn push(&self, games: &[cooldown_core::Game]) {
        let now = Local::now().timestamp();
        let champions = self.champions.clone();
        let views = session_views(games, now, &local_time, &|id| champions.info(id));
        let result = self.window.upgrade_in_event_loop(move |window| {
            window.set_sessions(SessionView::to_slint(&views, &window));
        });
        if result.is_err() {
            tracing::debug!("UI event loop is gone");
        }
    }
}

/// A Unix time as the player's local date and time.
fn local_time(seconds: i64) -> NaiveDateTime {
    Local.timestamp_opt(seconds, 0).single().map_or_else(
        || chrono::DateTime::UNIX_EPOCH.naive_utc(),
        |time| time.naive_local(),
    )
}
