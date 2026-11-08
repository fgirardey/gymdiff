---
status: implemented
moment: any time
design: "sketched: App shell / Settings / Default, Updates checking, Updates ready, Updates failed"
---

# Auto-update

## User story
As the author testing GymDiff on my Windows PC, I want the alpha build to update itself to the latest build of `main`, so that I always test the current code without downloading anything by hand.

## Why
Development happens on macOS against the simulator ([[ADR-0003 macOS-first development with a simulator]]); the real target is Windows. Every merge should reach the gaming PC without a manual step, so real-client problems show up early. The mechanism is the base for a stable channel for friends later ([[Roadmap]]).

## Behaviour
**Channels.** The channel is baked into the build ([[ADR-0008 Versioning and releases]]); nothing to configure.

| Build | Channel | Updates itself |
|---|---|---|
| CI build of `main` (the alpha) | `alpha` | yes, to the latest alpha |
| Pull request built on demand (`build` label) | `preview` | no |
| Tagged release | `release` | not yet (stable channel later) |
| Local build | `development` | no |

**When it checks** (alpha builds only):
- a few seconds after launch,
- then every hour while GymDiff is open,
- and when the player clicks **Check now** in Settings.

**What a check does:**
1. Lists GymDiff's GitHub releases and takes the latest of the channel: for alpha, the `v<version>-alpha.<build>` pre-release with the highest build; then reads its `manifest.json` (see [[ADR-0010 Alpha channel and self-update]]).
2. Newer means a **higher build number** than the running build. Same or lower: up to date.
3. Downloads the new executable in the background, checks its size and SHA-256 against the manifest.
4. Replaces the executable on disk. The running app keeps going unchanged; **the new build starts the next time GymDiff is opened**. No restart, no prompt, never interrupts a game.
5. If an even newer build appears before the next launch, it replaces the waiting one.

**States shown in Settings** (section **Updates**, alpha builds only; other builds do not show the section). The row reads **Alpha channel**, with the state below it and a **Check now** button on the right:

| State | Text | Check now |
|---|---|---|
| Not checked yet | `Checks for updates every hour.` | enabled |
| Checking | `Checking for updates…` | disabled |
| Up to date | `Up to date. Last checked at 14:05.` | enabled |
| Downloading | `Downloading build 58…` | disabled |
| Ready | `Build 58 is ready. It starts the next time you open GymDiff.` | enabled |
| Failed | `Couldn't update: <short reason>. Next try in an hour.` | enabled |

Short reasons: `no connection`, `GitHub is limiting requests` (over 60 requests an hour from the same address), `no alpha build published yet` (no alpha release with a manifest), `download incomplete` (size or checksum mismatch), `can't write next to GymDiff` (the executable sits in a folder the player can't write to, e.g. Program Files).

**Edge cases:**
- Offline or GitHub down: Failed with `no connection`; the app works as usual.
- A check already running: Check now is disabled; the hourly timer skips.
- The manifest is for another target than the running build: ignored (up to date).
- Logs: each check result at `INFO` when the state changes, failures at `WARN` (never `ERROR`: a network failure is not a bug, see [[Logging]]).

**Installing the alpha the first time:** download `gymdiff-<version>-alpha.<build>-windows-x64.exe` from the newest `v…-alpha.…` pre-release on GitHub into a folder you own (e.g. `%LOCALAPPDATA%\GymDiff\`), unblock it once (Properties → Unblock, or SmartScreen → Run anyway). Later updates are downloaded by the app and carry no Mark of the Web, so SmartScreen stays quiet.

## Data
No League data. Per check: the release list (GitHub API, 60 requests an hour without a token) and the latest release's `manifest.json`, then the executable when newer.

## Design
Section **Updates** under the error reports in `App shell / Settings / Default` (up to date) and `App shell / Settings / Crash reports on`; states in `App shell / Settings / Updates checking` (button at 40 % opacity, disabled), `Updates ready` and `Updates failed`. **Check now** is a compact secondary button (36 px high, `$border` outline, `$radius-m`); hover and pressed follow the secondary button in `Foundations`.

## Compliance
Nothing shown during a game; no interaction with the game process. Updating our own executable is outside [[Riot policies]].

## Simulator scenario
None: updates do not depend on the League client. Tested with a local release list (`GYMDIFF_UPDATE_URL`, see [[Releasing]]) on macOS. Code: `crates/gymdiff-app/src/updater/`.

## Windows validation
- [ ] First install from the newest alpha pre-release, after unblocking.
- [ ] A merge to `main` → within an hour (or Check now) Settings says the new build is ready; the next launch runs it (About shows the new build).
- [ ] Executable in a read-only folder → `can't write next to GymDiff`, app keeps working.
- [ ] Offline → `no connection`.

## Open questions
- Stable channel for friends: same mechanism on tagged releases, when to switch it on?
