# Code organisation

How source files are split and ordered, in Rust and in Slint. The goal: open a file and see what it offers first, and never let one file collect everything.

## One logical unit per file
- A file holds **one logical unit**: a type with its `impl`s, a trait, a screen, a widget, a server. Not necessarily one item: a type and the small types that only exist to serve it (an error enum, a config struct, a constant) stay together.
- Split when a file starts holding **two things a reader would look for separately** (e.g. a trait and its HTTP implementation, an entry point and the server it starts), not at a fixed line count.
- Name the file after its main item: `http_client.rs` → `HttpLolClient`, `top-bar.slint` → `TopBar`.

## Public API first
Inside a file, read top to bottom:
1. module doc comment (`//!` in Rust, a leading `//` block in Slint),
2. `mod` / `use` / `import`,
3. **the main item** — the type, trait, function or component the file is named after,
4. its `impl` blocks, right after it (`impl Foo` first, then trait impls),
5. other public items (secondary types, constants, free functions),
6. private helpers,
7. `#[cfg(test)] mod tests` (Rust), always last.

In a binary, `main()` is the main item: it comes first, before the `Args` struct and helpers.

## Rust
- Module tree: `lib.rs` / `main.rs` declare modules and re-export the public API (`pub use`); keep logic out of `lib.rs`.
- Constants follow the item that uses them, unless several items share them.

## Slint
Slint resolves names in **declaration order**: a component or enum must be declared before it is used in the same file. So a file can't put its main component first and its helpers after. Instead:
- each file **exports one main component** (or one global / enum), declared first after the imports;
- a sub-component that the main component needs goes in **its own file**, named after its parent when it is specific to it (`welcome-rule.slint` next to `welcome.slint`), or in `components/` when it is reusable;
- the root file `app-window.slint` re-exports what Rust needs (`export { Screen }`), since `slint::include_modules!()` only sees what the root file exports;
- exception: a `*.preview.slint` file exports several stories and ends with its `Preview` gallery, which uses them (see [[Previews]]).

Layout of `crates/gymdiff-app/ui/`:

| Path | What |
|---|---|
| `app-window.slint` | `AppWindow`, the root compiled by `build.rs`. Routes between screens. |
| `screen.slint` | `Screen` enum (which screen is shown). |
| `theme.slint`, `role-icons.slint` | Globals: design tokens ([[Tokens]]), bundled images. |
| `components/` | Reusable widgets with no app knowledge (`Icon`, `Crest`, `PrimaryButton`, `BackLink`). |
| `shell/` | The app shell around the screens (`AppShell`, `TopBar` and its parts). Pencil: `App shell / …`. |
| `screens/` | One file per screen, plus its private parts (`welcome-rule.slint`). |
| `*.preview.slint` | Next to each component, shell part, screen and `app-window.slint`: its stories and `Preview` gallery. See [[Previews]]. Never imported by the app. |
| `preview-kit/` | Helpers for the previews only: `PreviewCase`, `PreviewPlayground`, `Fixtures`. |
| `images/` | SVG and PNG assets, referenced with `@image-url` relative to the `.slint` file. |
