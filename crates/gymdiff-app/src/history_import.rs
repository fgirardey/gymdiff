//! Fills the history with the games the League client already knows (ADR-0014): the active
//! summoner's recent games, read from the client's match history, stored as imported games.

use std::time::Duration;

use anyhow::Context;
use gymdiff_lol::LolClient;

use crate::history_store::HistoryStore;

/// Imports the past games as soon as the client can give them, retrying every [`RETRY_DELAY`]
/// until it does. The client answers before the player is logged in (and fails in other ways
/// while it starts), so a single attempt at connection time may find nothing. Returns how many
/// games were new. Runs for as long as it takes: the caller stops it when the client goes away.
pub async fn import_when_ready(client: &impl LolClient, store: &HistoryStore) -> usize {
    loop {
        match import_past_games(client, store).await {
            Ok(Some(imported)) => return imported,
            Ok(None) => tracing::debug!("nobody is logged into the client yet: will retry"),
            Err(error) => tracing::debug!(%error, "cannot import the past games yet: will retry"),
        }
        tokio::time::sleep(RETRY_DELAY).await;
    }
}

/// Reads the active summoner's past games from `client` and stores those not stored yet.
/// Returns how many were new, or `None` while nobody is logged into the client.
///
/// It pages through the match history, a handful of local requests.
async fn import_past_games(
    client: &impl LolClient,
    store: &HistoryStore,
) -> anyhow::Result<Option<usize>> {
    let Some(summoner) = client
        .current_summoner()
        .await
        .context("reading the active summoner")?
    else {
        return Ok(None);
    };

    let mut imported = 0;
    for beg_index in (0..MAX_GAMES).step_by(PAGE_SIZE as usize) {
        let games = match client.match_history(beg_index, beg_index + PAGE_SIZE).await {
            Ok(games) => games,
            // The client may refuse to go further back than it keeps: keep what we have.
            Err(error) if beg_index > 0 => {
                tracing::debug!(%error, beg_index, "match history stops here");
                break;
            }
            Err(error) => return Err(error).context("reading the match history"),
        };
        if games.is_empty() {
            break;
        }
        let full_page = games.len() >= PAGE_SIZE as usize;

        let store = store.clone();
        let puuid = summoner.puuid.clone();
        imported += tokio::task::spawn_blocking(move || store.import_games(&puuid, &games))
            .await
            .context("storing the past games")?
            .context("storing the past games")?;

        if !full_page {
            break;
        }
    }
    Ok(Some(imported))
}

/// Wait between two attempts while the client cannot give the games yet.
const RETRY_DELAY: Duration = Duration::from_secs(10);

/// Games asked per request.
const PAGE_SIZE: u32 = 20;

/// Most games read per connection. What the client really keeps is to be checked on Windows
/// (`docs/Product/Features/History.md`, open questions).
const MAX_GAMES: u32 = 200;

#[cfg(test)]
mod tests {
    use gymdiff_core::{Game, GamePhase, GameResult, GameSource};
    use gymdiff_lol::live_client::GameStats;
    use gymdiff_lol::{LolError, Summoner};

    use super::*;

    /// A client with `count` games, newest first, that can fail from a given index on.
    struct FakeClient {
        summoner: Option<Summoner>,
        count: u32,
        fail_from: Option<u32>,
    }

    impl FakeClient {
        fn with_games(count: u32) -> Self {
            Self {
                summoner: Some(Summoner {
                    puuid: "p1".to_owned(),
                    game_name: "Test".to_owned(),
                    tag_line: "T".to_owned(),
                }),
                count,
                fail_from: None,
            }
        }
    }

    // A fake answers at once: its async methods never await.
    #[allow(clippy::unused_async_trait_impl)]
    impl LolClient for FakeClient {
        async fn gameflow_phase(&self) -> Result<GamePhase, LolError> {
            Ok(GamePhase::default())
        }

        async fn game_stats(&self) -> Result<Option<GameStats>, LolError> {
            Ok(None)
        }

        async fn current_summoner(&self) -> Result<Option<Summoner>, LolError> {
            Ok(self.summoner.clone())
        }

        async fn match_history(&self, beg: u32, end: u32) -> Result<Vec<Game>, LolError> {
            if self.fail_from.is_some_and(|from| beg >= from) {
                return Err(LolError::UnexpectedStatus {
                    status: 400,
                    url: "matches".to_owned(),
                });
            }
            Ok((beg..end.min(self.count)).map(game).collect())
        }
    }

    /// Game `index` of the history: the lower the index, the newer.
    fn game(index: u32) -> Game {
        Game {
            game_id: 1_000 + u64::from(index),
            source: GameSource::Imported,
            started_at: 1_000_000 - i64::from(index) * 4_000,
            duration_secs: 1_800,
            queue_id: 420,
            champion_id: 103,
            role: None,
            result: GameResult::Win,
            kills: 1,
            deaths: 2,
            assists: 3,
        }
    }

    #[tokio::test]
    async fn imports_every_page_until_the_history_ends() {
        let store = HistoryStore::open_in_memory().unwrap();
        let imported = import_past_games(&FakeClient::with_games(45), &store)
            .await
            .unwrap();
        assert_eq!(imported, Some(45));
        assert_eq!(store.games("p1").unwrap().len(), 45);
    }

    #[tokio::test]
    async fn importing_again_adds_only_new_games() {
        let store = HistoryStore::open_in_memory().unwrap();
        import_past_games(&FakeClient::with_games(30), &store)
            .await
            .unwrap();
        let imported = import_past_games(&FakeClient::with_games(33), &store)
            .await
            .unwrap();
        assert_eq!(imported, Some(3));
        assert_eq!(store.games("p1").unwrap().len(), 33);
    }

    #[tokio::test]
    async fn stops_at_the_cap() {
        let store = HistoryStore::open_in_memory().unwrap();
        let imported = import_past_games(&FakeClient::with_games(500), &store)
            .await
            .unwrap();
        assert_eq!(imported, Some(MAX_GAMES as usize));
    }

    #[tokio::test]
    async fn keeps_earlier_pages_when_the_client_refuses_to_go_back_further() {
        let store = HistoryStore::open_in_memory().unwrap();
        let client = FakeClient {
            fail_from: Some(20),
            ..FakeClient::with_games(100)
        };
        assert_eq!(import_past_games(&client, &store).await.unwrap(), Some(20));
    }

    #[tokio::test]
    async fn nobody_logged_in_imports_nothing() {
        let store = HistoryStore::open_in_memory().unwrap();
        let client = FakeClient {
            summoner: None,
            ..FakeClient::with_games(10)
        };
        assert_eq!(import_past_games(&client, &store).await.unwrap(), None);
    }

    #[tokio::test]
    async fn a_failing_first_page_is_an_error() {
        let store = HistoryStore::open_in_memory().unwrap();
        let client = FakeClient {
            fail_from: Some(0),
            ..FakeClient::with_games(10)
        };
        assert!(import_past_games(&client, &store).await.is_err());
    }

    /// Fails like a client that is not ready, then answers.
    struct LateClient {
        inner: FakeClient,
        failures_left: std::sync::atomic::AtomicU32,
    }

    #[allow(clippy::unused_async_trait_impl)]
    impl LolClient for LateClient {
        async fn gameflow_phase(&self) -> Result<GamePhase, LolError> {
            self.inner.gameflow_phase().await
        }

        async fn game_stats(&self) -> Result<Option<GameStats>, LolError> {
            self.inner.game_stats().await
        }

        async fn current_summoner(&self) -> Result<Option<Summoner>, LolError> {
            use std::sync::atomic::Ordering;
            if self.failures_left.load(Ordering::SeqCst) > 0 {
                self.failures_left.fetch_sub(1, Ordering::SeqCst);
                return Ok(None);
            }
            self.inner.current_summoner().await
        }

        async fn match_history(&self, beg: u32, end: u32) -> Result<Vec<Game>, LolError> {
            self.inner.match_history(beg, end).await
        }
    }

    #[tokio::test(start_paused = true)]
    async fn retries_until_somebody_is_logged_in() {
        let store = HistoryStore::open_in_memory().unwrap();
        let client = LateClient {
            inner: FakeClient::with_games(5),
            failures_left: std::sync::atomic::AtomicU32::new(3),
        };
        assert_eq!(import_when_ready(&client, &store).await, 5);
        assert_eq!(store.games("p1").unwrap().len(), 5);
    }
}
