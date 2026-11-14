---
status: in progress
moment: any time
design: "sketched: App shell / Settings / Default, Updates checking, Updates ready, Updates failed"
---

# Auto-update

## User story
As the author testing cooldown.lol on my Windows PC, I want the alpha build to update itself to the latest build of `main`, so that I always test the current code without downloading anything by hand.

## Why
Development happens on macOS against the simulator ([[ADR-0003 macOS-first development with a simulator]]); the real target is Windows. Every merge should reach the gaming PC without a manual step, so real-client problems show up early. The mechanism is the base for a stable channel for friends later ([[Roadmap]]).

## Behaviour
**Channels.** The channel is baked into the build ([[ADR-0008 Versioning and releases]]) and into its installer ([[Installation]]); nothing to configure. Only builds **installed with the installer** update themselves ([[ADR-0011 Windows installer and updates with Velopack]]).

| Build | Channel | Updates itself |
|---|---|---|
| Alpha installer (`cooldown.lol-alpha-Setup.exe`, builds of `main`) | `alpha` | yes, to the latest alpha |
| Stable installer (`cooldown.lol-stable-Setup.exe`, tagged releases) | `stable` | not yet (switched on once updates are signed) |
| Pull request built on demand (`build` label), local build | — | no |

**When it checks** (installed alpha builds only):
- a few seconds after launch,
- then every hour while cooldown.lol is open,
- and when the player clicks **Check now** in Settings.

**What a check does:**
1. Velopack reads the channel's feed (`releases.alpha.json`) on cooldown.lol's GitHub releases and finds the newest package; for alpha, `<version>-alpha.<build>`, so the highest build wins.
2. Newer than the installed package: downloads it in the background (a small delta when possible), checksums included. Same or older: up to date.
3. The running app keeps going unchanged; **the new build starts the next time cooldown.lol is opened** (Velopack applies it at startup, in a second). No prompt, never interrupts a game.
4. If an even newer build appears before the next launch, it is downloaded instead.

**States shown in Settings** (section **Updates**, installed alpha builds only; other builds do not show the section). The row reads **Alpha channel**, with the state below it and a **Check now** button on the right:

| State | Text | Check now |
|---|---|---|
| Not checked yet | `Checks for updates every hour.` | enabled |
| Checking | `Checking for updates…` | disabled |
| Up to date | `Up to date. Last checked at 14:05.` | enabled |
| Downloading | `Downloading build 58…` | disabled |
| Ready | `Build 58 is ready. It starts the next time you open cooldown.lol.` | enabled |
| Failed | `Couldn't update: <short reason>. Next try in an hour.` | enabled |

The build number comes from the package version (`0.1.0-alpha.58` → build 58); a stable package reads `Version 0.2.0` instead of `Build 58`.

Short reasons: `no connection` (network error, GitHub down or limiting requests: 60 an hour without a token), `no alpha build published yet` (no alpha feed on the releases), `update error` (anything else, such as a corrupted download; details in the log).

**Edge cases:**
- Offline or GitHub down: Failed with `no connection`; the app works as usual.
- A check already running: Check now is disabled; the hourly timer skips.
- Logs: each check result at `INFO` when the state changes, failures at `WARN` (never `ERROR`: a network failure is not a bug, see [[Logging]]).

**Installing the alpha the first time:** run `cooldown.lol-alpha-Setup.exe` from the newest `v…-alpha.…` pre-release ([[Installation]]). Updates are downloaded by Velopack and carry no Mark of the Web, so SmartScreen stays quiet.

## Data
No League data. Per check: the release list (GitHub API, 60 requests an hour without a token) and the channel feeds, then the package when newer.

## Design
Section **Updates** under the error reports in `App shell / Settings / Default` (up to date) and `App shell / Settings / Crash reports on`; states in `App shell / Settings / Updates checking` (button at 40 % opacity, disabled), `Updates ready` and `Updates failed`. **Check now** is a compact secondary button (36 px high, `$border` outline, `$radius-m`); hover and pressed follow the secondary button in `Foundations`.

## Compliance
Nothing shown during a game; no interaction with the game process. Updating our own executable is outside [[Riot policies]].

## Simulator scenario
None: updates do not depend on the League client. Needs an installed build, so it is tested on the Windows PC after a merge. Code: `crates/cooldown-app/src/updater/`.

## Windows validation
- [ ] Installed with `cooldown.lol-alpha-Setup.exe` ([[Installation]]), Settings shows the Updates section.
- [ ] A merge to `main` → within an hour (or Check now) Settings says the new build is ready; the next launch runs it (About shows the new build).
- [ ] Offline → `no connection`.
- [ ] A pull request's `cooldown.exe` has no Updates section.

## Open questions
- Stable channel for friends: same mechanism on tagged releases, when to switch it on?
