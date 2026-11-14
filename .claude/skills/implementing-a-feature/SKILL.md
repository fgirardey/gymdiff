---
name: implementing-a-feature
description: Use when implementing or extending a cooldown.lol feature described in docs/Product/Features/ — reads the spec and Pencil frames, then builds core logic, data source and UI in that order
---

# Implementing a feature

1. **Spec**: read the note in `docs/Product/Features/<Feature>.md` and every ADR it links. If the feature shows anything during a game, check `docs/Compliance/Riot policies.md`. Large feature: list open questions and ask before writing code.
2. **Design**: read the frames `<Feature> / <Screen> / <State>` in `design/cooldown.pen` through the Pencil MCP (see the `pencil-to-slint` skill). Every state in the note should have a frame; if one is missing, ask.
3. **Tokens**: new values in the design → add them to `docs/Design/Tokens.md` **and** `crates/cooldown-app/ui/theme.slint` before using them.
4. **Implement**, in this order:
   1. `cooldown-core`: new `Event` variants, reducer changes, unit tests.
   2. Data source: `cooldown-lol` (models with lenient serde + fixture parsing test, `LolClient` method) and the app's `watcher.rs`.
   3. UI: previews first (one story per state frame, see `docs/Workflow/Previews.md`), then the `.slint` components until each story matches its frame (see the `pencil-to-slint` skill), then wire properties/callbacks in Rust.
5. **Verify**: `just check` passes; run `just dev` against the League client (open it and log in) and compare with the frames. A state you cannot reach on demand (end of game, champion select roles) is covered by unit tests on fixtures of the real payloads and checked on the Windows PC.
6. **Docs**: update the feature note if behaviour changed during implementation; write an ADR for any significant trade-off.
7. **Commit** with the `committing-with-commitlint` skill. In the description, list what must be tested on the Windows PC (`docs/Workflow/Windows testing.md`).
