---
status: implemented (skeleton)
moment: always
design: "App shell / Top bar / Default"
---

# Client status

Skeleton feature proving the end-to-end pipeline (client discovery → polling → state → UI). Expected to be redesigned once [[Vision]] is written.

## User story
As a player, I can see whether GymDiff is connected to my League client, which phase the client is in, and the in-game clock during a game.

## Behaviour
Shown in the top bar, on every screen except Welcome (see [[Screens]]).
- While no client is found: red dot, "Looking for the League client…".
- When the client answers: green dot, "Connected · <phase>" (phase label from `GamePhase::label`, e.g. "Connected · Lobby").
- When requests fail: red dot, "Error: <message>", cut with an ellipsis if too long.
- When the client closes: back to the disconnected state within a few seconds.
- The in-game clock is still tracked in `CompanionState` but no longer shown: the top bar has no room for it. The data source (simulator or real client) is only logged.

## Data
- Phase: LCU `GET /lol-gameflow/v1/gameflow-phase`
- Clock: Live Client Data `GET /liveclientdata/gamestats` → `gameTime`

See [[League APIs]].

## Simulator scenario
`fixtures/scenarios/synthetic-full-game`

## Windows validation
- [ ] Detects the real client through the lockfile
- [ ] Clock matches the in-game clock
- [ ] Recovers after the client restarts
