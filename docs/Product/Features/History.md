---
status: in progress (games from before cooldown.lol; the workout flow is not built yet)
moment: always
design: "sketched: History / Main / *"
---

# History

Every game cooldown.lol has seen, **and the games the player played before installing it**, grouped by session, with the workout that followed each game. It is the app's **main screen**: the workout itself opens as a modal over it (see [[Post-game workout#Workout modal]]). Inspired by the session view of DPM.LOL's match history (games grouped under a session header with a summary), but the workout comes first and the game second.

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
- A game of a mode out of scope (ARAM, arena, custom games, practice tool…) is **not shown at all**: no card, no place in a session, and it does not bridge two sessions (see [[Post-game workout#Scope]]).
- A game without a workout (remake, role not detected, stats unreadable) shows a muted card: the game on the left, "NO WORKOUT", the reason, and "Not counted in the session".
- Clicking a card expands it below the game and workout: one line per exercise with its pictogram, the stat that produced it, its value (`danger` under target) and "/ target", then "Champion · Role · with Linked · game length". Clicking again collapses it. One card expanded at a time.

### Games from before cooldown.lol
When cooldown.lol connects to the League client, it reads the **active summoner** and the client's **match history** (the player's own recent games) and lists the games it has not stored yet. This is the only way to show history from before the installation.
- Such a game is **imported**. cooldown.lol was not there to offer the workout, so it is shown as **skipped**: the workout is computed with the [[Workout formula]] from the game's stats in the match history, and the game gets the usual *Skipped* card ("0/5", "0/35", "0s/30s"…). The played champion portrait has no linked champion (the match history does not give it).
- An imported game whose role Riot did not detect cannot be computed: it shows the muted "No workout" card with the reason "Couldn't detect your role", like a tracked game.
- Imported games group into sessions like any other game (same 1-hour rule), but they **never count in the session summary**: not in the ring, the "x of y workouts done" line, the totals or the session colour. A session made only of imported games has a **neutral header**: `text-secondary` tint, label ("Sun 1 Nov · #1"), "5 games · before cooldown.lol", no ring and no exercise totals. A session mixing both is summarised from its tracked games only.
- On first launch the client usually has recent games, so History opens already filled with these neutral sessions instead of the empty state.
- Imported games from modes out of scope are not stored, and not shown (see above).
- Kept from the match history: result, champion, role (when Riot detected one), K/D/A, queue, start and length, and the stats the [[Workout formula]] needs (CS, vision score, damage to champions, and the team's kills and damage to champions for damage share and kill participation). No linked champion. The current-summoner list only describes the player, so the team totals come from the game's detail (`GET /lol-match-history/v1/games/{gameId}`): only the two team sums are kept, never another player's stats or name.
- The import runs once per client connection (retrying every 10 s until the player is logged in) and stores only games not stored yet (by game id). It also completes a game stored without its stats by an earlier version when the client lists it again. A game cooldown.lol tracked itself is never replaced by its imported copy.
- The champion names and portraits come from Data Dragon, downloaded in the background ([[ADR-0004 Champion images from Data Dragon]]): until an image is there, the card shows the champion's initials.
- Built so far: the neutral session headers, the *Skipped* cards and the muted "No workout" cards. The rings, colours, *Pending/Partial/Done* cards, "Computing" card and expanded card come with the post-game workout.
- The match history of the client is limited (recent games only, see [[#Open questions]]): older games cannot be recovered.
- Another summoner logging into the same client on the same PC: games are kept per player (`puuid`), History shows the active summoner's games only.

### Other rules
- Numbers done can only be recorded in the workout modal; History is read-only in the MVP.
- History is persisted locally and survives restarts. Nothing is sent anywhere.
- Empty state: "No game yet. Every Summoner's Rift game you finish while cooldown.lol is open is listed here."
- A game is stored once, keyed by player and game id: when its end-of-game stats are read (see [[Post-game workout]]), or when it is imported from the client's match history (see above).
- If the history file cannot be read: show the empty state with "Couldn't read your history" and keep the file untouched (never overwrite it).

## Data
- Stored per game: player (`puuid`), game id, source (*tracked* or *imported*), start and end time, queue, champion, role, linked champion, result, K/D/A, the stats used by the [[Workout formula]], the computed amounts due (and caps applied), the amounts done per exercise, and the workout status. An imported game has no linked champion; its status is *skipped*, its amounts done are 0, and its amounts due are computed at import with the formula of the time.
- Store the stats, not only the amounts: if the formula changes, old games keep the amounts computed at the time (amounts are stored too).
- Location: `%APPDATA%\cooldown.lol\` on Windows, `~/Library/Application Support/cooldown.lol/` on macOS. One SQLite database, `history.sqlite` ([[ADR-0007 Local persistence]]); its `-wal` and `-shm` files next to it while the app runs.
- Sessions are computed from the stored games (pure function in `cooldown-core`), never stored.

## Design
Pencil frames `History / Main / Before cooldown.lol`, `History / Main / Empty`, `History / Main / Nominal`, `History / Main / Game expanded`, `History / Main / Computing`, `History / Main / No workout`, built from the `Session header / *` and `Game row / *` components. See [[Screens#Frames]].

## Compliance
Only the local player's own games, after they end (live ones and the client's match history of the active summoner). Never other players' games or names, nothing during a game, no call to the Riot API.

## Simulator scenario
A scenario chaining several games with short and long breaks (two sessions), plus a remake, and a client match history with games older than the first tracked one (see `GET /lol-match-history/v1/products/lol/current-summoner/matches`).

## Windows validation
- [ ] History survives an app restart
- [ ] A break over 1 hour starts a new session
- [ ] `history.sqlite` in `%APPDATA%\cooldown.lol\`, and gone with the folder when uninstalling with "Also delete my settings"
- [ ] Games played before the install appear as *Skipped* cards with their computed amounts, newest first, in sessions with a neutral header
- [ ] Reconnecting the client does not duplicate them
- [ ] How many past games the client really returns (see open questions)

## Open questions
- How many past games does the LCU match history return? Seen on the macOS client: 21 games at one moment, 3 a little later, and `begIndex`/`endIndex` ignored in the second case (the same games came back for every page). It looks like a cache the client fills as its own match history is browsed. To check on the Windows PC. The import stops when a page brings no new game, so this never loops.
- Roles: `timeline.lane` alone is not reliable (Riot called a top laner `JUNGLE`). cooldown.lol reads a `JUNGLE` without Smite (summoner spell 11) as top. To validate on more real games, on Windows too.
- Empty-state wording once the client can fill History: kept as is for now.
- Keep everything forever, or only the last N days?
- Allow fixing the numbers done from History (e.g. forgot to finish the flow)?
- Filters (role, queue) like DPM.LOL, later?
