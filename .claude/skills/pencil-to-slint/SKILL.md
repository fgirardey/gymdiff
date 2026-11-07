---
name: pencil-to-slint
description: Use when turning a Pencil frame from design/gymdiff.pen into Slint markup, or checking a .slint screen against its design — reads the frame through the Pencil MCP, maps values to Theme tokens and places components in the ui/ layout
---

# Pencil frame → Slint

`.pen` files are encrypted: access them **only** through the Pencil MCP tools (`mcp__pencil__*`), never `Read`/`Grep`. The Pencil app must be running with `design/gymdiff.pen` open; if `get_app_state` fails, ask the user to open it.

1. **Find the frame**: names follow `<Feature> / <Screen> / <State>` (conventions in `design/README.md`; screen map in `docs/Product/Screens.md`). Read every state of the screen, not just the default one.
2. **Read structure and values**: hierarchy, layout direction, sizes, padding, gaps, colors, text styles, icons. Read reusable components on the `Components` page once and reuse them.
3. **Map to tokens**: every color, spacing, radius and font size must match a `Theme.*` token (`crates/gymdiff-app/ui/theme.slint`). A value with no token: add it to `docs/Design/Tokens.md` and `theme.slint` (same name, kebab-case) — or ask if it looks like a design mistake. Never hard-code.
4. **Write the preview first** (`docs/Workflow/Previews.md`): in `<name>.preview.slint` next to the component file, one story per frame named after it (`History / Main / Empty` → `HistoryMainEmpty`, a `Rectangle` at the frame size with the frame's texts as data, through `Fixtures` when Rust sends them), then the playground and the `Preview` gallery last.
5. **Place the code** (rules in `.claude/rules/slint.md`):
   - one exported component per file, declared first after the imports;
   - screens in `ui/screens/`, app shell in `ui/shell/`, generic widgets in `ui/components/`;
   - a sub-component specific to a screen goes in its own file next to it (`welcome-rule.slint`);
   - icons and images in `ui/images/`, referenced with `@image-url` relative to the `.slint` file. Never commit Riot logos (`docs/Compliance/Riot policies.md`).
6. **Wire it**: the component stays pure, state as `in property`, actions as `callback`; only the root `app-window.slint` binds app state, routes screens and re-exports types Rust needs.
7. **Check**: `cargo test -p gymdiff-app --test previews`, then `just preview <dir>/<name> <Story>` side by side with the Pencil frame until they match; finally `just dev`.
