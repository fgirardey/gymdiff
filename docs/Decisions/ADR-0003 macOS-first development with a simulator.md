---
status: accepted
date: 2026-11-06
---

# ADR-0003 macOS-first development with a simulator

## Context
GymDiff targets Windows, but day-to-day development happens on a Mac. Waiting for real League games to test every change is slow, and some states (champion select, end of game, reconnect) are hard to reproduce on demand.

## Decision
- Develop and iterate on **macOS** against **`gymdiff-simulator`**, a fake League client that replays scenario files and speaks the same protocols as the real client (lockfile + LCU REST + Live Client Data REST).
- Fence every OS-specific behaviour in `gymdiff-app/src/platform/`, with macOS stubs for Windows-only features.
- Catch Windows compilation errors early with `cargo check --target x86_64-pc-windows-msvc` on the Mac (`just check-windows`) and with CI on `windows-latest`.
- Validate on the **Windows PC** what only it can show: overlays, real client behaviour, performance next to the game.
- Over time, replace synthetic scenarios with **recordings of real sessions** captured on Windows (planned `gymdiff-recorder`, see [[Roadmap]]).

## Alternatives considered
- **Develop directly on Windows** — accurate but slower to iterate, and splits the toolchain (Claude Code, Pencil, Obsidian) across machines.
- **Mock at the Rust trait level only** — simpler, but would not exercise discovery, auth and HTTP; a fake server tests the real code path.
- **Use the macOS League client** — useful for occasional checks (the local APIs exist on macOS too), but still requires real games and can't replay a specific state.

## Consequences
- Scenarios in `fixtures/scenarios/` are a first-class asset, validated by `cargo test`.
- The simulator must evolve with the app: every new endpoint used by the app needs simulator support and scenario data.
- Not yet emulated: HTTPS with Riot's certificate, the LCU WebSocket event stream.
- Windows-specific bugs are diagnosed from log files fetched from the PC (`just win-logs`).
