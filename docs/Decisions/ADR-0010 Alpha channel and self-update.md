---
status: superseded by [[ADR-0011 Windows installer and updates with Velopack]]
date: 2026-11-08
---

# ADR-0010 Alpha channel and self-update

## Context
The author tests GymDiff on a Windows gaming PC while developing on macOS. Getting each merge there meant downloading a CI artifact by hand or building over SSH ([[Windows testing]]). The repository is now public ([[ADR-0009 Open-source license]]), so release assets download without a token, and every merge to `main` already produces a symbolicated Windows build ([[ADR-0008 Versioning and releases]]). Product behaviour: [[Auto-update]].

## Decision
**`main` builds are the `alpha` channel. CI publishes each one as its own GitHub pre-release `v<version>-alpha.<build>` with a `manifest.json`; a build finds the latest release of its channel through the GitHub API and replaces its own executable, which takes effect at the next launch.** Decided with the author on 2026-11-08 (revised the same day: one release per build instead of a single `alpha` release whose tag moved).

- **Channel baked in**: `ci.yml` builds `main` with `GYMDIFF_CHANNEL=alpha` (pull requests built on demand stay `preview`). Only `alpha` builds run the updater.
- **One release per build**: after each `main` build, `publish-alpha.mjs` creates the pre-release `v0.1.0-alpha.57` (never marked latest, so it never hides a real release) as a draft, attaches two assets, then publishes it, so clients never see it half uploaded:
  - `gymdiff-0.1.0-alpha.57-windows-x64.exe`,
  - `manifest.json`: `{ "version", "build", "commit", "target", "url", "size", "sha256", "published_at" }` (`release-manifest.mjs`).

  Tags are immutable. CI keeps the **last 10** alpha releases and deletes older ones with their tags. release-please is not confused: it only matches the exact version of its manifest (`v0.1.0`). Tagged releases (`v0.1.0`) get the same `manifest.json` (`publish-release.mjs`).
- **Finding the latest release**: the app lists `https://api.github.com/repos/fgirardey/gymdiff/releases` (no token: 60 requests an hour, plenty for one check an hour; over the limit, Settings says so). Alpha channel: the published pre-release with the **highest build number** in its tag, so a slow publish never hides a newer build. Stable channel: the newest non-prerelease `v<version>`. Drafts and releases without `manifest.json` are skipped. `GYMDIFF_UPDATE_URL` points at a test list.
- **Newer** = higher build number in the manifest (commit count, monotonic on `main`) for the same target triple.
- **Stable channel**: built in, off (`STABLE_UPDATES`); release builds will follow the stable releases once the manifest is signed.
- **Install**: download next to the executable, check size and SHA-256, then swap with the `self-replace` crate (on Windows it renames the running executable, which Windows allows, and puts the new one in its place). The running process is untouched; the new build starts at the next launch. Downloaded by the app, the file carries no Mark of the Web, so SmartScreen does not warn.
- **When**: shortly after launch, every hour, and on **Check now** in Settings.
- **Code**: `gymdiff-app/src/updater/` (manifest, check, download, swap), state pushed to the Settings screen like the watcher pushes client status. Network through the existing `reqwest` client.

## Alternatives considered
- **A single `alpha` release whose tag moves** (the first version of this ADR) — a fixed URL, no API, but tags were rewritten on every merge and older builds disappeared; one immutable release per build is easier to follow and to roll back from.
- **Restart by itself when idle** — always on the newest build, but the window would close on its own; installing at the next launch is enough for testing.
- **Channel in `settings.json`** — one build for all channels, but easy to forget and needs a stable feed to make sense; a dedicated alpha build is installed once and done.
- **Signed manifest** (minisign) — protects against a compromised GitHub account or release; worth adding before a stable channel reaches other players, overkill for the author's own PC today.
- **An updater framework** (Velopack, cargo-packager) — installers, delta updates, signing; heavy for a single portable `.exe`. To revisit with the Windows installer ([[Code signing]]).

## Consequences
- `ci.yml` gets a job on `main` that needs `contents: write` (creating releases and tags, deleting old alpha releases), and the Windows `main` build carries channel `alpha` (About shows `· alpha`).
- New dependencies in `gymdiff-app`: `self-replace`, `sha2`.
- Integrity relies on GitHub (HTTPS, the repository's access) and the SHA-256 in the manifest; no signature yet.
- The executable must sit in a folder the player can write to; Program Files would fail with a clear message.
- The stable channel (tagged releases, for friends) reuses the same updater with another manifest, once a release exists and a signature is in place.
