---
status: specified
moment: before first launch
design: "none: Velopack's installer and Windows' own Apps settings"
---

# Installation

## User story
As a player on Windows, I want to install GymDiff like any other app, find it in the Start menu and uninstall it from Windows settings, so that I never handle loose `.exe` files.

## Why
Friends test GymDiff on their own PCs ([[Vision]]); copying an executable into a folder and unblocking it is a barrier, and leaves no clean way to remove the app. Decision: [[ADR-0011 Windows installer and updates with Velopack]].

## Behaviour
**Getting the installer.** Each GitHub release carries `GymDiff-<channel>-Setup.exe`:
- `GymDiff-alpha-Setup.exe` on the alpha pre-releases `v<version>-alpha.<build>` (the latest build of `main`, for the author),
- `GymDiff-stable-Setup.exe` on the stable releases `v<version>`.

**Installing.**
- Double-click the installer. No administrator rights are asked: GymDiff installs for the current user in `%LOCALAPPDATA%\GymDiff`.
- Not signed yet: Windows SmartScreen warns once ("Windows protected your PC" → **More info** → **Run anyway**).
- Velopack shows a short progress window, creates the **Start menu** and **Desktop** shortcuts (crest icon), registers GymDiff in *Settings → Apps*, and starts GymDiff.
- One GymDiff per PC: the installer you run decides the channel ([[Auto-update]]). Running the other channel's installer replaces the installed GymDiff.

**Updating.** Installed alpha builds update themselves ([[Auto-update]]). Re-running an installer reinstalls that version.

**Uninstalling.** *Settings → Apps → GymDiff → Uninstall* removes the app, its shortcuts and `%LOCALAPPDATA%\GymDiff`, including the logs. Settings, the error-report consent and the install id in `%APPDATA%\GymDiff` are kept, so a reinstall starts where the player left off (no welcome screen again).

**Not installed.** The portable zip, a pull request's `gymdiff.exe` and local builds run as before but do not update themselves (no Updates section in Settings).

## Data
None from League. Files: `%LOCALAPPDATA%\GymDiff` (app, Velopack's `Update.exe`, logs), `%APPDATA%\GymDiff` (settings, consent).

## Design
No GymDiff screen: Velopack's installer window (GymDiff title and icon) and Windows' Apps settings.

## Compliance
Nothing shown during a game; installing our own app is outside [[Riot policies]].

## Simulator scenario
None.

## Windows validation
- [ ] `GymDiff-alpha-Setup.exe` installs without admin rights, after SmartScreen's "Run anyway"; Start menu and Desktop shortcuts with the crest; GymDiff starts.
- [ ] GymDiff appears in *Settings → Apps* and uninstalls from there; `%LOCALAPPDATA%\GymDiff` is gone, `%APPDATA%\GymDiff` is kept.
- [ ] Reinstalling keeps the settings (no welcome screen).

## Open questions
- Start GymDiff with Windows? Not for now.
