# CLAUDE.md

GymDiff is a **Windows desktop companion app for League of Legends**, in **Rust** with a native **Slint** UI. It reads the game's local APIs (LCU + Live Client Data). Development happens on **macOS** against a **simulator**; the real target is **Windows**.

Everything in the repository is in **English**: code, comments, commits, docs.

Instructions are split so that only what is relevant gets loaded:
- this file: what applies everywhere;
- `.claude/rules/*.md`: rules per area (Rust, core, lol, platform, Slint, docs), loaded when working on matching files;
- `.claude/skills/*/SKILL.md`: step-by-step workflows (implementing a feature, Pencil → Slint, simulator scenarios, committing).

## Sources of truth
In this order; never invent product behaviour or visuals they don't describe, ask instead:
1. **Product**: `docs/Product/` (Obsidian vault), one note per feature in `docs/Product/Features/`.
2. **Design**: `design/*.pen`, read only through the **Pencil MCP** (never `Read`/`Grep` a `.pen`).
3. **Design tokens**: `docs/Design/Tokens.md`, mirrored in `crates/gymdiff-app/ui/theme.slint`.
4. **Decisions**: `docs/Decisions/ADR-*.md`. Respect them; propose a new ADR to change one.

If code, docs and design disagree, stop and ask which one is right.

## Repository layout
```
crates/gymdiff-core/       Pure domain logic (reducer: CompanionState::apply(Event)).
crates/gymdiff-lol/        League local APIs: lockfile, LolClient trait, HTTP client, models.
crates/gymdiff-simulator/  Fake League client replaying fixtures/scenarios/*.json.
crates/gymdiff-app/        Slint UI (ui/) + watcher + OS integration (src/platform/ only).
fixtures/                  Scenarios for the simulator and tests.
docs/                      Obsidian vault: product, architecture, decisions, workflows.
design/                    Pencil design files (.pen).
```

## Commands
```sh
just dev            # app + simulator (main dev loop on macOS)
just check          # fmt-check + clippy + tests + Windows type-check: run before committing
just test           # cargo test --workspace
just check-windows  # cargo check --target x86_64-pc-windows-msvc
just preview        # live preview of a .slint file (needs slint-viewer)
```

## Rules that apply everywhere
- **Riot policies**: only the documented local APIs (LCU, Live Client Data) and the Riot API. **Never** read or write game memory, inject code, hook the game process, or automate inputs. Check `docs/Compliance/Riot policies.md` before adding anything shown during a game.
- **Architecture**: data sources produce `Event`s, `gymdiff-core` reduces them into `CompanionState`, the UI only renders state. Talk to the game only through `LolClient`.
- **Code organisation**: one logical unit per file; the main item comes first, right after the imports (public API, then private helpers, then tests). Details: `docs/Workflow/Code organisation.md`.
- **Docs**: a change to behaviour, architecture or a workflow updates the matching note in `docs/` in the same change; significant decisions get a new ADR.
- **Commits**: small, Conventional Commits, enforced by commitlint. Use the `committing-with-commitlint` skill; never `--no-verify`.
