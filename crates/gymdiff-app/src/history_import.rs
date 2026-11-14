//! Fills the history with the games the League client already knows (ADR-0014): the active
//! summoner's recent games, read from the client's match history, with the workout they would
//! have given, stored as imported games.

use std::collections::HashSet;
use std::time::Duration;

use anyhow::Context;
use gymdiff_core::workout;
use gymdiff_lol::LolClient;

use crate::history_store::HistoryStore;

/// Imports the past games as soon as the client can give them, retrying every [`RETRY_DELAY`]
/// until it does. The client answers before the player is logged in (and fails in other ways
/// while it starts), so a single attempt at connection time may find nothing. Returns how many
/// games were stored, and for whom. Runs for as long as it takes: the caller stops it when the
/// client goes away.
pub async fn import_when_ready(client: &impl LolClient, store: &HistoryStore) -> Imported {
    loop {
        match import_past_games(client, store).await {
            Ok(Some(imported)) => return imported,
            Ok(None) => tracing::debug!("nobody is logged into the client yet: will retry"),
            Err(error) => tracing::debug!(%error, "cannot import the past games yet: will retry"),
        }
        tokio::time::sleep(RETRY_DELAY).await;
    }
}

/// The result of an import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imported {
    /// The active summoner: whose history to show.
    pub puuid: String,
    /// Games stored or completed.
    pub stored: usize,
}

/// Reads the active summoner's past games from `client` and stores those that are new or still
/// miss their stats. Returns `None` while nobody is logged in.
///
/// It pages through the match history and reads the detail of each game it needs (for the team's
/// kills and damage): a few dozen local requests the first time, none for games already stored.
async fn import_past_games(
    client: &impl LolClient,
    store: &HistoryStore,
) -> anyhow::Result<Option<Imported>> {
    let Some(summoner) = client
        .current_summoner()
        .await
        .context("reading the active summoner")?
    else {
        return Ok(None);
    };
    let puuid = summoner.puuid;

    let complete = {
        let (store, puuid) = (store.clone(), puuid.clone());
        tokio::task::spawn_blocking(move || store.ids_with_stats(&puuid))
            .await
            .context("reading the stored games")?
            .context("reading the stored games")?
    };

    let mut seen = HashSet::new();
    let mut stored = 0;
    for beg_index in (0..MAX_GAMES).step_by(PAGE_SIZE as usize) {
        let page = match client.match_history(beg_index, beg_index + PAGE_SIZE).await {
            Ok(page) => page,
            // The client may refuse to go further back than it keeps: keep what we have.
            Err(error) if beg_index > 0 => {
                tracing::debug!(%error, beg_index, "match history stops here");
                break;
            }
            Err(error) => return Err(error).context("reading the match history"),
        };
        // The client may ignore the indices and answer the same games again.
        let new_in_page = page
            .iter()
            .filter(|past| seen.insert(past.game().game_id))
            .count();
        if new_in_page == 0 {
            break;
        }
        let full_page = page.len() >= PAGE_SIZE as usize;

        let mut games = Vec::new();
        for past in page {
            // Games of the other modes are not shown, so there is no point in storing them.
            if !past.game().is_in_scope() || complete.contains(&past.game().game_id) {
                continue;
            }
            let totals = match client
                .team_totals(past.game().game_id, past.team_id())
                .await
            {
                Ok(totals) => totals,
                Err(error) => {
                    tracing::debug!(%error, game_id = past.game().game_id, "cannot read the game's detail");
                    None
                }
            };
            let mut game = past.with_team_totals(totals);
            game.workout = workout::compute(&game);
            games.push(game);
        }

        let (store, puuid) = (store.clone(), puuid.clone());
        stored += tokio::task::spawn_blocking(move || store.import_games(&puuid, &games))
            .await
            .context("storing the past games")?
            .context("storing the past games")?;

        if !full_page {
            break;
        }
    }
    Ok(Some(Imported { puuid, stored }))
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
    use std::sync::atomic::{AtomicU32, Ordering};

    use gymdiff_core::GamePhase;
    use gymdiff_lol::live_client::GameStats;
    use gymdiff_lol::{LolError, MatchHistory, PastGame, Summoner, TeamTotals};

    use super::*;

    /// A client with `count` games, newest first.
    struct FakeClient {
        summoner: Option<Summoner>,
        count: u32,
        /// Answers every page request with the first page, like a client that ignores indices.
        ignores_indices: bool,
        fail_from: Option<u32>,
        details_readable: bool,
        detail_requests: AtomicU32,
        /// The queue of game `index` of the history.
        queue_of: fn(u32) -> u32,
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
                ignores_indices: false,
                fail_from: None,
                details_readable: true,
                detail_requests: AtomicU32::new(0),
                queue_of: |_| 420,
            }
        }

        fn detail_requests(&self) -> u32 {
            self.detail_requests.load(Ordering::SeqCst)
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

        async fn match_history(&self, beg: u32, end: u32) -> Result<Vec<PastGame>, LolError> {
            let beg = if self.ignores_indices { 0 } else { beg };
            let end = if self.ignores_indices { PAGE_SIZE } else { end };
            if self.fail_from.is_some_and(|from| beg >= from) {
                return Err(LolError::UnexpectedStatus {
                    status: 400,
                    url: "matches".to_owned(),
                });
            }
            Ok((beg..end.min(self.count))
                .map(|index| past_game(index, (self.queue_of)(index)))
                .collect())
        }

        async fn team_totals(&self, _: u64, _: u32) -> Result<Option<TeamTotals>, LolError> {
            self.detail_requests.fetch_add(1, Ordering::SeqCst);
            Ok(self.details_readable.then_some(TeamTotals {
                kills: 25,
                damage_to_champions: 80_000,
            }))
        }
    }

    /// Game `index` of the history, a bottom laner of `queue_id`: the lower the index, the newer.
    fn past_game(index: u32, queue_id: u32) -> PastGame {
        let json = serde_json::json!({
            "games": { "games": [{
                "gameId": 1_000 + u64::from(index),
                "gameCreation": (1_000_000 - i64::from(index) * 4_000) * 1_000,
                "gameDuration": 1_800,
                "queueId": queue_id,
                "participants": [{
                    "championId": 103,
                    "teamId": 100,
                    "stats": { "win": true, "kills": 1, "deaths": 2, "assists": 3,
                               "totalMinionsKilled": 150, "totalDamageDealtToChampions": 10_000,
                               "visionScore": 20 },
                    "timeline": { "lane": "BOTTOM", "role": "CARRY" }
                }]
            }] }
        });
        serde_json::from_value::<MatchHistory>(json)
            .unwrap()
            .into_games()
            .remove(0)
    }

    fn stored(imported: Option<Imported>) -> Option<usize> {
        imported.map(|imported| imported.stored)
    }

    #[tokio::test]
    async fn imports_every_page_with_its_workout() {
        let store = HistoryStore::open_in_memory().unwrap();
        let imported = import_past_games(&FakeClient::with_games(45), &store)
            .await
            .unwrap();
        assert_eq!(imported.map(|i| i.stored), Some(45));
        let games = store.games("p1").unwrap();
        assert_eq!(games.len(), 45);
        // Bottom, 30 min: 240 target CS, 150 CS = 90 missed / 5 = 18 burpees.
        assert!(
            games
                .iter()
                .all(|game| game.workout.is_some_and(|w| w.burpees.due == 18))
        );
    }

    #[tokio::test]
    async fn reads_the_detail_only_of_games_not_stored_yet() {
        let store = HistoryStore::open_in_memory().unwrap();
        import_past_games(&FakeClient::with_games(30), &store)
            .await
            .unwrap();
        let client = FakeClient::with_games(33);
        let imported = import_past_games(&client, &store).await.unwrap();
        assert_eq!(imported.map(|i| i.stored), Some(3));
        assert_eq!(client.detail_requests(), 3);
        assert_eq!(store.games("p1").unwrap().len(), 33);
    }

    #[tokio::test]
    async fn a_game_whose_detail_is_unreadable_is_stored_and_completed_later() {
        let store = HistoryStore::open_in_memory().unwrap();
        let blind = FakeClient {
            details_readable: false,
            ..FakeClient::with_games(2)
        };
        import_past_games(&blind, &store).await.unwrap();
        assert!(
            store
                .games("p1")
                .unwrap()
                .iter()
                .all(|g| g.stats.is_none() && g.workout.is_none())
        );

        let imported = import_past_games(&FakeClient::with_games(2), &store)
            .await
            .unwrap();
        assert_eq!(imported.map(|i| i.stored), Some(2));
        assert!(
            store
                .games("p1")
                .unwrap()
                .iter()
                .all(|g| g.workout.is_some())
        );
    }

    #[tokio::test]
    async fn games_of_the_other_modes_are_not_stored() {
        let store = HistoryStore::open_in_memory().unwrap();
        let client = FakeClient {
            queue_of: |index| if index % 2 == 0 { 420 } else { 450 },
            ..FakeClient::with_games(10)
        };
        let imported = import_past_games(&client, &store).await.unwrap();
        assert_eq!(imported.map(|i| i.stored), Some(5));
        assert_eq!(client.detail_requests(), 5);
        assert!(
            store
                .games("p1")
                .unwrap()
                .iter()
                .all(|game| game.queue_id == 420)
        );
    }

    #[tokio::test]
    async fn stops_at_the_cap() {
        let store = HistoryStore::open_in_memory().unwrap();
        let imported = import_past_games(&FakeClient::with_games(500), &store)
            .await
            .unwrap();
        assert_eq!(imported.map(|i| i.stored), Some(MAX_GAMES as usize));
    }

    #[tokio::test]
    async fn stops_when_the_client_ignores_the_indices() {
        let store = HistoryStore::open_in_memory().unwrap();
        let client = FakeClient {
            ignores_indices: true,
            ..FakeClient::with_games(500)
        };
        assert_eq!(
            stored(import_past_games(&client, &store).await.unwrap()),
            Some(20)
        );
    }

    #[tokio::test]
    async fn keeps_earlier_pages_when_the_client_refuses_to_go_back_further() {
        let store = HistoryStore::open_in_memory().unwrap();
        let client = FakeClient {
            fail_from: Some(20),
            ..FakeClient::with_games(100)
        };
        assert_eq!(
            stored(import_past_games(&client, &store).await.unwrap()),
            Some(20)
        );
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

    /// Answers "nobody is logged in" a few times, then behaves like its inner client.
    struct LateClient {
        inner: FakeClient,
        failures_left: AtomicU32,
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
            if self.failures_left.load(Ordering::SeqCst) > 0 {
                self.failures_left.fetch_sub(1, Ordering::SeqCst);
                return Ok(None);
            }
            self.inner.current_summoner().await
        }

        async fn match_history(&self, beg: u32, end: u32) -> Result<Vec<PastGame>, LolError> {
            self.inner.match_history(beg, end).await
        }

        async fn team_totals(
            &self,
            game_id: u64,
            team_id: u32,
        ) -> Result<Option<TeamTotals>, LolError> {
            self.inner.team_totals(game_id, team_id).await
        }
    }

    #[tokio::test(start_paused = true)]
    async fn retries_until_somebody_is_logged_in() {
        let store = HistoryStore::open_in_memory().unwrap();
        let client = LateClient {
            inner: FakeClient::with_games(5),
            failures_left: AtomicU32::new(3),
        };
        assert_eq!(import_when_ready(&client, &store).await.stored, 5);
        assert_eq!(store.games("p1").unwrap().len(), 5);
    }
}
