---
status: proposed
date: 2026-11-08
---

# ADR-0014 Import past games from the client

## Context
[[History]] was specified as "every game GymDiff has seen". A player who installs GymDiff therefore starts with an empty screen, although the League client already knows their recent games. The player asked to see that history too.

## Decision
- When GymDiff connects to the League client, it reads the active summoner (`/lol-summoner/v1/current-summoner`) and the client's match history for that summoner (`/lol-match-history/v1/products/lol/current-summoner/matches`), and stores the games it does not have yet as **imported** games.
- Imported games carry only what the match history gives (result, champion, role, K/D/A, queue, start, length). They have no workout and show as "No workout · Played before GymDiff" ([[History#Games from before GymDiff]]).
- Games are keyed by `puuid` and game id. A game tracked live is never overwritten by its imported copy.
- Only the active summoner's own participant is read; other players in a payload are ignored and never stored ([[Riot policies]], de-anonymisation rule).
- The data lives in the local SQLite database of [[ADR-0007 Local persistence]]; nothing is sent anywhere.
- Roles come from `timeline.lane` and `timeline.role`, which Riot computes after the game; when they are missing the role is unknown, not guessed.

## Alternatives considered
- **Riot Games API (match-v5)** — deeper history, but it needs an API key and a registered product, and the LCU already answers locally ([[Riot policies]]).
- **Compute workouts retroactively for past games** — would show *Skipped* workouts for games that were never offered one, and break the session rings.
- **Only track games from the install onwards** — simplest, but an empty History for a new player.

## Consequences
- `LolClient` gains the active summoner and the match history; the simulator serves both.
- `gymdiff-core` gets a `Game` model (tracked or imported) and a pure function that groups games into sessions.
- The history depth is bounded by what the client returns; to confirm on the Windows PC.
- The LCU is unsupported by Riot: payloads are parsed leniently and a missing endpoint means "no past games", never an error shown to the player.
