# Simulator

`cooldown-simulator` is a fake League client for development (see [[ADR-0003 macOS-first development with a simulator]]).

## What it does
- Writes a **lockfile** (default `target/simulator/lockfile`, protocol `http`), so the app discovers it exactly like the real client.
- Serves a fake **LCU API** on `127.0.0.1:51337` (HTTP Basic auth `riot:<password from the lockfile>`).
- Serves a fake **Live Client Data API** on `127.0.0.1:12999` (the real one uses 2999).
- Replays a **scenario**: the response of each endpoint changes over time.
- Removes the lockfile on Ctrl+C, like the real client.

Not emulated yet: HTTPS with Riot's certificate, the LCU WebSocket event stream (see [[Roadmap]]).

## Running
```sh
just dev                       # simulator + app, 10× speed, looping
just sim --speed 2 --loop      # simulator only
cargo run -p cooldown-simulator -- --help
```

## Scenario format
`fixtures/scenarios/<name>/scenario.json`:

```json
{
  "description": "What this scenario shows",
  "synthetic": true,
  "steps": [
    { "at": 0,  "note": "Lobby",
      "lcu": { "/lol-gameflow/v1/gameflow-phase": "Lobby" } },
    { "at": 55, "note": "Game starts",
      "lcu":  { "/lol-gameflow/v1/gameflow-phase": "InProgress" },
      "live": { "/liveclientdata/gamestats": { "gameMode": "CLASSIC", "gameTime": 0.0,
                "mapName": "Map11", "mapNumber": 11, "mapTerrain": "Default" } },
      "game_clock": "start" },
    { "at": 1255, "note": "Game over",
      "live": { "/liveclientdata/gamestats": null },
      "game_clock": "stop" }
  ]
}
```

The simulator serves a path whatever its query string. `synthetic-full-game` also answers `/lol-summoner/v1/current-summoner` and `/lol-match-history/v1/products/lol/current-summoner/matches` from its first step: five older games in two sessions, one ARAM game (never shown) and one remake, for [[History#Games from before cooldown.lol]].

| Field | Meaning |
|---|---|
| `synthetic` | `true` if hand-written, `false` if recorded from a real session |
| `steps[].at` | seconds since the scenario start (scaled by `--speed`) |
| `steps[].note` | printed at startup, helps reading logs |
| `steps[].lcu` / `steps[].live` | endpoint path → JSON body served from now on; `null` removes the endpoint (404) |
| `steps[].game_clock` | `"start"` / `"stop"`: while running, every Live Client body with a `gameTime` field gets the elapsed game time |

The state at time *t* is the result of applying all steps with `at <= t`, in order. Paths must match exactly what the app requests (query strings are ignored).

## Writing a scenario
- Start from `synthetic-full-game` and keep JSON bodies **shaped like the real API** (field names, types). When in doubt, copy a recorded response.
- One scenario per situation worth reproducing (e.g. `synthetic-dodge-in-champ-select`, `synthetic-reconnect`).
- `cargo test` validates every scenario file.

## Recording real sessions
`cooldown-recorder` captures a real session into a scenario with `"synthetic": false`. See [[Recorder]].
