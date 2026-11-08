---
paths:
  - "crates/**/*.rs"
  - "**/Cargo.toml"
---

# Rust

- Lints: the workspace uses `clippy::pedantic`; `just clippy` must pass with `-D warnings`. `unsafe` is denied. Prefer fixing over `#[allow]`; when an `allow` is justified, scope it to the item and add a comment saying why.
- File order: module doc (`//!`), `mod`/`use`, the main item, its `impl`s right after it (inherent first, then trait impls), other public items, private helpers, `#[cfg(test)] mod tests` last. In a binary, `main()` comes first.
- One logical unit per file, named after its main item (`http_client.rs` → `HttpLolClient`). `lib.rs` declares modules and re-exports the public API; no logic in it.
- Log levels follow `docs/Workflow/Logging.md`: log state changes, not loop iterations; `ERROR` only for bugs (each one becomes a Sentry issue); a missing or closing League client is never a warning.
- Never panic on external data (API payloads, files, env vars): return errors or fall back.
- After a change, run `just check` (fmt, clippy, tests, Windows type-check).
