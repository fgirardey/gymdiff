//! The game history, in one SQLite database (`history.sqlite`) in the app's data folder
//! ([`crate::platform::data_dir`]). See `docs/Decisions/ADR-0007 Local persistence.md`.
//!
//! The file is never recreated or overwritten when it cannot be read: [`HistoryStore::open`]
//! fails and the app goes on without a history (`docs/Product/Features/History.md`).

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use gymdiff_core::{Game, GameResult, GameSource, Role};
use rusqlite::{Connection, params};

/// Handle on the history database. Cheap to clone: every clone shares one connection, so there
/// is one writer, and calls block briefly (call from `spawn_blocking` in async code).
#[derive(Debug, Clone)]
pub struct HistoryStore {
    connection: Arc<Mutex<Connection>>,
}

impl HistoryStore {
    /// Opens `<data_dir>/history.sqlite`, creating it and migrating its schema when needed.
    ///
    /// # Errors
    /// The file cannot be opened or read (damaged, locked, no disk access), or it was written by
    /// a newer GymDiff. The file is left untouched.
    pub fn open(data_dir: &Path) -> Result<Self, HistoryError> {
        std::fs::create_dir_all(data_dir)?;
        Self::from_connection(Connection::open(data_dir.join(FILE_NAME))?)
    }

    /// A throwaway database in memory, for tests.
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, HistoryError> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(connection: Connection) -> Result<Self, HistoryError> {
        // Read first: switching to WAL already writes to the file, which a database from a newer
        // GymDiff (or a damaged one) must never get.
        let found = schema_version(&connection)?;
        if found > SCHEMA_VERSION {
            return Err(HistoryError::TooNew {
                found,
                supported: SCHEMA_VERSION,
            });
        }
        // WAL: a crash never corrupts the file and readers do not block the writer. In memory it
        // answers "memory", which is fine.
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "NORMAL")?;
        migrate(&connection, found)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    /// Stores `games` of `puuid` that are not stored yet and returns how many were new. A game
    /// already stored (same player and game id) is never replaced: a game GymDiff tracked keeps
    /// its workout when the client's match history lists it again.
    ///
    /// # Errors
    /// The database cannot be written. Nothing is stored then (one transaction).
    pub fn import_games(&self, puuid: &str, games: &[Game]) -> Result<usize, HistoryError> {
        let mut connection = self.lock();
        let transaction = connection.transaction()?;
        let mut inserted = 0;
        {
            let mut insert = transaction.prepare_cached(
                "INSERT OR IGNORE INTO games
                    (puuid, game_id, source, started_at, duration_secs, queue_id, champion_id,
                     role, result, kills, deaths, assists)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            )?;
            for game in games {
                inserted += insert.execute(params![
                    puuid,
                    // SQLite integers are signed 64 bits; game ids are far below 2^63.
                    i64::try_from(game.game_id).unwrap_or(i64::MAX),
                    source_code(game.source),
                    game.started_at,
                    game.duration_secs,
                    game.queue_id,
                    game.champion_id,
                    game.role.map(role_code),
                    result_code(game.result),
                    game.kills,
                    game.deaths,
                    game.assists,
                ])?;
            }
        }
        transaction.commit()?;
        Ok(inserted)
    }

    /// The stored games of `puuid`, newest first. A row this version cannot understand (written
    /// by a newer one) is skipped, not an error.
    ///
    /// # Errors
    /// The database cannot be read.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "read by the History screen, the next step")
    )]
    pub fn games(&self, puuid: &str) -> Result<Vec<Game>, HistoryError> {
        let connection = self.lock();
        let mut select = connection.prepare_cached(
            "SELECT game_id, source, started_at, duration_secs, queue_id, champion_id,
                    role, result, kills, deaths, assists
             FROM games WHERE puuid = ?1 ORDER BY started_at DESC, game_id DESC",
        )?;
        let rows = select.query_map([puuid], |row| {
            Ok(StoredGame {
                game_id: row.get(0)?,
                source: row.get(1)?,
                started_at: row.get(2)?,
                duration_secs: row.get(3)?,
                queue_id: row.get(4)?,
                champion_id: row.get(5)?,
                role: row.get(6)?,
                result: row.get(7)?,
                kills: row.get(8)?,
                deaths: row.get(9)?,
                assists: row.get(10)?,
            })
        })?;
        let mut games = Vec::new();
        for row in rows {
            if let Some(game) = row?.into_game() {
                games.push(game);
            } else {
                tracing::debug!("skipped a stored game this version cannot read");
            }
        }
        Ok(games)
    }

    fn lock(&self) -> MutexGuard<'_, Connection> {
        // A panic while holding the lock leaves the connection usable: SQLite rolled back.
        self.connection
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    #[error("history database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("cannot prepare the data folder: {0}")]
    Io(#[from] std::io::Error),

    /// The file comes from a newer GymDiff: opening it could lose data on the next migration.
    #[error(
        "history database version {found} is newer than this GymDiff understands ({supported})"
    )]
    TooNew { found: u32, supported: u32 },
}

/// File name, in the data folder.
const FILE_NAME: &str = "history.sqlite";

/// One entry per schema version: entry `n` migrates the database from version `n` to `n + 1`.
/// Append only: a released migration is never edited.
const MIGRATIONS: &[&str] = &[
    // 1: the games of a player, as listed by History. Tracked-only columns (linked champion,
    // formula stats, amounts, workout status) come with the post-game workout.
    "CREATE TABLE games (
        puuid         TEXT    NOT NULL,
        game_id       INTEGER NOT NULL,
        source        TEXT    NOT NULL,
        started_at    INTEGER NOT NULL,
        duration_secs INTEGER NOT NULL,
        queue_id      INTEGER NOT NULL,
        champion_id   INTEGER NOT NULL,
        role          TEXT,
        result        TEXT    NOT NULL,
        kills         INTEGER NOT NULL,
        deaths        INTEGER NOT NULL,
        assists       INTEGER NOT NULL,
        PRIMARY KEY (puuid, game_id)
    ) WITHOUT ROWID;
    CREATE INDEX games_by_time ON games (puuid, started_at DESC);",
];

fn schema_version(connection: &Connection) -> Result<u32, HistoryError> {
    Ok(connection.query_row("PRAGMA user_version", [], |row| row.get(0))?)
}

/// The schema version of this GymDiff: the number of migrations.
#[allow(clippy::cast_possible_truncation)] // a handful of migrations
const SCHEMA_VERSION: u32 = MIGRATIONS.len() as u32;

/// Brings the schema from version `found` up to date, each step in its own transaction with its
/// version bump.
fn migrate(connection: &Connection, found: u32) -> Result<(), HistoryError> {
    for (version, sql) in MIGRATIONS.iter().enumerate().skip(found as usize) {
        connection.execute_batch(&format!(
            "BEGIN; {sql}\nPRAGMA user_version = {}; COMMIT;",
            version + 1
        ))?;
    }
    Ok(())
}

/// A row as stored; codes are turned into [`Game`] values in [`StoredGame::into_game`].
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "read by the History screen, the next step")
)]
struct StoredGame {
    game_id: i64,
    source: String,
    started_at: i64,
    duration_secs: u32,
    queue_id: u32,
    champion_id: u32,
    role: Option<String>,
    result: String,
    kills: u32,
    deaths: u32,
    assists: u32,
}

impl StoredGame {
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "read by the History screen, the next step")
    )]
    fn into_game(self) -> Option<Game> {
        Some(Game {
            game_id: u64::try_from(self.game_id).ok()?,
            source: match self.source.as_str() {
                "tracked" => GameSource::Tracked,
                "imported" => GameSource::Imported,
                _ => return None,
            },
            started_at: self.started_at,
            duration_secs: self.duration_secs,
            queue_id: self.queue_id,
            champion_id: self.champion_id,
            // An unknown role is not worth losing the game: the role is only decoration.
            role: self.role.as_deref().and_then(role_from_code),
            result: match self.result.as_str() {
                "win" => GameResult::Win,
                "loss" => GameResult::Loss,
                "remake" => GameResult::Remake,
                _ => return None,
            },
            kills: self.kills,
            deaths: self.deaths,
            assists: self.assists,
        })
    }
}

// The codes below are stored in the database: never rename one.

fn source_code(source: GameSource) -> &'static str {
    match source {
        GameSource::Tracked => "tracked",
        GameSource::Imported => "imported",
    }
}

fn result_code(result: GameResult) -> &'static str {
    match result {
        GameResult::Win => "win",
        GameResult::Loss => "loss",
        GameResult::Remake => "remake",
    }
}

fn role_code(role: Role) -> &'static str {
    match role {
        Role::Top => "top",
        Role::Jungle => "jungle",
        Role::Mid => "mid",
        Role::Bottom => "bottom",
        Role::Support => "support",
    }
}

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "read by the History screen, the next step")
)]
fn role_from_code(code: &str) -> Option<Role> {
    match code {
        "top" => Some(Role::Top),
        "jungle" => Some(Role::Jungle),
        "mid" => Some(Role::Mid),
        "bottom" => Some(Role::Bottom),
        "support" => Some(Role::Support),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(game_id: u64, started_at: i64) -> Game {
        Game {
            game_id,
            source: GameSource::Imported,
            started_at,
            duration_secs: 1_800,
            queue_id: 420,
            champion_id: 103,
            role: Some(Role::Support),
            result: GameResult::Win,
            kills: 2,
            deaths: 4,
            assists: 17,
        }
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("gymdiff-history-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_stored_game_reads_back_unchanged() {
        let store = HistoryStore::open_in_memory().unwrap();
        let remake = Game {
            result: GameResult::Remake,
            role: None,
            ..game(2, 500)
        };
        store
            .import_games("p1", &[game(1, 100), remake.clone()])
            .unwrap();
        assert_eq!(store.games("p1").unwrap(), [remake, game(1, 100)]);
    }

    #[test]
    fn games_come_newest_first() {
        let store = HistoryStore::open_in_memory().unwrap();
        store
            .import_games("p1", &[game(1, 100), game(3, 300), game(2, 200)])
            .unwrap();
        let ids: Vec<u64> = store
            .games("p1")
            .unwrap()
            .iter()
            .map(|g| g.game_id)
            .collect();
        assert_eq!(ids, [3, 2, 1]);
    }

    #[test]
    fn importing_twice_stores_each_game_once() {
        let store = HistoryStore::open_in_memory().unwrap();
        assert_eq!(
            store
                .import_games("p1", &[game(1, 100), game(2, 200)])
                .unwrap(),
            2
        );
        assert_eq!(
            store
                .import_games("p1", &[game(2, 200), game(3, 300)])
                .unwrap(),
            1
        );
        assert_eq!(store.games("p1").unwrap().len(), 3);
    }

    #[test]
    fn an_imported_copy_never_replaces_a_tracked_game() {
        let store = HistoryStore::open_in_memory().unwrap();
        let tracked = Game {
            source: GameSource::Tracked,
            ..game(1, 100)
        };
        store
            .import_games("p1", std::slice::from_ref(&tracked))
            .unwrap();
        assert_eq!(store.import_games("p1", &[game(1, 100)]).unwrap(), 0);
        assert_eq!(store.games("p1").unwrap(), [tracked]);
    }

    #[test]
    fn players_do_not_see_each_others_games() {
        let store = HistoryStore::open_in_memory().unwrap();
        store.import_games("p1", &[game(1, 100)]).unwrap();
        store
            .import_games("p2", &[game(1, 100), game(2, 200)])
            .unwrap();
        assert_eq!(store.games("p1").unwrap().len(), 1);
        assert_eq!(store.games("p2").unwrap().len(), 2);
        assert_eq!(store.games("nobody").unwrap(), []);
    }

    #[test]
    fn the_history_survives_reopening_the_file() {
        let dir = temp_dir("reopen");
        HistoryStore::open(&dir)
            .unwrap()
            .import_games("p1", &[game(1, 100)])
            .unwrap();
        let reopened = HistoryStore::open(&dir).unwrap();
        assert_eq!(reopened.games("p1").unwrap(), [game(1, 100)]);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_file_from_a_newer_version_is_refused_and_left_alone() {
        let dir = temp_dir("newer");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(FILE_NAME);
        Connection::open(&path)
            .unwrap()
            .pragma_update(None, "user_version", 99)
            .unwrap();
        let before = std::fs::read(&path).unwrap();
        let error = HistoryStore::open(&dir).unwrap_err();
        assert!(
            matches!(error, HistoryError::TooNew { found: 99, .. }),
            "{error}"
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_damaged_file_is_refused_and_left_alone() {
        let dir = temp_dir("damaged");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(FILE_NAME);
        std::fs::write(
            &path,
            b"this is not a database, but it is long enough to be read as one",
        )
        .unwrap();
        assert!(HistoryStore::open(&dir).is_err());
        assert_eq!(
            std::fs::read(&path).unwrap(),
            b"this is not a database, but it is long enough to be read as one"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_row_this_version_cannot_read_is_skipped() {
        let store = HistoryStore::open_in_memory().unwrap();
        store.import_games("p1", &[game(1, 100)]).unwrap();
        store
            .lock()
            .execute("UPDATE games SET result = 'draw' WHERE game_id = 1", [])
            .unwrap();
        store.import_games("p1", &[game(2, 200)]).unwrap();
        let ids: Vec<u64> = store
            .games("p1")
            .unwrap()
            .iter()
            .map(|g| g.game_id)
            .collect();
        assert_eq!(ids, [2]);
    }
}
