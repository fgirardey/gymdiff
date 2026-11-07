# Recorder

`gymdiff-recorder` captures a real League session into a simulator scenario, so it can be replayed on the Mac ([[Simulator]]). It exists because the real payloads (champ select session, end-of-game stats) are only available on the Windows PC during real games.

## Safety
- **Read-only**: only `GET` requests, to a fixed list of endpoints plus the ones you add with `--lcu` / `--live`. It never writes to the client, never touches the game process. See [[Riot policies#The recorder]].
- Polls every 2 s by default (minimum 1 s), the same kind of traffic as the app.
- **Anonymised before writing**: player names (also inside event texts), Riot ID tags and `puuid` / `summonerId` / `accountId` are replaced by stable fakes. Nothing raw ever reaches the disk. Still read a recording before committing it.

## Recording
### From a CI download (nothing to install)
1. GitHub → *Actions* → the latest run → *Artifacts* → `gymdiff-recorder-windows-<sha>`, unzip it.
2. In PowerShell, in the folder where you want the recording:
   ```powershell
   Unblock-File .\gymdiff-recorder.exe   # removes the Mark of the Web (no SmartScreen warning)
   .\gymdiff-recorder.exe 2026-11-07-ranked-support
   ```
   The scenario lands in `.\fixtures\scenarios\recorded-<name>\scenario.json` (change it with `--out-dir`).

### From the repository
On the Windows PC, League client open:

```sh
just record 2026-11-07-ranked-support
# extra endpoints: just record my-lobby --lcu /lol-lobby/v2/lobby
```

Stop with Ctrl+C. The scenario is saved on every change, so a crash loses nothing. Output: `fixtures/scenarios/recorded-<name>/scenario.json` (`"synthetic": false`).

Default endpoints:
| API | Endpoints |
|---|---|
| LCU | `gameflow-phase`, `champ-select/v1/session`, `end-of-game/v1/eog-stats-block` |
| Live Client Data | `gamestats`, `allgamedata` |

## Replaying
Copy the folder to the Mac, then `just dev 1 fixtures/scenarios/recorded-<name>/scenario.json` (speed 1 = real time).

## Limits
- Only changes are stored; a field that changes every poll (the game clock) is replaced by the simulator's `game_clock`, which restarts at 0 when the clock first appears. A recording started mid-game therefore replays with a wrong game time.
- Polling can miss a state that lasts less than the interval.
- The LCU WebSocket event stream is not recorded (nor emulated yet, see [[Roadmap]]).
