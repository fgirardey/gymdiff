---
status: proposed
date: 2026-11-08
---

# ADR-0007 Local persistence

## Context
cooldown.lol stores two kinds of data locally, with different needs:
- **Preferences** (welcome seen, later difficulty and targets): a few values, read at start, rarely written.
- **Game history** ([[History]]): grows with every game, is filtered by date and session, and must survive crashes.

So far preferences were spread over marker files (`welcome-seen`) and one JSON per feature (`diagnostics.json`).

## Decision
- **Preferences: one `settings.json`** in the data folder (`platform::data_dir`), a `Settings` struct with `serde`:
  - `version` field, to migrate;
  - every field has a default, a missing, unknown or unreadable file gives the defaults (never blocks the app);
  - writes are atomic (temporary file, then rename);
  - earlier marker files are migrated on read (`welcome-seen`).
- **Consent to error reports stays in `diagnostics.json`** ([[ADR-0006 Opt-in error reports]]): it is read by the pure `cooldown-diagnostics` crate and "corrupt means off" must hold, so a damaged settings file must never be able to switch reporting on.
- **Game history: SQLite through `rusqlite`** (feature `bundled`, so nothing to install on Windows), one `history.sqlite` in the data folder, WAL mode, one writer (one shared connection behind a mutex). Implemented in `cooldown-app/src/history_store.rs`:
  - **Migrations** are an append-only list of SQL scripts applied in order, the schema version being `PRAGMA user_version` (no extra crate for a handful of scripts). Each runs in its own transaction with its version bump. A released migration is never edited.
  - **Never destroyed**: a file that cannot be opened, or that carries a newer version than the app knows, makes `open` fail without writing anything (the version is read before the first write), and the app runs without a history. The file is never recreated or overwritten.
  - **Rows are keyed by player and game id** (`puuid`, `game_id`); inserting a game already stored is ignored, so a tracked game keeps its workout when the client's match history lists it again ([[ADR-0014 Import past games from the client]]).
  - A row this version cannot understand is skipped, not an error.
- Persistence stays in the app crate, behind small store types; `cooldown-core` stays pure and never touches the disk.

## Alternatives considered
- **One JSON file for everything, history included** — rewritten in full on every game, no queries, fragile on crash.
- **SQLite for preferences too** — a database for a handful of values; harder to inspect and edit by hand.
- **`sqlx`** — async and compile-time checked queries, but heavy for a local single-user database.
- **`sled` / `redb`** — less tooling and a less standard file format than SQLite.
- **Keep one file per feature** — no single place to see or reset preferences.

## Consequences
- `crates/cooldown-app/src/settings.rs` owns `settings.json`; new preferences are new fields.
- "Reset the welcome" is `settings.json` edited or deleted, no longer a marker file.
- `diagnostics.json` and `settings.json` are two files on purpose.
- `rusqlite` (bundled SQLite, compiled with the app) and `HistoryStore` are in place; the post-game workout adds its columns with a new migration. This ADR is revisited if the data outgrows SQLite.
- `history.sqlite` (with its `-wal` and `-shm` files while the app runs) lives in the data folder, so "Also delete my settings" at uninstall deletes the history too ([[Installation]]).
- Status stays `proposed` until accepted by the author.
