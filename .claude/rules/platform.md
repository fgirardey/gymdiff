---
paths:
  - "crates/cooldown-app/src/platform/**"
---

# Platform code

- All OS-specific code lives in `crates/cooldown-app/src/platform/{windows,macos,other}.rs`, nowhere else.
- The three modules expose **the same functions** with the same signatures; `mod.rs` re-exports the one for the current OS. Adding a function means adding it to all three.
- macOS is the development platform and may stub Windows-only features (log what is stubbed with `tracing::debug!`).
- Always run `just check-windows` after touching this folder, and list in the commit description what must be tested on the Windows PC (`docs/Workflow/Windows testing.md`).
