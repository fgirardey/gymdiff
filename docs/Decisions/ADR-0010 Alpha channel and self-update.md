---
status: proposed
date: 2026-11-08
---

# ADR-0010 Alpha channel and self-update

## Context
The author tests GymDiff on a Windows gaming PC while developing on macOS. Getting each merge there meant downloading a CI artifact by hand or building over SSH ([[Windows testing]]). The repository is now public ([[ADR-0009 Open-source license]]), so release assets download without a token, and every merge to `main` already produces a symbolicated Windows build ([[ADR-0008 Versioning and releases]]). Product behaviour: [[Auto-update]].

## Decision
**`main` builds are the `alpha` channel. CI publishes each one to a rolling GitHub pre-release tagged `alpha`, with a `latest.json` manifest; alpha builds check it and replace their own executable, which takes effect at the next launch.** Decided with the author on 2026-11-08.

- **Channel baked in**: `ci.yml` builds `main` with `GYMDIFF_CHANNEL=alpha` (pull requests built on demand stay `preview`). Only `alpha` builds run the updater.
- **Feed**: the pre-release `alpha` (not marked latest, so it never hides a real release). After each `main` build, CI moves the `alpha` tag to the built commit and replaces two assets:
  - `gymdiff-alpha-windows-x64.exe`,
  - `latest.json`: `{ "version", "build", "commit", "target", "url", "size", "sha256", "published_at" }`.
  
  The app reads `https://github.com/fgirardey/gymdiff/releases/download/alpha/latest.json`: an asset download, not the GitHub API, so no rate limit and no token. `GYMDIFF_UPDATE_URL` overrides it for tests.
- **Newer** = higher build number (commit count, monotonic on `main`) for the same target triple.
- **Install**: download next to the executable, check size and SHA-256, then swap with the `self-replace` crate (on Windows it renames the running executable, which Windows allows, and puts the new one in its place). The running process is untouched; the new build starts at the next launch. Downloaded by the app, the file carries no Mark of the Web, so SmartScreen does not warn.
- **When**: shortly after launch, every hour, and on **Check now** in Settings.
- **Code**: `gymdiff-app/src/updater/` (manifest, check, download, swap), state pushed to the Settings screen like the watcher pushes client status. Network through the existing `reqwest` client.

## Alternatives considered
- **Tag per build** (`v0.1.0-alpha.57`) — a release per merge clutters the Releases page, and finding the newest needs the rate-limited API.
- **Restart by itself when idle** — always on the newest build, but the window would close on its own; installing at the next launch is enough for testing.
- **Channel in `settings.json`** — one build for all channels, but easy to forget and needs a stable feed to make sense; a dedicated alpha build is installed once and done.
- **Signed manifest** (minisign) — protects against a compromised GitHub account or release; worth adding before a stable channel reaches other players, overkill for the author's own PC today.
- **An updater framework** (Velopack, cargo-packager) — installers, delta updates, signing; heavy for a single portable `.exe`. To revisit with the Windows installer ([[Code signing]]).

## Consequences
- `ci.yml` gets a job on `main` that needs `contents: write` (moving the tag, replacing assets), and the Windows `main` build carries channel `alpha` (About shows `· alpha`).
- New dependencies in `gymdiff-app`: `self-replace`, `sha2`.
- Integrity relies on GitHub (HTTPS, the repository's access) and the SHA-256 in the manifest; no signature yet.
- The executable must sit in a folder the player can write to; Program Files would fail with a clear message.
- The stable channel (tagged releases, for friends) reuses the same updater with another manifest, once a release exists and a signature is in place.
