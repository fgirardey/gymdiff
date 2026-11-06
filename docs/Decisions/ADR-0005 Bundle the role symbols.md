---
status: accepted
date: 2026-11-06
---

# ADR-0005 Bundle the role symbols

## Context
[[Champion portraits]] shows League's position symbol (top, jungle, middle, bottom, utility) on each avatar. These symbols are not in Data Dragon ([[ADR-0004 Champion images from Data Dragon]]). They exist in the League client's files (LCU local assets, only while the client runs) and in CommunityDragon, an unofficial mirror of those files. They almost never change.

Riot's developer policy explicitly lists Data Dragon, the Press Kit, TFT and LoR assets as usable; the role symbols are not listed. Riot's "Legal Jibber Jabber" policy allows Riot IP in free, non-commercial community projects that carry its disclaimer and use no Riot logos.

GymDiff will **never be paid** (see [[Vision#Non-goals]]).

## Decision
Bundle the five symbols in the app: `crates/gymdiff-app/ui/images/roles/*.png` (136 × 136, from CommunityDragon's copy of the client files, recoloured from Riot's gold to `text-primary` `#E6EDF3`, alpha kept), exposed to Slint by the `RoleIcons` global in `ui/role-icons.slint`. Same files as `design/assets/roles/`.

## Alternatives considered
- **LCU local assets** — always current, but unavailable when the client is closed, so History would lose the symbols.
- **CommunityDragon at runtime** — unofficial, needs the network, can break.
- **Our own symbols** — no policy question, but less recognisable. Kept as the fallback if Riot objects.

## Consequences
- Works offline, no code for downloading or caching.
- If Riot redraws the symbols, replace the files in a release.
- Keep the Legal Jibber Jabber disclaimer in the app ([[Riot policies]]); ask about the symbols when registering the product on the developer portal.
