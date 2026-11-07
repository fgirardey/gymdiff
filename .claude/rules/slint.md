---
paths:
  - "crates/gymdiff-app/ui/**"
---

# Slint UI

- Use `Theme.*` tokens (`ui/theme.slint`) for every color, spacing, radius and font size; never hard-code them. A missing token is added to `docs/Design/Tokens.md` **and** `theme.slint` first.
- Slint resolves names in declaration order, so each file **exports one main component** (or global / enum), declared first after the imports. A sub-component it needs goes in its own file: next to its parent when specific (`screens/welcome-rule.slint`), in `components/` when reusable.
- Layout: `app-window.slint` (root, routing; re-exports what Rust needs, since `include_modules!` only sees the root's exports), `screen.slint`, `components/` (generic widgets), `shell/` (top bar), `screens/` (one file per screen), `images/` (`@image-url` paths are relative to the `.slint` file).
- Screen and shell files start with a `//` comment naming their Pencil frame(s) in `design/gymdiff.pen`.
- Keep `AboutSlint` reachable from the app's top level (ADR-0001, Slint license).
- To translate a Pencil frame, use the `pencil-to-slint` skill.
