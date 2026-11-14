---
name: implementing-a-feature
description: Use when implementing or extending a cooldown.lol feature described in docs/Product/Features/ — reads the spec and Pencil frames, then builds core logic, data source, UI and a simulator scenario in that order
---

# Implementing a feature

1. **Spec**: read the note in `docs/Product/Features/<Feature>.md` and every ADR it links. If the feature shows anything during a game, check `docs/Compliance/Riot policies.md`. Large feature: list open questions and ask before writing code.
2. **Design**: read the frames `<Feature> / <Screen> / <State>` in `design/cooldown.pen` through the Pencil MCP (see the `pencil-to-slint` skill). Every state in the note should have a frame; if one is missing, ask.
3. **Tokens**: new values in the design → add them to `docs/Design/Tokens.md` **and** `crates/cooldown-app/ui/theme.slint` before using them.
4. **Implement**, in this order:
   1. `cooldown-core`: new `Event` variants, reducer changes, unit tests.
   2. Data source: `cooldown-lol` (models with lenient serde + fixture parsing test, `LolClient` method) and the app's `watcher.rs`.
   3. UI: previews first (one story per state frame, see `docs/Workflow/Previews.md`), then the `.slint` components until each story matches its frame (see the `pencil-to-slint` skill), then wire properties/callbacks in Rust.
5. **Simulator**: add or extend a scenario so the feature runs in `just dev` (see the `writing-a-simulator-scenario` skill).
6. **Verify**: `just check` passes; if possible run `just dev` and compare with the frames.
7. **Docs**: update the feature note if behaviour changed during implementation; write an ADR for any significant trade-off.
8. **Commit** with the `committing-with-commitlint` skill. In the description, list what must be tested on the Windows PC (`docs/Workflow/Windows testing.md`).
