# cooldown.lol

A Windows desktop companion app for **League of Legends**, written in Rust with a native [Slint](https://slint.dev) UI.

> Status: project skeleton. The product definition lives in [`docs/Product/Vision.md`](docs/Product/Vision.md).

## How this repository works

| What | Where | Tool |
|---|---|---|
| Product definition | [`docs/Product/`](docs/Product/) | Obsidian (the `docs/` folder is a vault) |
| Visual design | [`design/`](design/) | [Pencil](https://pencil.dev) (`.pen` files, read by Claude through Pencil's MCP server) |
| Decisions | [`docs/Decisions/`](docs/Decisions/) | ADRs in Markdown |
| Implementation | [`crates/`](crates/) | Rust, Claude Code (see [`CLAUDE.md`](CLAUDE.md)) |

Development happens on **macOS** against a **simulated League client**; builds are tested on a **Windows PC**. See [`docs/Workflow/Development.md`](docs/Workflow/Development.md).

## Quick start (macOS)

```sh
# Prerequisites
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # Rust (toolchain pinned by rust-toolchain.toml)
brew install just                                                 # task runner

# Open the League client, log in, then run the app against it
just dev
```

The top bar shows the client's phase (lobby, champion select, in game…) and History lists your recent games. The macOS client is enough for everything except what is Windows-only.

Other recipes: `just` lists them all.

```sh
just check          # fmt + clippy + tests + Windows type-check — run before committing
just win-build      # build on the Windows PC over SSH
```

## Layout

```
crates/
  cooldown-core/       pure domain logic (no I/O, no OS code)
  cooldown-lol/        League local APIs (lockfile, LCU, Live Client Data)
  cooldown-app/        Slint UI + OS integration
docs/                 Obsidian vault (product, architecture, decisions, workflows)
design/               Pencil design files
```

## Legal

cooldown.lol isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc.

The UI is built with Slint under the Slint Royalty-free Desktop License (see [ADR-0001](docs/Decisions/ADR-0001%20Native%20UI%20with%20Slint.md)).

## License

cooldown.lol is licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option. Unless you explicitly state otherwise, any contribution you submit for inclusion in cooldown.lol is licensed the same way, without any additional terms.

Riot Games assets, the cooldown.lol name and logo, and the Lucide settings icon are excluded: see [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).
