---
status: implemented
moment: first launch
design: "sketched: First launch / Welcome / Default"
---

# First launch

## User story
As a new player, I want to understand what cooldown.lol will ask of me, and how it computes it, before my first game.

## Why
The workout is a penalty: the player must agree to the deal and know that the numbers are explainable and safe (see [[Vision#Principles]]).

## Behaviour
- Shown full window (no top bar) the first time cooldown.lol starts. See [[Screens]].
- Content:
  - the principle: the role's main stat → burpees, a secondary stat → squats, deaths → plank (see [[Workout formula]]);
  - the health warning: "Listen to your body. Skip anything that hurts." cooldown.lol makes no health claim;
  - Riot's "Legal Jibber Jabber" disclaimer ([[Riot policies]]).
- One button, **Let's go**: opens History and records that the welcome has been seen.
- Never shown again once recorded. If the record cannot be read or written, show the welcome again at the next start (never block the app).
- If a game ends while the welcome is open, the workout is still computed and shown once the player clicks Let's go.

## Data
The `welcome_seen` field of `settings.json` ([[ADR-0007 Local persistence]]; earlier versions used a `welcome-seen` marker file, still read and migrated), stored locally (`%APPDATA%\cooldown.lol\` on Windows, `~/Library/Application Support/cooldown.lol/` on macOS). This is the first piece of settings persistence ([[Roadmap]]).

## Design
`First launch / Welcome / Default` in `design/cooldown.pen`.

## Compliance
Includes Riot's disclaimer. Nothing shown during a game.

## Simulator scenario
None needed: start the app with an empty settings folder.

## Windows validation
- [ ] Welcome shown on first start, not on the second
- [ ] Flag stored under `%APPDATA%\cooldown.lol\`

## Open questions
- Should the About screen offer to show the welcome again?
