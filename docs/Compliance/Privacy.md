# Privacy

What cooldown.lol keeps, what leaves the PC and how to stop it. Decision: [[ADR-0006 Opt-in error reports]]. Working summary, not legal advice.

## By default
Nothing leaves the PC except requests to the League client on `127.0.0.1` and to Data Dragon for champion images ([[ADR-0004 Champion images from Data Dragon]]). Logs are local files only.

## Reports (opt-in)
Two switches in Settings, both off until the player turns them on. Changes apply at the next launch. Turning both off stops sending and forgets the install id.

- **Send crash reports**: errors and panics with their stack trace, and native crashes with a memory snapshot (minidump), as Sentry issues.
- **Send all logs**: everyday logs (`INFO`, `WARN`, `ERROR`), as Sentry logs.

| Sent | Never sent |
|---|---|
| App version, build number, commit, channel (release, preview, development) | Summoner names, Riot IDs, `puuid`, account ids |
| OS name and version, compiler version | Match, champ select or Live Client data |
| Random install id (to answer deletion requests) | `DEBUG`/`TRACE` logs, file paths with user names |
| Scrubbed messages, their fields and module (cooldown.lol's own code only) | Machine name, IP addresses (not stored by the backend), e-mails, tokens |
| Stack traces: functions, file names, lines; loaded modules (file names only) | Local variables |
| On a native crash: a minidump (registers and stack memory of the app's threads) | |

- **Minidumps** are the one thing the app cannot scrub: stack memory may hold a Riot ID or the LCU token if cooldown.lol crashes during a game. The Sentry project does not store them (*Store Native Crash Reports* off): Sentry turns the dump into a stack trace and discards it. The switch's text says a hard crash sends a memory snapshot.
- **Source code and symbols** of the app (not player data) are uploaded to the same Sentry project by CI, to show where an error happened ([[ADR-0008 Versioning and releases]]).
- **Retention**: 30 days, then deleted.
- **Deletion**: send the install id from Settings to the author; matching reports are deleted.
- **Local data**: uninstalling from Settings with **Also delete my settings** removes the settings, the consent and the install id from the PC ([[Installation]]); the logs always go with the app.
- **Processor**: Sentry, **EU region** (Frankfurt). Named here and in the Riot registration.
- **Developers** can also opt in by hand with `{"crash_reports": true, "logs": true}` in `diagnostics.json` in the data folder (see [[Overview#Configuration (environment variables)]]) and `COOLDOWN_SENTRY_DSN`.
- **Offline / failures**: local logs keep working; a failed send is dropped silently and never affects the app or the game.

## Rules for contributors
- Never log LCU or Live Client payloads, names or ids, even at `debug`.
- Anything that changes what is sent updates this note, the ADR and the tests in `reporting.rs`.
