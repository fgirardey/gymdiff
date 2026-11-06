# Development

Daily loop on **macOS**. The Windows PC is only needed for what the Mac can't show (see [[Windows testing]]).

## One-time setup (Mac)
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # rustup; the toolchain is pinned by rust-toolchain.toml
brew install just
cargo install slint-viewer                                        # optional: live preview of .slint files
```
`rust-toolchain.toml` also installs the `x86_64-pc-windows-msvc` target, used by `just check-windows`.

Recommended editor extensions are listed in `.vscode/extensions.json` (rust-analyzer, Slint).

## The loop
```mermaid
flowchart LR
    A[Feature note<br/>Obsidian] --> B[Frames<br/>Pencil]
    B --> C[Implement<br/>Claude Code]
    C --> D[just dev<br/>simulator]
    D --> E[just check]
    E --> F[Push → CI<br/>Windows build]
    F --> G[Test on PC<br/>real game]
    G -- fixtures, bugs, logs --> A
```

1. **Spec**: write or update the feature note ([[Spec to code]]).
2. **Design**: frames in Pencil ([[Design with Pencil]]).
3. **Implement** with Claude Code (rules in `CLAUDE.md`).
4. **Run** `just dev`: builds and starts the simulator at 10× speed with the default scenario, then the app. Use `just dev 1` for real time, or `just dev 10 fixtures/scenarios/<name>/scenario.json` for another scenario.
5. **Iterate on UI** with `just preview` (Slint live reload).
6. **Check** with `just check` before committing: format, clippy (pedantic, warnings denied), tests, Windows type-check.
7. **Push**: CI builds and tests on Windows and macOS, and publishes `gymdiff.exe` as an artifact.
8. **Validate on Windows** when the change touches platform code, overlays or real-client behaviour.

## Useful commands
| Command | What |
|---|---|
| `just dev [speed] [scenario]` | simulator + app |
| `just sim --speed 5` / `just app-sim` | simulator and app in separate terminals |
| `just run` | app against a real League client (also works with the macOS client) |
| `just test` | all tests |
| `just check-windows` | type-check the Windows build |
| `RUST_LOG=trace just dev` | verbose logs |

## Branches and commits
- `main` is always green in CI.
- Small commits, imperative mood, English (`Add champ select timer state`).
- Design-only commits: `design: …`. Docs-only commits: `docs: …`.
