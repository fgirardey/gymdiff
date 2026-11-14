---
status: proposed
date: 2026-11-13
---

# ADR-0015 Rename to cooldown.lol

## Context
The product started as **GymDiff**. Its new identity, **cooldown.lol**, is already in the design and in [[Brand]]: an ability on cooldown in League, and the cool-down after a workout. The code, the docs, the build and the installer still said GymDiff. Alpha builds are installed on a few PCs (Velopack pack id `GymDiff`, data in `%APPDATA%\GymDiff`).

## Decision
- **Visible name**: `cooldown.lol`, lowercase, everywhere a person reads it (window and installer titles, tray menu, notifications, Windows file properties, shortcuts `cooldown.lol.lnk`, startup entry `cooldown.lol`).
- **Code identifiers**: `cooldown`: crates `cooldown-*` (`cooldown_core`…), executable `cooldown.exe`, environment variables `COOLDOWN_*`, Sentry release `cooldown@<version>+<build>`, design file `design/cooldown.pen`, icon `design/brand/cooldown.ico`.
- **Folders and installer**: Velopack pack id `cooldown.lol` (install in `%LOCALAPPDATA%\cooldown.lol`, setup `cooldown.lol-<channel>-Setup.exe`); data in `%APPDATA%\cooldown.lol` (`~/Library/Application Support/cooldown.lol` on macOS), logs in `%LOCALAPPDATA%\cooldown.lol\logs`.
- **Clean break**: nothing migrates from GymDiff. The few alpha installs are uninstalled by hand and replaced by the new installer; their history and settings start empty.
- **GitHub repository**: `fgirardey/cooldown.lol` (renamed from `fgirardey/gymdiff`), which the updater reads.
- **Sentry project**: `cooldown` (renamed from `gymdiff`), set in the `SENTRY_PROJECT` variable.
- **Local checkouts** (such as `C:\dev\gymdiff`) keep their folder name: only a default path in the `justfile`.

## Alternatives considered
- **Migrate the data folder** on first launch — keeps the alpha history, but adds code that only serves a handful of testers, once.
- **Keep the internal ids** (pack id, data folder, startup entry) as GymDiff so updates keep flowing — no reinstall, but the old name stays visible in folders, Apps & features and the Registry for good.
- **`cooldownlol` as the code identifier** — unambiguous but heavier; `cooldown` reads well in crate names and the dotted name stays for what people see.

## Consequences
- An alpha build of GymDiff never updates to cooldown.lol: install `cooldown.lol-alpha-Setup.exe` once by hand after uninstalling GymDiff (Settings → Uninstall, or Apps & features).
- GitHub redirects `fgirardey/gymdiff` to `fgirardey/cooldown.lol`, so links and clones made before the rename keep working.
- Sentry releases get a new prefix (`cooldown@…`); older `gymdiff@…` releases stay in the same project.
- Accepted and superseded ADRs keep saying GymDiff: they are never edited.
- The name and logo stay outside the code license ([[ADR-0009 Open-source license]]).
