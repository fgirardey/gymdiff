---
name: writing-a-simulator-scenario
description: Use when adding or editing a simulator scenario in fixtures/scenarios/ so a cooldown.lol feature can be exercised with just dev — format, conventions and validation
---

# Writing a simulator scenario

The full format is in `docs/Workflow/Simulator.md` (read its *Scenario format* section). Essentials:

- File: `fixtures/scenarios/<name>/scenario.json`. Hand-written scenarios are named `synthetic-<situation>` and set `"synthetic": true`.
- `steps[]` are applied in `at` order (seconds, scaled by `--speed`). `lcu` / `live` map an endpoint path to the JSON body served from then on; `null` removes it (404). `game_clock: "start" | "stop"` patches `gameTime` in Live Client bodies.
- Paths must match exactly what the app requests (see `crates/cooldown-lol/src/http_client.rs`).
- Bodies must be **shaped like the real API** (field names and types); copy from `docs/Architecture/League APIs.md` or a recorded response. Start from `synthetic-full-game`.
- Add a `note` on each meaningful step: it is printed at startup and makes logs readable.

Validate:
1. `cargo test -p cooldown-simulator` (every bundled scenario is loaded by `bundled_scenarios_are_valid`).
2. `just dev 10 fixtures/scenarios/<name>/scenario.json` and watch the app go through the states.
