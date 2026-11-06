---
status: accepted
date: 2026-11-06
---

# ADR-0004 Champion images from Data Dragon

## Context
[[Champion portraits]] needs the square image of the played and linked champions. Until now GymDiff only talks to `127.0.0.1`. The LCU serves champion images locally too (`/lol-game-data/assets/...`), but only while the client runs, and History must show old games with the client closed.

## Decision
Download champion square images from Riot's Data Dragon CDN (`https://ddragon.leagueoflegends.com/cdn/<version>/img/champion/<id>.png`, version from `https://ddragon.leagueoflegends.com/api/versions.json`) and cache them under the app data folder. The champion id → image mapping comes from Data Dragon's `champion.json` for the same version.

- Download lazily, in the background, never on the UI thread.
- Missing image = placeholder with initials; the app works fully offline.
- No other data is sent: requests only contain the version and champion id.

## Alternatives considered
- **LCU game-data assets** — local, no network, but unavailable when the client is closed, so History would lose images.
- **Text only** — simplest, but the player asked for portraits.
- **Bundling images in the app** — large, and outdated at every new champion.

## Consequences
- GymDiff makes its first outbound network requests: mention it in the About screen and the README.
- Cache invalidation per Data Dragon version; tests use a fake image source.
- Check Riot's policy on Data Dragon asset use before release ([[Riot policies]]).
