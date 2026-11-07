---
status: in progress (empty state only)
moment: always
design: "sketched: History / Main / *"
---

# History

Every game GymDiff has seen, grouped by session, with the workout that followed each game. It is the app's **main screen**: the workout itself opens as a modal over it (see [[Post-game workout#Workout modal]]). Inspired by the session view of DPM.LOL's match history (games grouped under a session header with a summary), but the workout comes first and the game second.

## User story
As a player, I want to look back at my sessions and see, game after game, which workout I did, partly did or skipped, so that I keep myself honest across days.

## Why
The workout only works as a habit if the player sees their streak, and their skips (see [[Vision#Success]]).

## Behaviour

### Sessions
- A **session** is a run of games where each game starts less than **1 hour** after the previous one ended. A longer break starts a new session. App restarts do not matter.
- Session label: day and rank of the session in that day, e.g. "Today · #1", "Yesterday · #2", "Sun 1 Nov · #1".
- Sessions are listed newest first. The current session (if any) is on top.

### Session summary
Each session is **one panel** with rounded corners and no outline: the **summary** is the panel's header (tinted at ~12 %, separated by a line), and the session's game cards sit inside the panel below it, on a faint tint (~4 %) of the same colour. The summary holds:
- a **ring** showing workouts done / workouts (e.g. "3/5"), partial workouts counting as not done;
- the label (e.g. "Today · #1") and, for the current session, an "IN PROGRESS" chip;
- "3 of 5 workouts done" in the session's colour;
- totals done / due per exercise, each behind its pictogram (e.g. burpee 25/25 · squat 77/87 · plank 230/300 s).

Session colour (header tint at ~12 %, header separator at ~40 %, games area at ~4 %, ring, "x of y" line):
| Session | Colour |
|---|---|
| Current session (still in progress) | `accent` |
| Every workout done | `success` |
| At least half done | `warning` |
| Less than half done | `danger` |

### Workout cards
The workout is the main content; the game is kept, smaller, on the side. Inside the session panel, one **card** per game, newest first (about four to five cards fit in the window):

- **Left, the game**: played and linked champion portraits with their role symbols ([[Champion portraits]]), and WIN (`success`) / LOSS (`danger`) under them.
- **Right, the workout**:
  - status line: status icon and label in its colour — *Done* (`success`), *Partial* (`warning`), *Skipped* (`text-secondary`), *Pending* (`accent`) — and, on the right, the K/D/A, or a **Start** button while the workout is pending (reopens the workout modal where it was left, see [[Post-game workout#Workout modal]]);
  - the three exercises side by side: number done (display font, title size), "/ due" next to it, then the pictogram and a progress bar (`success` when complete, `warning` when partial, empty track otherwise).
- The pending card has an `accent` outline.
- While the end-of-game stats are read, a "Game over · Computing your workout…" card (`accent` outline, spinner) sits on top of the current session.
- A game without a workout (remake, mode out of scope, role not detected, stats unreadable) shows a muted card: the game on the left, "NO WORKOUT", the reason, and "Not counted in the session".
- Clicking a card expands it below the game and workout: one line per exercise with its pictogram, the stat that produced it, its value (`danger` under target) and "/ target", then "Champion · Role · with Linked · game length". Clicking again collapses it. One card expanded at a time.

### Other rules
- Numbers done can only be recorded in the workout modal; History is read-only in the MVP.
- History is persisted locally and survives restarts. Nothing is sent anywhere.
- Empty state: "No game yet. Every Summoner's Rift game you finish while GymDiff is open is listed here."
- A game is stored once (keyed by game id), when its end-of-game stats are read (see [[Post-game workout]]).
- If the history file cannot be read: show the empty state with "Couldn't read your history" and keep the file untouched (never overwrite it).

## Data
- Stored per game: game id, start and end time, queue, champion, role, linked champion, result, K/D/A, the stats used by the [[Workout formula]], the computed amounts due (and caps applied), the amounts done per exercise, and the workout status.
- Store the stats, not only the amounts: if the formula changes, old games keep the amounts computed at the time (amounts are stored too).
- Location: `%APPDATA%\GymDiff\` on Windows, `~/Library/Application Support/GymDiff/` on macOS. Format to decide in an ADR at implementation.
- Sessions are computed from the stored games (pure function in `gymdiff-core`), never stored.

## Design
Pencil frames `History / Main / Empty`, `History / Main / Nominal`, `History / Main / Game expanded`, `History / Main / Computing`, `History / Main / No workout`. See [[Screens#Frames]].

## Compliance
Only the local player's own games, after they end. Nothing during a game.

## Simulator scenario
A scenario chaining several games with short and long breaks (two sessions), plus a remake.

## Windows validation
- [ ] History survives an app restart
- [ ] A break over 1 hour starts a new session
- [ ] File in `%APPDATA%\GymDiff\`

## Open questions
- Keep everything forever, or only the last N days?
- Allow fixing the numbers done from History (e.g. forgot to finish the flow)?
- Filters (role, queue) like DPM.LOL, later?
