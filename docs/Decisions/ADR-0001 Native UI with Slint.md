---
status: accepted
date: 2026-11-06
---

# ADR-0001 Native UI with Slint

## Context
GymDiff is a Rust desktop app for Windows, developed on macOS. The UI must:
- look custom and polished (it will be designed in Pencil, not built from stock widgets),
- render identically on macOS (development) and Windows (target),
- support always-on-top / transparent windows later for in-game overlays,
- stay light: it runs next to the game.

## Decision
Use **[Slint](https://slint.dev)** (crate `slint`, currently 1.x) for the UI, with `.slint` markup files compiled at build time by `slint-build`.

## Alternatives considered
- **Tauri (web UI)** — easiest path from design to code, but the UI runs in WebKit on macOS and WebView2 on Windows: rendering differs between the development and target platforms, and it embeds a browser engine.
- **egui** — very fast to prototype, but immediate-mode UIs are hard to make match a precise visual design.
- **iced** — pure Rust and themeable, but less mature for multi-window and overlay use cases.

## Consequences
- The UI is described in `crates/gymdiff-app/ui/*.slint`; design tokens live in `ui/theme.slint` (mirroring [[Tokens]]).
- Fast UI iteration with `slint-viewer --auto-reload` (`just preview`) and the Slint VS Code extension's live preview.
- No design tool exports Slint code: Claude Code translates Pencil frames into `.slint` by hand, using the structure and values read through the Pencil MCP.
- **Licensing**: Slint is available under GPLv3, a Royalty-free license for desktop/mobile/web apps, or a commercial license. GymDiff uses the **Slint Royalty-free Desktop, Mobile, and Web Applications License**, which requires:
  - displaying the `AboutSlint` widget in an About screen reachable from the top level of the app (done: *About* button in the main window),
  - showing the Slint attribution badge on the public web page where GymDiff can be downloaded (to do before any public release),
  - keeping Slint's license notices intact.
  Re-check the current terms on <https://slint.dev/terms-and-conditions> before release.
