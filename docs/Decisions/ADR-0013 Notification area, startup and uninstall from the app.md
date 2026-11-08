---
status: proposed
date: 2026-11-08
---

# ADR-0013 Notification area, startup and uninstall from the app

## Context
GymDiff has to be running when a game ends ([[Post-game workout]]), but closing its window quit it, and it never started on its own. The author wants GymDiff to stay in the notification area when its window is closed, to optionally start with Windows (in Settings and in the installer), and to be uninstallable from GymDiff itself, with a choice to also delete the player's settings. Velopack already installs, updates and uninstalls GymDiff ([[ADR-0011 Windows installer and updates with Velopack]], [[ADR-0012 Installer window in Slint over Velopack]]). Product behaviour: [[Running in the background]], [[Installation]].

## Decision
**A `tray-icon` notification area icon keeps GymDiff alive when its window is closed; a local socket keeps it to one instance; a per-user `Run` registry value starts it with Windows; Settings starts Velopack's own uninstaller.** Decided with the author on 2026-11-08.

- **Notification area**: crate `tray-icon` (Tauri's, menus through `muda`), Windows and macOS. Created on the main thread once the event loop runs; its click and menu events are forwarded to the event loop with `slint::invoke_from_event_loop`. Menu: Open GymDiff, Quit GymDiff; a left click opens the window on Windows. Icon: `ui/images/app-icon.png`, decoded with `png`.
- **Closing**: the window hides (Slint's default close response) and the app runs `slint::run_event_loop_until_quit()` instead of `Window::run()`; **Quit GymDiff** calls `slint::quit_event_loop()`. Showing the window again un-minimises it and asks for focus through Slint's winit accessor (`unstable-winit-030`, the winit version Slint already uses).
- **One instance**: crate `interprocess`, a local socket named after the data folder (a named pipe on Windows, a socket file in the data folder on macOS), so two Windows accounts, or `GYMDIFF_DATA_DIR`, get their own. At start GymDiff connects: if someone answers, it sends `show` (or nothing when started with `--background`) and exits; otherwise it listens on a `single-instance` thread. After Velopack's hooks, so an update restart is unaffected.
- **Start with Windows**: the value `GymDiff` in `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` = `"%LOCALAPPDATA%\GymDiff\gymdiff.exe" --background` (Velopack's launcher, which forwards arguments to the current version). Enabled means the value exists and Task Manager has not disabled it in `…\Explorer\StartupApproved\Run` (odd first byte); turning it on removes that override. Crate `winreg`, in `gymdiff-shortcuts` (shared with the installer), installed builds only. The registry is the only record: no copy in `settings.json` to drift.
- **Uninstall from the app**: Settings starts `%LOCALAPPDATA%\GymDiff\Update.exe --uninstall` (the command Windows' Apps settings runs) and quits; Velopack stops any GymDiff left, runs our uninstall hook, removes the files and shows its own progress and completion messages. To delete the player's data too, GymDiff first writes the marker `delete-on-uninstall` in the data folder; the uninstall hook deletes the data folder when it finds it. The hook runs after GymDiff has been stopped, so no file is held open; an uninstall from Windows' settings finds no marker and keeps the data, as before.
- **Uninstall hook** (before uninstall): removes the shortcuts (ADR-0012), the startup entry, then the data folder if marked.

## Alternatives considered
- **Close quits, with a "minimise to tray" setting** — the author wants closing to always keep GymDiff running.
- **Startup folder shortcut** instead of the `Run` value — also per user, but Task Manager's toggle and our switch would then live in two different places to reconcile; `Run` is what most apps use.
- **Scheduled task at logon** — can delay start and survive uninstall if the hook fails; overkill without admin needs.
- **Delete the data folder from the app before starting the uninstaller** — the app still has files open (logs, crash reporter), and a failed uninstall would leave an installed GymDiff without its settings.
- **Named mutex for single instance** — tells a second instance to exit but cannot ask the first to show its window.
- **Our own uninstall code** — Velopack already removes files, the Apps entry and its folder; doing it twice risks drifting from what Windows' uninstall does.

## Consequences
- New dependencies: `tray-icon` (pulls a second `muda` version next to Slint's), `interprocess`, `winreg` (Windows only), `png` at runtime; Slint's `unstable-winit-030` feature ties GymDiff to the winit version of each Slint upgrade.
- macOS gets the menu bar icon too, so closing and reopening can be tested with `just dev`; start with Windows and uninstall stay Windows-only and hidden elsewhere.
- The uninstall hook now does three things; it must stay fast (Velopack gives it 60 s) and never fail the uninstall.
- Windows validation is needed for focus stealing (a second launch bringing the window forward), Task Manager's startup toggle and Velopack's uninstall messages.
