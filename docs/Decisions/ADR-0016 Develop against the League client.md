---
status: accepted
date: 2026-11-14
---

# ADR-0016 Develop against the League client

## Context
[[ADR-0003 macOS-first development with a simulator]] chose to develop on macOS against `cooldown-simulator`, a fake League client replaying scenario files, with `cooldown-recorder` capturing real sessions on Windows to feed it. In practice the League client exists for macOS and exposes the same local APIs: the app connects to it, follows its phase and reads the match history of the logged-in player ([[ADR-0014 Import past games from the client]]) with no fake in between. The simulator and the recorder are about 1 000 lines of Rust, a scenario format, two skills and a rule that every endpoint used by the app needs simulator support: a cost paid on every feature, for states that unit tests on real payloads cover as well.

## Decision
- Develop on **macOS against the League client running on the same Mac**: `just dev` runs the app, which discovers the client's lockfile like on Windows.
- **Remove** `cooldown-simulator`, `cooldown-recorder`, `fixtures/scenarios/`, the `writing-a-simulator-scenario` skill and their recipes and docs. `COOLDOWN_LOCKFILE` stays, for a client installed outside the default folders.
- A state the real client cannot produce on demand (end of game, champion select roles, remake) is covered by **unit tests on fixtures of the real payloads** (`cooldown-lol` models, `cooldown-core` logic) and validated on the Windows PC, as before for Windows-only behaviour.
- Everything else of ADR-0003 stands: OS code fenced in `cooldown-app/src/platform/`, `just check-windows` and CI on `windows-latest`, validation on the PC for what only it can show.

## Alternatives considered
- **Keep the simulator** — reproduces any state on demand, but every endpoint and every state has to be written twice (app and fake), and the fake can drift from the real client.
- **Keep it as an option (`just dev-sim`)** — less drastic, but the maintenance cost stays.

## Consequences
- Developing needs the League client installed and a logged-in account on the Mac.
- No way to replay a state without playing it: for the post-game workout, the first real end-of-game payloads are captured by hand (a `curl` against the client while the game ends) and kept as test fixtures.
- Fewer crates, no `axum` or `clap` dependency, no scenario format to maintain.
- The macOS build stays in CI and in the releases: it is the build developed and run daily.
