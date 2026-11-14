---
status: in progress
moment: before first launch
design: "sketched: Installer / Window / Options, Installing, Done, Failed; App shell / Settings / Scrolled to the bottom, Uninstall confirm, Uninstall failed"
---

# Installation

## User story
As a player on Windows, I want to install cooldown.lol like any other app, find it in the Start menu and uninstall it from cooldown.lol or from Windows settings, so that I never handle loose `.exe` files.

## Why
Friends test cooldown.lol on their own PCs ([[Vision]]); copying an executable into a folder and unblocking it is a barrier, and leaves no clean way to remove the app. Decision: [[ADR-0011 Windows installer and updates with Velopack]].

## Behaviour
**Getting the installer.** Each GitHub release carries `cooldown.lol-<channel>-Setup.exe`:
- `cooldown.lol-alpha-Setup.exe` on the alpha pre-releases `v<version>-alpha.<build>` (the latest build of `main`, for the author),
- `cooldown.lol-stable-Setup.exe` on the stable releases `v<version>`.

**Installing.** Double-click the installer. No administrator rights are asked: cooldown.lol installs for the current user in `%LOCALAPPDATA%\cooldown.lol`. Not signed yet: Windows SmartScreen warns once ("Windows protected your PC" → **More info** → **Run anyway**). The installer is a cooldown.lol window (crest, cooldown.lol colours), in three steps:

1. **Options**: a short welcome and three switches under **Options**:
   - **Add to the Start menu** (on by default: without it Windows Search does not find cooldown.lol),
   - **Add a Desktop shortcut** (off by default),
   - **Start with Windows** (off by default; "Waits in the notification area when you sign in.", see [[Running in the background]]),

   then **Install** (and **Cancel**, which closes the installer without installing).
2. **Installing**: progress while cooldown.lol is installed. Nothing to click.
3. **Done**: "cooldown.lol is installed" (the chosen shortcuts and startup entry exist by now, the others are removed), a switch **Launch cooldown.lol** (on by default) and **Finish**, which launches cooldown.lol if asked and closes the installer.

If installing fails: **Failed** screen with a short reason ("the installation could not start", "the installation was interrupted", or for a local build "this installer was built without cooldown.lol inside") and **Close**. A shortcut or startup entry that cannot be created does not fail the install: Settings can add it later.

- The shortcuts are named **cooldown.lol**, show the crest, and open the installed cooldown.lol (they keep working across updates).
- One cooldown.lol per PC: the installer you run decides the channel ([[Auto-update]]). Running the other channel's installer replaces the installed cooldown.lol.
- Already installed: the installer reinstalls that version and applies the new choices. If cooldown.lol is running, Velopack stops it first.

**Changing the shortcuts later.** Settings → **Shortcuts** (installed builds only): **Start menu** and **Desktop** switches add or remove each shortcut at once. They reflect whether the shortcut exists, so a shortcut deleted by hand shows as off. **Start with Windows** is under Settings → **Startup** ([[Running in the background]]).

**Updating.** Installed alpha builds update themselves ([[Auto-update]]). Re-running an installer reinstalls that version.

**Uninstalling.** Two ways, same result:
- from cooldown.lol: Settings → **Uninstall** (installed builds only) → **Uninstall**. A dialog asks to confirm: "cooldown.lol closes, then its app, shortcuts and logs are removed from this PC.", a switch **Also delete my settings** (off by default; "Preferences, error-report choice and install id. Keep them to pick up where you left off if you reinstall."), **Cancel** and **Uninstall**. Uninstall closes cooldown.lol and starts Velopack's uninstaller, which shows its own progress and "uninstalled" messages. If the uninstaller cannot start, the dialog stays open with "The uninstaller could not start. Uninstall from Windows Settings → Apps → cooldown.lol instead." and nothing is deleted;
- from Windows: *Settings → Apps → cooldown.lol → Uninstall*.

Either way it removes the app, its Start menu and Desktop shortcuts, its startup entry and `%LOCALAPPDATA%\cooldown.lol`, including the logs. Settings, the error-report consent, the install id and the game history in `%APPDATA%\cooldown.lol` are kept, so a reinstall starts where the player left off (no welcome screen again), unless **Also delete my settings** was on: then `%APPDATA%\cooldown.lol` goes too. Windows' own uninstall always keeps them.

**Not installed.** The portable zip, a pull request's `cooldown.exe` and local builds run as before but do not update themselves and offer no shortcuts, startup entry or uninstall (no Updates, Shortcuts, Startup or Uninstall section in Settings).

## Data
None from League. Files: `%LOCALAPPDATA%\cooldown.lol` (app, Velopack's `Update.exe`, logs), `%APPDATA%\cooldown.lol` (settings, consent, `history.sqlite`; while an uninstall that deletes them is running, the marker `delete-on-uninstall`). Registry: the startup entry ([[Running in the background#Data]]).

## Design
`Installer / Window / Options`, `Installing`, `Done`, `Failed` in `design/cooldown.pen`: a 420 × 460 window with the crest, a headline and subline like the welcome screen, switch rows like Settings, primary and secondary buttons. Installing shows an indeterminate progress bar (`$accent` on `$surface`). Settings gains a **Shortcuts** section (rows **Start menu** and **Desktop**) under Updates in every `App shell / Settings / *` frame; the screen now scrolls, see `App shell / Settings / Scrolled to the bottom`, which also shows the **Startup** and **Uninstall** sections (row "Remove cooldown.lol from this PC", "The app, its shortcuts and its logs. You choose whether to keep your settings.", an outlined **Uninstall** button in `$danger`). The confirmation is a centred dialog over a scrim, like the workout prompt: `App shell / Settings / Uninstall confirm` and `App shell / Settings / Uninstall failed`; its **Uninstall** button is filled with `$danger`. The installer window is now 420 × 500.

## Compliance
Nothing shown during a game; installing our own app is outside [[Riot policies]].

## Simulator scenario
None.

## Windows validation
- [ ] `cooldown.lol-alpha-Setup.exe` installs without admin rights, after SmartScreen's "Run anyway"; only the chosen shortcuts exist, with the crest; cooldown.lol starts only if "Launch cooldown.lol" was on.
- [ ] Settings → Shortcuts adds and removes each shortcut; a shortcut deleted by hand shows as off.
- [ ] cooldown.lol appears in *Settings → Apps* and uninstalls from there; `%LOCALAPPDATA%\cooldown.lol` is gone, `%APPDATA%\cooldown.lol` is kept.
- [ ] Settings → Uninstall → Uninstall (delete off): cooldown.lol closes, Velopack shows its uninstall messages, the app, shortcuts and startup entry are gone, `%APPDATA%\cooldown.lol` is kept.
- [ ] Same with **Also delete my settings** on: `%APPDATA%\cooldown.lol` is gone too; reinstalling shows the welcome screen.
- [ ] Reinstalling keeps the settings (no welcome screen).

## Open questions
- None.
