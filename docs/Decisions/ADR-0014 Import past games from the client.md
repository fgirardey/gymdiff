---
status: proposed
date: 2026-11-08
---

# ADR-0014 Import past games from the client

## Context
[[History]] was specified as "every game cooldown.lol has seen". A player who installs cooldown.lol therefore starts with an empty screen, although the League client already knows their recent games. The player asked to see that history too.

## Decision
- When cooldown.lol connects to the League client, it reads the active summoner (`/lol-summoner/v1/current-summoner`) and the client's match history for that summoner (`/lol-match-history/v1/products/lol/current-summoner/matches`), and stores the games it does not have yet as **imported** games.
- Imported games carry what the match history gives (result, champion, role, K/D/A, queue, start, length) and the stats the [[Workout formula]] needs. Their workout is computed at import and they show as *Skipped*, never counting in a session's summary ([[History#Games from before cooldown.lol]]). The list only describes the player, so the team's kills and damage come from the detail of each game (`GET /lol-match-history/v1/games/{gameId}`), of which only the two team sums are kept.
- Games are keyed by `puuid` and game id. A game tracked live is never overwritten by its imported copy.
- Only the active summoner's own participant is read; other players in a payload are ignored and never stored ([[Riot policies]], de-anonymisation rule).
- The data lives in the local SQLite database of [[ADR-0007 Local persistence]]; nothing is sent anywhere.
- Roles come from `timeline.lane` and `timeline.role`, which Riot computes after the game. A `JUNGLE` lane without Smite is a top laner (Riot's detection mixes them up). When the lane is missing the role is unknown, not guessed, and the game shows "Couldn't detect your role".

## Alternatives considered
- **Riot Games API (match-v5)** — deeper history, but it needs an API key and a registered product, and the LCU already answers locally ([[Riot policies]]).
- **Count past games in the session summary** — they would drag the rings and colours down for games the player never had a chance to do the workout for: they are shown as *Skipped* but not counted.
- **Only track games from the install onwards** — simplest, but an empty History for a new player.

## Consequences
- `LolClient` gains the active summoner and the match history.
- `cooldown-core` gets a `Game` model (tracked or imported) and a pure function that groups games into sessions.
- The history depth is bounded by what the client returns; to confirm on the Windows PC.
- Games of the modes out of scope (ARAM, arena…) are neither stored nor shown, so a game of another mode never bridges two sessions.
- The LCU is unsupported by Riot: payloads are parsed leniently and a missing endpoint means "no past games", never an error shown to the player.
