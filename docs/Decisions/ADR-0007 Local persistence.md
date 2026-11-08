---
status: proposed
date: 2026-11-08
---

# ADR-0007 Local persistence

## Context
GymDiff stores two kinds of data locally, with different needs:
- **Preferences** (welcome seen, later difficulty and targets): a few values, read at start, rarely written.
- **Game history** ([[History]]): grows with every game, is filtered by date and session, and must survive crashes.

So far preferences were spread over marker files (`welcome-seen`) and one JSON per feature (`diagnostics.json`).

## Decision
- **Preferences: one `settings.json`** in the data folder (`platform::data_dir`), a `Settings` struct with `serde`:
  - `version` field, to migrate;
  - every field has a default, a missing, unknown or unreadable file gives the defaults (never blocks the app);
  - writes are atomic (temporary file, then rename);
  - earlier marker files are migrated on read (`welcome-seen`).
- **Consent to error reports stays in `diagnostics.json`** ([[ADR-0006 Opt-in error reports]]): it is read by the pure `gymdiff-diagnostics` crate and "corrupt means off" must hold, so a damaged settings file must never be able to switch reporting on.
- **Game history: SQLite through `rusqlite`** (feature `bundled`, so nothing to install on Windows), WAL mode, schema migrations with `PRAGMA user_version` or `rusqlite_migration`, one writer. Added with the History feature, not before.
- Persistence stays in the app crate, behind small store types; `gymdiff-core` stays pure and never touches the disk.

## Alternatives considered
- **One JSON file for everything, history included** — rewritten in full on every game, no queries, fragile on crash.
- **SQLite for preferences too** — a database for a handful of values; harder to inspect and edit by hand.
- **`sqlx`** — async and compile-time checked queries, but heavy for a local single-user database.
- **`sled` / `redb`** — less tooling and a less standard file format than SQLite.
- **Keep one file per feature** — no single place to see or reset preferences.

## Consequences
- `crates/gymdiff-app/src/settings.rs` owns `settings.json`; new preferences are new fields.
- "Reset the welcome" is `settings.json` edited or deleted, no longer a marker file.
- `diagnostics.json` and `settings.json` are two files on purpose.
- History will add `rusqlite` and a store module; this ADR is revisited if the data outgrows it.
- Status stays `proposed` until accepted by the author.
