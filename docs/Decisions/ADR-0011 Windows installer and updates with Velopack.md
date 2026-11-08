---
status: proposed
date: 2026-11-08
---

# ADR-0011 Windows installer and updates with Velopack

## Context
GymDiff shipped as a bare `gymdiff.exe` to copy into a folder by hand, with a home-made updater ([[ADR-0010 Alpha channel and self-update]]) that swapped its own executable. The author wants GymDiff to be **installable** on Windows: an installer, Start menu and Desktop shortcuts, an entry in *Settings → Apps* with an uninstaller. An installer and an updater must agree on where the app lives and how it is replaced, so both are decided together. Product behaviour: [[Installation]], [[Auto-update]].

## Decision
**[Velopack](https://velopack.io) installs and updates GymDiff on Windows. CI packages every alpha and stable build with `vpk` and publishes it on GitHub Releases; the app checks the releases of its channel with Velopack's `UpdateManager`.** Decided with the author on 2026-11-08. Supersedes ADR-0010.

- **Versions**: crate `velopack` and tool `vpk` pinned to the same version (1.2.161).
- **Install**: per user, no administrator rights, in `%LOCALAPPDATA%\GymDiff` (`current\gymdiff.exe` and Velopack's `Update.exe`). Pack id `GymDiff`, title `GymDiff`, icon `design/brand/gymdiff.ico`. Shortcuts: chosen by the player in our installer window and made by GymDiff, not by Velopack (`--shortcuts None`), see [[ADR-0012 Installer window in Slint over Velopack]]. One GymDiff per PC: the installer you run (alpha or stable) picks the channel; installing the other one replaces it.
- **Uninstall**: Velopack deletes `%LOCALAPPDATA%\GymDiff`, so the logs there go too. Settings, consent and install id stay in `%APPDATA%\GymDiff` and survive a reinstall.
- **Channels and versions**: alpha packages are `<version>-alpha.<build>` on channel `alpha` (e.g. `0.1.0-alpha.60`), stable packages `<version>` on channel `stable`. Velopack compares SemVer within a channel, so alpha builds order by build number.
- **Publishing (CI)**: on a Windows runner, `vpk download github` fetches the previous package of the channel (for a delta), `vpk pack` builds the installer `GymDiff-<channel>-Setup.exe`, the portable zip, the full and delta packages and the feed `releases.<channel>.json`, and `vpk upload github` attaches them:
  - alpha: to a new pre-release `v<version>-alpha.<build>` (never marked latest), created and published by `vpk`; CI then keeps the last 10 alpha releases;
  - stable: merged into the `v<version>` release created by release-please.
- **App**: `VelopackApp::build().run()` runs first in `main` (after the crash reporter's own early exit), handles Velopack's install and uninstall hooks, and applies a downloaded update at startup. The updater (`gymdiff-app/src/updater/`) builds an `UpdateManager` on `GithubSource` (pre-releases included on the alpha channel); it is on only when GymDiff was installed by Velopack, on the alpha channel (stable later, `STABLE_UPDATES`). It checks 5 s after launch, every hour and on **Check now**, and downloads in the background; Velopack applies the update the next time GymDiff starts.
- **Kept from ADR-0010**: channel baked into the build (`GYMDIFF_CHANNEL`), checks timing, the Settings section and its states, one release per build with immutable tags, last 10 alpha releases.
- **Removed**: our manifest (`manifest.json`, `release-manifest.mjs`), the executable swap (`self-replace`), our release-list parsing and `GYMDIFF_UPDATE_URL`.

## Alternatives considered
- **Inno Setup + our updater** — a classic per-user installer and no new framework, but full downloads only, and two pieces (installer, updater) to keep in agreement by hand.
- **MSI (WiX)** — built for per-machine installs in Program Files, where an app cannot update itself without admin rights.
- **MSIX / Microsoft Store** — signing mandatory, sandboxed, Store review; too much for a free community app today.
- **Keep the bare executable** (ADR-0010) — no install step, but no shortcuts, no uninstaller, and copying files by hand.

## Consequences
- CI installs `vpk` (a .NET tool) on the Windows runner; the `publish-alpha` job and a new stable packaging job in `release.yml` run on Windows.
- The installer is **not signed** yet: downloaded in a browser, `GymDiff-<channel>-Setup.exe` triggers SmartScreen ("More info → Run anyway"). Updates downloaded by Velopack do not. SignPath Foundation (free for open source, now that the repository is public) is the next step ([[Code signing]]).
- Builds not installed by Velopack (local builds, the raw `gymdiff.exe` of a pull request, the portable zip) do not update themselves and hide the Updates section.
- Alpha builds installed with ADR-0010's updater keep finding only their own manifest releases: install `GymDiff-alpha-Setup.exe` once by hand (delete the old `.exe` from `%LOCALAPPDATA%\GymDiff` first).
- Testing updates needs an installed build: on the Windows PC, after a merge. There is no local feed on macOS any more.
