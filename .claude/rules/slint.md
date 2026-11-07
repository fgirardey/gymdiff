---
paths:
  - "crates/gymdiff-app/ui/**"
---

# Slint UI

- Use `Theme.*` tokens (`ui/theme.slint`) for every color, spacing, radius and font size; never hard-code them. A missing token is added to `docs/Design/Tokens.md` **and** `theme.slint` first.
- Slint resolves names in declaration order, so each file **exports one main component** (or global / enum), declared first after the imports. A sub-component it needs goes in its own file: next to its parent when specific (`screens/welcome-rule.slint`), in `components/` when reusable.
- Layout: `app-window.slint` (root, routing; re-exports what Rust needs, since `include_modules!` only sees the root's exports), `screen.slint`, `components/` (generic widgets), `shell/` (app shell, top bar), `screens/` (one file per screen), `preview-kit/` (preview helpers, never imported by the app), `images/` (`@image-url` paths are relative to the `.slint` file).
- Screen and shell files start with a `//` comment naming their Pencil frame(s) in `design/gymdiff.pen`.
- Components in `components/`, `shell/` and `screens/` are **pure**: state comes in through `in`/`in-out` properties, actions go out through callbacks; they never read an app-state global (only `Theme` and asset globals like `RoleIcons`). Structured data uses structs from `ui/models.slint`. Only `app-window.slint` binds app state.
- Every component, shell part, screen and `app-window.slint` has a preview `<name>.preview.slint` **next to it**, written before or with the component (see `docs/Workflow/Previews.md`). In order: one story per Pencil frame (named after the frame, `HistoryMainEmpty`, `Rectangle` at the frame size), a `<Component>Playground` window with `in-out` knobs and `playground.log(...)` in callbacks when the component has inputs or callbacks, then the `Preview` gallery (last, built from `PreviewCase`). Shared sample data goes in `preview-kit/fixtures.slint`, matching what Rust sends. App files never import a preview or `preview-kit/`. `cargo test -p gymdiff-app --test previews` enforces all of this.
- Keep `AboutSlint` reachable from the app's top level (ADR-0001, Slint license).
- To translate a Pencil frame, use the `pencil-to-slint` skill.
