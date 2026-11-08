---
status: proposed
date: 2026-11-08
---

# ADR-0008 Versioning and releases

## Context
Until now every build said `0.1.0` (the crate version), whether it came from a pull request, `main` or a developer's Mac. Sentry got the same release for all of them, and its issues had no stack trace ([[ADR-0006 Opt-in error reports]] sent the message only, and the release profile stripped the symbols). An issue said *what* went wrong, not *where* nor *in which build*. Friends run CI artifacts from pull requests, so a build must be traceable to its commit even without a formal release.

## Decision
**SemVer bumped from the Conventional Commits by release-please, every build stamped with a build number, commit and channel, and debug files plus sources uploaded to Sentry from CI.** Decided with the author on 2026-11-08.

| Piece | Source | Example |
|---|---|---|
| Version | `[workspace.package] version` in `Cargo.toml`, written by **release-please** | `0.2.0` |
| Build number | `git rev-list --count HEAD`: grows with every commit on `main`, same for every build of a commit, also works locally | `57` |
| Commit | `BUILD_SHA` in CI (the PR head, not GitHub's merge commit), else `git rev-parse HEAD` | `a1b2c3d` |
| Channel | `GYMDIFF_CHANNEL` at build time: `release` (tagged release), `preview` (CI artifact), unset → `development` | |

Derived strings (`crates/gymdiff-app/src/build_info.rs`):
- full version `0.2.0+57.a1b2c3d`, printed by `gymdiff --version` and `just version`;
- About screen `0.2.0 (build 57 · a1b2c3d)`, with `· preview` or `· dev` outside releases;
- Sentry release `gymdiff@0.2.0+57` (Sentry orders it as SemVer), Sentry environment = channel (`production`, `preview`, `development`);
- Windows file version `0.2.0.57`, product version = full version (Explorer → Properties → Details).

Releases:
1. release-please keeps a `chore: release x.y.z` pull request open on `main`, with `CHANGELOG.md` (features, fixes, performance) and the version bumped in `Cargo.toml` and `Cargo.lock`. Before 1.0, a breaking change bumps the minor version.
2. Merging it tags `vx.y.z` and creates the GitHub Release; `release.yml` builds that commit with channel `release`, attaches `gymdiff-x.y.z-windows-x64.exe`, the recorder, `gymdiff-x.y.z-macos-arm64.tar.gz` and `SHA256SUMS`, then finalizes the Sentry release and records a `production` deploy.

Symbols:
- Release builds keep line-level debug info in a separate file (`debug = "limited"`, `split-debuginfo = "packed"`): `gymdiff.pdb` on Windows, `gymdiff.dSYM` on macOS. They are uploaded, never shipped.
- **Every** CI release build (pull requests included, since friends run them) uploads its executable, debug file and a source bundle with `sentry-cli debug-files upload --include-sources`. Sentry matches them to events by debug id, so the release name does not matter for symbolication.
- The Windows build also creates the Sentry release and links its commits (`set-commits --local`).

## Alternatives considered
- **Manual `just release` with git-cliff** — full control, no bot, but one more thing to remember; release-please reads the commits we already lint.
- **release-please `rust` release type** — does not handle a virtual workspace with `version.workspace = true`; the `simple` type with TOML `extra-files` does.
- **release-plz** — Rust-native, but built around publishing to crates.io; we publish nothing.
- **GitHub run number as build number** — differs between the CI and release workflows for the same commit, and does not exist locally.
- **Version only, no symbols** — keeps the binaries smaller to build, but leaves Sentry issues without a place in the code, which was the point.

## Consequences
- Secrets and variables to set once: `SENTRY_AUTH_TOKEN` (secret), `SENTRY_ORG` and `SENTRY_PROJECT` (variables), `RELEASE_PLEASE_TOKEN` (secret, a personal token so CI runs on the release pull request). Without the Sentry token builds still succeed, without symbols.
- CI checks out the full history (`fetch-depth: 0`) for the build number.
- The release commit `chore: release x.y.z` is exempt from commitlint (its body is the changelog).
- The source of the app is stored in Sentry (EU region) with the symbols, to show the code around each frame.
- The first release PR will be `v0.1.0`, with the whole history in the changelog.
- How to release and check a symbolicated issue: [[Releasing]].
