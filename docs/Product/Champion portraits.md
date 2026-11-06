---
status: draft
---

# Champion portraits

How a game shows who the player was and who they played with. Used by [[Post-game workout]] (workout modal header) and [[History]] (game rows).

## What is shown
Both avatars are shown **every time a game is shown**, side by side, as on DPM.LOL's match history:
- **Played champion**: rounded square, 52 px, in front.
- **Linked champion**: rounded square, 24 px (less than half the played one), to its right, top-aligned, slightly tucked behind it.
- Each avatar has a **role badge** in its bottom corner holding League's position symbol (top, jungle, middle, bottom, utility), in `text-primary` like the text: 18 px on the played avatar, 14 px on the linked one. "?" when the role is unknown.
- The text next to them names both: "Ashe · Bot", then "with Lulu" in `text-secondary`.

Pencil component: `Champion portrait`.

## Linked role
| Played role | Linked ally |
|---|---|
| Top | Jungle |
| Jungle | Mid |
| Mid | Jungle |
| Bottom | Support |
| Support | Bottom |

## Rules
- Only **allies** are shown, never opponents.
- The linked ally is the player of the same team whose position is the linked role, read from the end-of-game stats (see [[Post-game workout#Data]]).
- If no ally has the linked role (positions missing from every source, see [[Post-game workout#Data]]): the linked avatar is still shown, as an empty slot with "?", and its role badge shows "?". The text has no "with …".
- If an image is missing (not downloaded yet, offline, new champion): show a placeholder with the champion's first two letters on `surface`. Never block the screen on an image.

## Images
Champion square images from Riot's **Data Dragon** CDN, downloaded once per game version and cached locally. See [[ADR-0004 Champion images from Data Dragon]].

Role symbols are bundled with the app (`RoleIcons` in `ui/role-icons.slint`). See [[ADR-0005 Bundle the role symbols]].

In Pencil, portraits use the Data Dragon squares stored in `design/assets/champions/` as image fills (see `design/README.md`); the `Champion portrait` component itself keeps the initials placeholder, which is also the missing-image state.

## Compliance
- Shown only after the game, and only allies, all visible on Riot's end-of-game screen.
- Data Dragon assets are provided by Riot for third-party products; keep the "Legal Jibber Jabber" disclaimer ([[Riot policies]]).
