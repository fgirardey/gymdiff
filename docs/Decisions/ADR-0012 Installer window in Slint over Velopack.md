---
status: proposed
date: 2026-11-08
---

# ADR-0012 Installer window in Slint over Velopack

## Context
[[ADR-0011 Windows installer and updates with Velopack]] gave GymDiff an installer, but Velopack's `Setup.exe` is "one-click": it shows no choices, and its shortcuts are fixed when the package is built. The author wants the player to choose the Start menu and Desktop shortcuts while installing, and to change them later. Velopack's MSI shows a few dialogs, none for shortcuts. Product behaviour: [[Installation]].

## Decision
**GymDiff ships its own installer window, `gymdiff-installer`, written in Rust with Slint like the app. It asks the choices, runs Velopack's `Setup.exe --silent` that it embeds, then creates the chosen shortcuts itself. Velopack keeps the install, the updates and the uninstaller.** Decided with the author on 2026-11-08.

- **Crate** `crates/gymdiff-installer` (binary). Its screens live in `crates/gymdiff-app/ui/installer/`, next to the app's theme and components, so they follow the same rules and previews; its `build.rs` compiles them.
- **Payload**: CI builds the installer after `vpk pack`, with `GYMDIFF_VELOPACK_SETUP` pointing at Velopack's `GymDiff-<channel>-Setup.exe`; `build.rs` embeds it. The installer then replaces Velopack's setup under the same name, so the release still offers one `GymDiff-<channel>-Setup.exe`. A build without a payload (local, previews) shows the window but cannot install.
- **Install**: writes the embedded setup to a temporary folder and runs it with `--silent` (no Velopack window, no launch afterwards), waits for it and maps its exit to Done or Failed.
- **Shortcuts**: `vpk pack --shortcuts None`; shortcuts are ours, named `GymDiff.lnk`, pointing at Velopack's launcher `%LOCALAPPDATA%\GymDiff\gymdiff.exe` (stable across updates), icon from `current\gymdiff.exe`. Start menu: `%APPDATA%\Microsoft\Windows\Start Menu\Programs`; Desktop: the user's known Desktop folder (OneDrive redirection included). Written with the `mslnk` crate, folders found with `dirs`. Shared by the installer and the app in a small crate, `gymdiff-shortcuts`.
- **Later and uninstall**: Settings switches create or remove the shortcuts (installed builds only); GymDiff's Velopack hook before uninstall removes them.
- **Launch**: on Finish, the installer starts the launcher if "Launch GymDiff" is on.

## Alternatives considered
- **Velopack's MSI** — dialogs for welcome, license and scope, none for shortcuts, and no way to add one.
- **cargo-packager (NSIS)** — a classic wizard with custom pages, but it replaces Velopack (no delta updates, another updater) and the pages are NSIS script.
- **Inno Setup with Velopack's portable zip** — checkboxes built in, but two systems own the install folder and the Apps entry goes stale.
- **Choices in GymDiff's welcome screen** — no installer to write, but the choice comes after installing, not in the installer the author asked for.

## Consequences
- Two Windows executables to sign later (installer and app).
- The `publish-alpha` and `package-windows` jobs also build the installer (Rust on the Windows runner) and swap it in before `vpk upload`.
- The installer is a third UI surface to design in Pencil and preview.
- Shortcuts we create are ours to remove: the uninstall hook must keep working, or they would point at nothing.
