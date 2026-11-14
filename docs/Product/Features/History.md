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

### Views
The screen's title row holds "HISTORY", a **view switch** and a one-line legend on the right. The two views list the same sessions and games; only the middle of each row changes:
- **Games** (default, swords icon): the stats of each game. Legend: "Red = under your target · tick = target for the role".
- **Workouts** (dumbbell icon): the exercises each game turned into. Legend: "Each exercise pays for one stat under its target".

The view picked stays while the app runs; it is not saved.

### Session header
Each session starts with a **header** line, not a panel: the label (display font, e.g. "TODAY · #1") on the left, then chips on the right:
- a summary chip: "x of y workouts done" in the session's colour on a tint of it, partial workouts counting as not done (see the table); for a session of games from before cooldown.lol, "5 games · before cooldown.lol" in `text-secondary` on `surface`;
- a wins and losses chip: "1 W" (`info`) and "2 L" (`danger`) on `surface`; remakes count in neither.

Summary chip colour:
| Session | Colour |
|---|---|
| Current session (still in progress) | `accent` |
| Every workout done | `success` |
| At least half done | `warning` |
| Less than half done | `danger` |

### Game rows
Under its header, one **row** per game, newest first: 64 px high, full width, `surface`, rounded. On the left a 3 px **stripe**; in the Games view a tint of the same colour fades from it to nothing at 60 % of the width (`info` for a win, `danger` for a loss, `text-secondary` for a remake or a game without workout; tints in [[Tokens]]).

- **Game** (both views): the played champion portrait with its role symbol and the linked champion ([[Champion portraits]]), the champion's name, then the queue ("Ranked Solo/Duo", Games view) or the role ("Bottom · with Lulu", Workouts view), then the result (WIN `info`, LOSS `danger`, REMAKE `text-secondary`) with the game's length ("34:12"), preceded by the K/D/A in the Workouts view.
- **Games view**, five equal columns:
  - K/D/A in the display font, deaths in `danger`, with the KDA ratio ("1.1 KDA" = kills plus assists over deaths, deaths counted as 1 when 0);
  - CS per minute, damage share, kill participation, vision per minute: pictogram, value, a gauge and the stat's name. The gauge is full at 10 CS per minute, 3 vision per minute and 100 % for the shares. The stats the [[Workout formula]] judges for the role (main and secondary) have a **tick** at the role's target, and their value and gauge turn `danger` under it; the others have no tick and stay `text-primary`. A game whose stats could not be read shows "—".
- **Workouts view**, three equal columns, one per exercise: the stat that pays for it (pictogram and name), its value (`danger` under target), "/ target" and the gap in the stat's unit ("−171 CS", "−9 pts"); an arrow; then the amount: pictogram, number done (display font) and "/due", a progress bar, and "max 30" in `warning` when the game's cap applied. The plank's stat is the deaths, "× 10 s".
- **Workout status**: status icon and label — *Done* (`success`), *Partial* (`warning`), *Skipped* (`text-secondary`), *Pending* (`accent`, with a small **Start** button that reopens the workout modal where it was left, see [[Post-game workout#Workout modal]]) — or "NO WORKOUT" and the reason.
- A **chevron** at the end opens the row's **details** below it, and closes them. Details: one line per exercise — pictogram, name and what it pays for ("Main stat · CS per minute", "Secondary stat · Damage share", "Every death"), then the formula with its numbers and their captions, right-aligned against "=" (e.g. "( 8.0 target CS / min × 34.2 minutes − 103 your CS ) ÷ 5 CS per burpee", the player's value in `danger` under target, the formula's constant in `success`), then the amount and its status.
- While the end-of-game stats are read, a "Game over · Computing your workout…" row sits on top of the current session.
- A game of a mode out of scope (ARAM, arena, custom games, practice tool…) is **not shown at all**: no row, no place in a session, and it does not bridge two sessions (see [[Post-game workout#Scope]]).
- A game without a workout (remake, role not detected, stats unreadable) has a muted row: `text-secondary` stripe and tint, the status "NO WORKOUT" and the reason. It has no details and is not counted in the session.

### Games from before cooldown.lol
When cooldown.lol connects to the League client, it reads the **active summoner** and the client's **match history** (the player's own recent games) and lists the games it has not stored yet. This is the only way to show history from before the installation.
- Such a game is **imported**. cooldown.lol was not there to offer the workout, so it is shown as **skipped**: the workout is computed with the [[Workout formula]] from the game's stats in the match history, and the game gets the usual *Skipped* row (Workouts view: "0/5", "0/35", "0s/30s"…; details: "skipped"). The played champion portrait has no linked champion (the match history does not give it), and the Workouts view shows the role alone.
- An imported game whose role Riot did not detect cannot be computed: it shows the muted "No workout" row with the reason "Couldn't detect your role", like a tracked game.
- Imported games group into sessions like any other game (same 1-hour rule), but they **never count in the session summary**: not in the "x of y workouts done" chip or its colour. A session made only of imported games has a **neutral header**: label ("Sun 1 Nov · #1"), the "5 games · before cooldown.lol" chip and the wins and losses chip. A session mixing both is summarised from its tracked games only.
- On first launch the client usually has recent games, so History opens already filled with these neutral sessions instead of the empty state.
- Imported games from modes out of scope are not stored, and not shown (see above).
- Kept from the match history: result, champion, role (when Riot detected one), K/D/A, queue, start and length, and the stats the [[Workout formula]] needs (CS, vision score, damage to champions, and the team's kills and damage to champions for damage share and kill participation). No linked champion. The current-summoner list only describes the player, so the team totals come from the game's detail (`GET /lol-match-history/v1/games/{gameId}`): only the two team sums are kept, never another player's stats or name.
- The import runs when cooldown.lol connects to the client (retrying every 10 s until the player is logged in), and again when a game ends: on reaching the end-of-game screen, and on leaving the post-game phases, since the client's match history may list the game a little later. Until the [[Post-game workout]] is built, this is how a game played while cooldown.lol is open shows up (as *Skipped*). It stores only games not stored yet (by game id). It also completes a game stored without its stats by an earlier version when the client lists it again. A game cooldown.lol tracked itself is never replaced by its imported copy.
- The champion names and portraits come from Data Dragon, downloaded in the background ([[ADR-0004 Champion images from Data Dragon]]): until an image is there, the row shows the champion's initials.
- Built so far: both views, the neutral session headers, the *Skipped* rows with their details and the muted "No workout" rows. The "x of y workouts done" chip, the *Pending/Partial/Done* rows and the "Computing" row come with the post-game workout.
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
Pencil frames `History / Main / Before cooldown.lol`, `History / Main / Empty`, `History / Main / Nominal`, `History / Main / Game expanded`, `History / Main / Computing`, `History / Main / No workout`, and the `History / Main / … (desktop)` frames of both views, built from the `Session header / *` and `Game row / *` components and the `Stat icon / *` pictograms. See [[Screens#Frames]].

## Compliance
Only the local player's own games, after they end (live ones and the client's match history of the active summoner). Never other players' games or names, nothing during a game, no call to the Riot API.

## How to try it
`just dev` with the League client open and logged in: History fills from its match history. A throwaway data folder (`COOLDOWN_DATA_DIR=$(mktemp -d) just dev`) shows the first opening. Finish a game: History lists it once the end-of-game screen shows, or once back in the lobby.

## Windows validation
- [ ] History survives an app restart
- [ ] A break over 1 hour starts a new session
- [ ] `history.sqlite` in `%APPDATA%\cooldown.lol\`, and gone with the folder when uninstalling with "Also delete my settings"
- [ ] Games played before the install appear as *Skipped* cards with their computed amounts, newest first, in sessions with a neutral header
- [ ] Reconnecting the client does not duplicate them
- [ ] A game played while cooldown.lol is open appears after the end-of-game screen, without restarting anything
- [ ] How many past games the client really returns (see open questions)

## Open questions
- How many past games does the LCU match history return? Seen on the macOS client: 21 games at one moment, 3 a little later, and `begIndex`/`endIndex` ignored in the second case (the same games came back for every page). It looks like a cache the client fills as its own match history is browsed. To check on the Windows PC. The import stops when a page brings no new game, so this never loops.
- Roles: `timeline.lane` alone is not reliable (Riot called a top laner `JUNGLE`). cooldown.lol reads a `JUNGLE` without Smite (summoner spell 11) as top. To validate on more real games, on Windows too.
- Empty-state wording once the client can fill History: kept as is for now.
- Keep everything forever, or only the last N days?
- Allow fixing the numbers done from History (e.g. forgot to finish the flow)?
- Filters (role, queue) like DPM.LOL, later?
