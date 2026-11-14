---
status: in progress
moment: any time
design: "native tray menu (no frame); sketched: App shell / Settings / Scrolled to the bottom, Installer / Window / Options"
---

# Running in the background

## User story
As a player, I want cooldown.lol to keep running when I close its window, and to start with Windows if I choose, so that it is there when a game ends without me thinking about it.

## Why
cooldown.lol reacts to games ([[Post-game workout]]): closed by mistake, it misses the end of a game. Keeping it in the notification area, like other gaming companions, makes it always ready; starting with Windows removes the last step. Decision: [[ADR-0013 Notification area, startup and uninstall from the app]].

## Behaviour
**Notification area icon.** While cooldown.lol runs, its crest is in the notification area (next to the clock; on macOS, in the menu bar), tooltip **cooldown.lol**.
- Click (left) on the icon: shows the cooldown.lol window and brings it to the front. On macOS the click opens the menu, as menu bar icons do.
- Right click: a menu with **Open cooldown.lol** and **Quit cooldown.lol**.
- **Quit cooldown.lol** is the only way to stop cooldown.lol (besides uninstalling or signing out).

**Closing the window.** The window's close button (×) hides the window; cooldown.lol keeps running in the notification area. There is no setting for this. Settings says it under the Startup section: "Closing the window keeps cooldown.lol running in the notification area. Quit from its icon."

**One cooldown.lol at a time.** Starting cooldown.lol again (Start menu, Desktop shortcut, the installer's **Launch cooldown.lol**) while it already runs shows the running cooldown.lol's window instead of starting a second one. Two Windows accounts on the same PC each have their own.

**Start with Windows.** Settings → **Startup** → **Start with Windows** (installed builds only, off by default), and the same choice in the installer ([[Installation]]):
- on: when the player signs in to Windows, cooldown.lol starts **without its window**, only the notification area icon;
- the switch shows whether Windows will really start cooldown.lol: turning it off in Task Manager's *Startup apps* shows it as off;
- turning it on again re-enables it in Task Manager too;
- if the welcome screen has never been seen ([[First launch]]), cooldown.lol shows its window anyway.

**Not installed.** The portable zip, a pull request's `cooldown.exe` and local builds also hide in the notification area and stay single, but offer no Startup section (Windows would start a file that may move).

## Data
None from League. Start with Windows is the value `cooldown.lol` in `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` (per user, no administrator rights): `"%LOCALAPPDATA%\cooldown.lol\cooldown.exe" --background`. Task Manager's choice is in `…\Explorer\StartupApproved\Run`. Nothing in `settings.json`: the registry is the truth.

## Design
- Icon and menu: native (Windows notification area, macOS menu bar), no Pencil frame. Icon: the crest (`ui/images/app-icon.png`).
- Settings: **Startup** section (row **Start with Windows**, detail "Open cooldown.lol in the notification area when you sign in.") and the note about closing the window, under Shortcuts in `App shell / Settings / Scrolled to the bottom`.
- Installer: a third option **Start with Windows** ("Waits in the notification area when you sign in."), off by default, in `Installer / Window / Options`; the section is now named **Options**.

## Compliance
Nothing shown during a game; the icon is cooldown.lol's own. Starting with Windows does not touch League. See [[Riot policies]].

## Simulator scenario
None: `just dev` shows the icon in the macOS menu bar; close the window and reopen it from there.

## Windows validation
- [ ] The crest shows in the notification area; left click opens the window, right click shows Open cooldown.lol / Quit cooldown.lol.
- [ ] Closing the window keeps cooldown.lol running (Task Manager); Quit cooldown.lol stops it.
- [ ] Starting cooldown.lol from the Start menu while it runs (window hidden, or behind another window) brings the running window to the front, without a second process.
- [ ] Settings → Start with Windows on: after signing out and in, cooldown.lol runs with only its icon. Off: it does not start.
- [ ] Disabling cooldown.lol in Task Manager → Startup apps shows the switch off; turning it on re-enables it there.
- [ ] Installer with Start with Windows on: the entry exists after installing; reinstalling with it off removes it.
- [ ] Uninstalling removes the startup entry.

## Open questions
- Show a one-time notification the first time the window is closed ("cooldown.lol is still running")? Not for now.
