---
status: idea
moment: after the game
design: none yet
---

# Post-game workout

The MVP feature. When a Summoner's Rift game ends, GymDiff shows the role's key stats and the workout they convert into, computed with the [[Workout formula]].

## User story
As a player who just finished a game, I want to see right away how many burpees, squats and seconds of plank I owe, and why, so that I do them before queuing again.

## Why
Turns the dead time between games into a workout, and gives the end-of-game stats a consequence (see [[Vision]]).

## Scope
- Queues on Summoner's Rift: ranked solo/duo, ranked flex, normal draft, normal blind, swiftplay.
- Out of scope for the MVP: ARAM, rotating modes, arena, custom games, practice tool, bot games, spectator.
- No overlay: nothing is shown during the game.

## Behaviour

### Main window by phase
| Client phase | What the window shows |
|---|---|
| No client / idle / lobby / queue / champ select | **Last workout** (if any) and the **session total**. Before the first game: an empty state inviting the player to play a game. |
| In game (`GameStart`, `InProgress`, `Reconnect`) | Same as above. Nothing new appears during the game. |
| `WaitingForStats`, `PreEndOfGame` | "Game over — computing your workout…" |
| `EndOfGame` | The **workout** for the game that just ended. |

The workout of a game stays on screen until the next eligible game ends.

### Workout screen (nominal)
- Header: Victory / Defeat, champion name, role, game length, K/D/A.
- One row per exercise, in this order: **Burpees** (role's main stat), **Squats** (role's secondary stat), **Plank** (deaths). Each row shows:
  - the amount (big number): reps, or seconds for the plank,
  - the stat that produced it, its actual value and its target (e.g. "Damage share 22 % · target 25 %", "6 deaths"),
  - the stat in `danger` when under target, in `success` when at or above target (then 0).
- A row whose cap applied says "capped at N".
- Which stats feed burpees and squats depends on the role (e.g. Support: vision score for burpees, kill participation for squats), see [[Workout formula]].
- Two actions: **Done** and **Skip**. Either one marks the workout as handled; the screen then shows the session total highlighted.
- A short line: "Listen to your body. Skip anything that hurts." (no health claim).

### Session total
- A session starts when GymDiff starts and ends when it quits.
- Shows: games played, total per exercise (reps, plank seconds), done vs. skipped.
- Not persisted across restarts in the MVP.

### Edge cases
| Situation | Behaviour |
|---|---|
| Remake (< 5 min or flagged by the client) | "Remake — no workout." Not counted in the session. |
| Mode out of scope (ARAM, arena, …) | "This mode isn't covered yet." Not counted. |
| Role unknown (blind pick, no position in the data) | Ask the player to pick their role (5 buttons), then compute. |
| End-of-game stats unavailable (endpoint missing, malformed payload) | "Couldn't read the end-of-game stats." Nothing counted. Logged with the payload. |
| Player leaves the end-of-game screen before stats arrive | Keep trying while the client is connected; if the stats never arrive, same as above. |
| GymDiff started while already on the end-of-game screen | Compute and show the workout normally. |
| Same game seen twice (polling) | Counted once (keyed by game id). |
| Client closes after the game | The workout and session stay visible. |

### Window and attention
- The workout appears in the GymDiff window; GymDiff does **not** steal focus from the League client.
- Proposal: flash the taskbar button on Windows when a new workout is ready (macOS: no-op stub).

## Data
- **Phase**: LCU `GET /lol-gameflow/v1/gameflow-phase` (already used by [[Client status]]).
- **End-of-game stats**: LCU `GET /lol-end-of-game/v1/eog-stats-block`, available around `EndOfGame`. Expected content: game id, game length, queue type, game mode, and per team the players with their stats (kills, deaths, assists, minions, neutral minions, damage to champions, vision score) and the winning team. **Field names to confirm with a recorded payload** from the Windows PC.
- **Role**, in this order:
  1. LCU `GET /lol-champ-select/v1/session` during champion select → `myTeam[].assignedPosition` of the local player (ranked, draft); remembered until the game ends.
  2. Live Client Data `GET /liveclientdata/playerlist` during the game → `position` of the local player.
  3. Otherwise ask the player.
- **Queue / mode** to check the scope: from the end-of-game stats, or LCU `GET /lol-gameflow/v1/session`.

See [[League APIs]].

## Design
To do in Pencil, frames `Post-game workout / <Screen> / <State>`:
- `Main / Empty` (no game yet)
- `Main / Computing`
- `Main / Workout` (variants: one per role, with a cap)
- `Main / Handled` (after Done or Skip, session total highlighted)
- `Main / Pick role`
- `Main / Not covered` (remake, mode out of scope)
- `Main / Error`

## Compliance
- Shows nothing during champion select or the game: only after the game.
- Shows only the local player's own stats, all visible on Riot's end-of-game screen.
- To check before release: whether Riot expects the Riot API (match-v5) rather than the LCU for end-of-game stats ([[Riot policies]]).

## Simulator scenario
Extend `synthetic-full-game` (or add one scenario per case) with:
- a champion select session with an assigned position,
- an end-of-game stats block at `EndOfGame`,
- variants: Support, blind pick without position, remake, ARAM.

## Windows validation
- [ ] The workout appears after a real ranked or normal game
- [ ] Role detected from champion select in ranked/draft
- [ ] Blind pick asks for the role
- [ ] Remake shows no workout
- [ ] ARAM shows "not covered"
- [ ] Taskbar flash, no focus stealing
- [ ] Record the end-of-game payload as a fixture

## Open questions
- Should Done/Skip be per exercise rather than for the whole workout?
- Should the plank row have a built-in countdown timer?
- Should the session survive an app restart (e.g. same day)? Needs persistence.
- Should GymDiff start with Windows / live in the tray so it never misses a game?
