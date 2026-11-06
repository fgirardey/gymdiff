---
status: implemented (skeleton)
moment: always
design: none yet
---

# Client status

Skeleton feature proving the end-to-end pipeline (client discovery → polling → state → UI). Expected to be redesigned once [[Vision]] is written.

## User story
As a player, I can see whether GymDiff is connected to my League client, which phase the client is in, and the in-game clock during a game.

## Behaviour
- While no client is found: red dot, "Looking for the League client…".
- When the client answers: green dot, "Connected", current phase label (see `GamePhase::label`).
- During a game: the in-game clock as `mm:ss`, refreshed every second. Outside a game: `--:--`.
- When the client closes: back to the disconnected state within a few seconds.
- The data source (simulator or real client) is shown at the bottom.

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
