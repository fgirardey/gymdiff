# Windows testing

The Mac covers most of the work. The **Windows PC** validates what only it can: the real League client and game, overlays, Windows-specific code, performance next to the game.

## Ways to get a Windows build
| Option | When | How |
|---|---|---|
| **Alpha** | default: always the latest `main`, updates itself | once: download `gymdiff-alpha-windows-x64.exe` from the `alpha` pre-release on GitHub into `%LOCALAPPDATA%\GymDiff\` (not Program Files), `Unblock-File` it; then each merge is installed for the next launch ([[Auto-update]]) |
| **CI artifact** | a pull request, before merging | add the `build` label to the PR → GitHub Actions → *Artifacts* → `gymdiff-windows-<sha>` (app) or `gymdiff-recorder-windows-<sha>` ([[Recorder]]); run `Unblock-File` on the `.exe` first |
| **Build on the PC over SSH** | fast loop on Windows-specific code | `just win-build` from the Mac |
| **Build on the PC directly** | debugging on the PC | `cargo run -p gymdiff-app` in a terminal on the PC (debug builds keep the console with live logs) |

## One-time setup of the PC
1. Install Rust: <https://rustup.rs> (accept the Visual Studio C++ Build Tools prompt).
2. Install Git and clone the repository, e.g. in `C:\dev\gymdiff`.
3. Optional: `winget install Casey.Just`.
4. For `just win-build` / `just win-logs` from the Mac:
   - enable **OpenSSH Server** (*Settings → System → Optional features → OpenSSH Server*), start the `sshd` service and set it to automatic,
   - add your Mac's public key to the PC (for an administrator account: `C:\ProgramData\ssh\administrators_authorized_keys`),
   - the commands assume the default `cmd.exe` SSH shell,
   - on the Mac, export (e.g. in `~/.zshrc`):
     ```sh
     export GYMDIFF_WIN_HOST="florian@<pc-name>.local"
     export GYMDIFF_WIN_REPO='C:\dev\gymdiff'
     ```
   - the PC must be able to `git pull` the repository (remote reachable from both machines).

A GUI app started over SSH does not appear on the Windows desktop: `just win-build` only builds. Launch `target\release\gymdiff.exe` on the PC.

## Test session checklist
- [ ] App starts, finds the client (lockfile) and shows "Connected"
- [ ] Phases follow the client (lobby → champ select → game → end of game)
- [ ] In-game data is correct (compare with the game)
- [ ] App recovers when the client restarts
- [ ] No noticeable impact on game performance (Task Manager: CPU/RAM of `gymdiff.exe`)
- [ ] Feature-specific checks from the feature note's *Windows validation* section

## When something only breaks on Windows
1. Reproduce, then `just win-logs` on the Mac: the latest log lands in `target/windows.log`.
   (Central error reports are opt-in and do not replace this: see [[Privacy]].)
2. Ask Claude Code to analyse it together with the relevant code.
3. If the bug comes from unexpected API data, add the payload to a scenario so it can be reproduced on the Mac.
