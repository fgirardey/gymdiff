# CLAUDE.md

Guidance for Claude Code when working in this repository.

## Project

GymDiff is a **Windows desktop companion app for League of Legends**, written in **Rust** with a **native Slint UI**. It reads data from the game's local APIs (LCU + Live Client Data). Development happens on **macOS** against a **simulator**; the real target is **Windows**.

Everything in this repository is written in **English**: code, comments, commit messages, docs.

## Sources of truth

Implement from these artifacts, in this order. Never invent product behaviour or visuals that they do not describe; ask instead.

1. **Product** — `docs/Product/` (Obsidian vault, Markdown). What to build and why. Each feature has a note in `docs/Product/Features/`.
2. **Design** — `design/*.pen` (Pencil). What it looks like. Read it through the **Pencil MCP server** (the Pencil app must be running with the file open). Frames are named after the feature notes (see `design/README.md`).
3. **Design tokens** — `docs/Design/Tokens.md`, mirrored in `crates/gymdiff-app/ui/theme.slint`. Use `Theme.*` in `.slint` files; never hard-code colors, spacing or font sizes.
4. **Decisions** — `docs/Decisions/ADR-*.md`. Respect them; propose a new ADR to change one.

If the code, the docs and the design disagree, stop and ask which one is right.

## Repository layout

```
crates/gymdiff-core/       Pure domain logic. No I/O, no #[cfg(target_os)]. Fully unit-tested.
crates/gymdiff-lol/        League local APIs: lockfile discovery, LCU + Live Client Data clients, models.
crates/gymdiff-simulator/  Fake League client replaying fixtures/scenarios/*.json (dev on macOS).
crates/gymdiff-app/        Slint UI + OS integration. OS-specific code only in src/platform/.
fixtures/                  Scenarios for the simulator and tests.
docs/                      Obsidian vault: product, architecture, decisions, workflows.
design/                    Pencil design files (.pen).
```

## Commands

```sh
just dev            # app + simulator (main dev loop on macOS)
just check          # fmt-check + clippy + tests + Windows type-check (run before committing)
just test           # cargo test --workspace
just check-windows  # cargo check --target x86_64-pc-windows-msvc
just preview        # live preview of a .slint file (needs slint-viewer)
```

## Rules

- **Architecture**: domain logic goes in `gymdiff-core` (pure functions, reducer pattern: `CompanionState::apply(Event)`). Data sources produce `Event`s; the UI only renders state. Talk to the game only through the `LolClient` trait.
- **Platform code**: anything Windows- or macOS-specific lives in `crates/gymdiff-app/src/platform/{windows,macos,other}.rs`, with the same functions in each. macOS may stub Windows-only features. Always run `just check-windows` after touching platform code.
- **Riot policies**: only use the documented local APIs (LCU, Live Client Data) and the Riot API. **Never** read or write game memory, inject code, hook the game process, or automate inputs. Check `docs/Compliance/Riot policies.md` before adding a feature that shows information during a game.
- **Robustness**: the LCU API is unsupported by Riot and can change without notice. Deserialize leniently (ignore unknown fields, keep unknown enum values), never panic on API data.
- **Tests**: every behaviour in `gymdiff-core` gets a unit test. New API payloads get a fixture-based parsing test. New simulator scenarios are validated automatically.
- **Lints**: the workspace uses `clippy::pedantic`; `just clippy` must pass with `-D warnings`. `unsafe` is denied.
- **Docs**: when a change alters behaviour, architecture or a workflow, update the matching note in `docs/` in the same change. Record significant decisions as a new ADR (template: `docs/_templates/ADR.md`).
- **Slint licensing**: keep the `AboutSlint` widget reachable from the app's top level (ADR-0001).
- **Commits**: small, imperative mood, English.

## Implementing a feature

1. Read the feature note in `docs/Product/Features/` and any linked ADRs.
2. Read the matching frames in `design/*.pen` through the Pencil MCP (structure, sizes, tokens, states).
3. Add or update tokens in `docs/Design/Tokens.md` **and** `ui/theme.slint` if the design introduces new ones.
4. Implement: core logic + tests first, then data source, then UI.
5. Add or extend a simulator scenario so the feature can be exercised with `just dev`.
6. Run `just check`. Note anything that needs testing on the Windows PC in the PR/commit description.
