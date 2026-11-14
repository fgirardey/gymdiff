---
status: in progress
moment: before first launch
design: "sketched: Installer / Window / Options, Installing, Done, Failed; App shell / Settings / Scrolled to the bottom, Uninstall confirm, Uninstall failed"
---

# Installation

## User story
As a player on Windows, I want to install GymDiff like any other app, find it in the Start menu and uninstall it from GymDiff or from Windows settings, so that I never handle loose `.exe` files.

## Why
Friends test GymDiff on their own PCs ([[Vision]]); copying an executable into a folder and unblocking it is a barrier, and leaves no clean way to remove the app. Decision: [[ADR-0011 Windows installer and updates with Velopack]].

## Behaviour
**Getting the installer.** Each GitHub release carries `GymDiff-<channel>-Setup.exe`:
- `GymDiff-alpha-Setup.exe` on the alpha pre-releases `v<version>-alpha.<build>` (the latest build of `main`, for the author),
- `GymDiff-stable-Setup.exe` on the stable releases `v<version>`.

**Installing.** Double-click the installer. No administrator rights are asked: GymDiff installs for the current user in `%LOCALAPPDATA%\GymDiff`. Not signed yet: Windows SmartScreen warns once ("Windows protected your PC" → **More info** → **Run anyway**). The installer is a GymDiff window (crest, GymDiff colours), in three steps:

1. **Options**: a short welcome and three switches under **Options**:
   - **Add to the Start menu** (on by default: without it Windows Search does not find GymDiff),
   - **Add a Desktop shortcut** (off by default),
   - **Start with Windows** (off by default; "Waits in the notification area when you sign in.", see [[Running in the background]]),

   then **Install** (and **Cancel**, which closes the installer without installing).
2. **Installing**: progress while GymDiff is installed. Nothing to click.
3. **Done**: "GymDiff is installed" (the chosen shortcuts and startup entry exist by now, the others are removed), a switch **Launch GymDiff** (on by default) and **Finish**, which launches GymDiff if asked and closes the installer.

If installing fails: **Failed** screen with a short reason ("the installation could not start", "the installation was interrupted", or for a local build "this installer was built without GymDiff inside") and **Close**. A shortcut or startup entry that cannot be created does not fail the install: Settings can add it later.

- The shortcuts are named **GymDiff**, show the crest, and open the installed GymDiff (they keep working across updates).
- One GymDiff per PC: the installer you run decides the channel ([[Auto-update]]). Running the other channel's installer replaces the installed GymDiff.
- Already installed: the installer reinstalls that version and applies the new choices. If GymDiff is running, Velopack stops it first.

**Changing the shortcuts later.** Settings → **Shortcuts** (installed builds only): **Start menu** and **Desktop** switches add or remove each shortcut at once. They reflect whether the shortcut exists, so a shortcut deleted by hand shows as off. **Start with Windows** is under Settings → **Startup** ([[Running in the background]]).

**Updating.** Installed alpha builds update themselves ([[Auto-update]]). Re-running an installer reinstalls that version.

**Uninstalling.** Two ways, same result:
- from GymDiff: Settings → **Uninstall** (installed builds only) → **Uninstall**. A dialog asks to confirm: "GymDiff closes, then its app, shortcuts and logs are removed from this PC.", a switch **Also delete my settings** (off by default; "Preferences, error-report choice and install id. Keep them to pick up where you left off if you reinstall."), **Cancel** and **Uninstall**. Uninstall closes GymDiff and starts Velopack's uninstaller, which shows its own progress and "uninstalled" messages. If the uninstaller cannot start, the dialog stays open with "The uninstaller could not start. Uninstall from Windows Settings → Apps → GymDiff instead." and nothing is deleted;
- from Windows: *Settings → Apps → GymDiff → Uninstall*.

Either way it removes the app, its Start menu and Desktop shortcuts, its startup entry and `%LOCALAPPDATA%\GymDiff`, including the logs. Settings, the error-report consent, the install id and the game history in `%APPDATA%\GymDiff` are kept, so a reinstall starts where the player left off (no welcome screen again), unless **Also delete my settings** was on: then `%APPDATA%\GymDiff` goes too. Windows' own uninstall always keeps them.

**Not installed.** The portable zip, a pull request's `gymdiff.exe` and local builds run as before but do not update themselves and offer no shortcuts, startup entry or uninstall (no Updates, Shortcuts, Startup or Uninstall section in Settings).

## Data
None from League. Files: `%LOCALAPPDATA%\GymDiff` (app, Velopack's `Update.exe`, logs), `%APPDATA%\GymDiff` (settings, consent, `history.sqlite`; while an uninstall that deletes them is running, the marker `delete-on-uninstall`). Registry: the startup entry ([[Running in the background#Data]]).

## Design
`Installer / Window / Options`, `Installing`, `Done`, `Failed` in `design/gymdiff.pen`: a 420 × 460 window with the crest, a headline and subline like the welcome screen, switch rows like Settings, primary and secondary buttons. Installing shows an indeterminate progress bar (`$accent` on `$surface`). Settings gains a **Shortcuts** section (rows **Start menu** and **Desktop**) under Updates in every `App shell / Settings / *` frame; the screen now scrolls, see `App shell / Settings / Scrolled to the bottom`, which also shows the **Startup** and **Uninstall** sections (row "Remove GymDiff from this PC", "The app, its shortcuts and its logs. You choose whether to keep your settings.", an outlined **Uninstall** button in `$danger`). The confirmation is a centred dialog over a scrim, like the workout prompt: `App shell / Settings / Uninstall confirm` and `App shell / Settings / Uninstall failed`; its **Uninstall** button is filled with `$danger`. The installer window is now 420 × 500.

## Compliance
Nothing shown during a game; installing our own app is outside [[Riot policies]].

## Simulator scenario
None.

## Windows validation
- [ ] `GymDiff-alpha-Setup.exe` installs without admin rights, after SmartScreen's "Run anyway"; only the chosen shortcuts exist, with the crest; GymDiff starts only if "Launch GymDiff" was on.
- [ ] Settings → Shortcuts adds and removes each shortcut; a shortcut deleted by hand shows as off.
- [ ] GymDiff appears in *Settings → Apps* and uninstalls from there; `%LOCALAPPDATA%\GymDiff` is gone, `%APPDATA%\GymDiff` is kept.
- [ ] Settings → Uninstall → Uninstall (delete off): GymDiff closes, Velopack shows its uninstall messages, the app, shortcuts and startup entry are gone, `%APPDATA%\GymDiff` is kept.
- [ ] Same with **Also delete my settings** on: `%APPDATA%\GymDiff` is gone too; reinstalling shows the welcome screen.
- [ ] Reinstalling keeps the settings (no welcome screen).

## Open questions
- None.
