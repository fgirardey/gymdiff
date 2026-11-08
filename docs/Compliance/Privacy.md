# Privacy

What GymDiff keeps, what leaves the PC and how to stop it. Decision: [[ADR-0006 Opt-in error reports]]. Working summary, not legal advice.

## By default
Nothing leaves the PC except requests to the League client on `127.0.0.1` and to Data Dragon for champion images ([[ADR-0004 Champion images from Data Dragon]]). Logs are local files only.

## Reports (opt-in)
Two switches in Settings, both off until the player turns them on. Changes apply at the next launch. Turning both off stops sending and forgets the install id.

- **Send crash reports**: errors and panics, as Sentry issues.
- **Send all logs**: everyday logs (`INFO`, `WARN`, `ERROR`), as Sentry logs.

| Sent | Never sent |
|---|---|
| App version, OS name, debug or release build | Summoner names, Riot IDs, `puuid`, account ids |
| Random install id (to answer deletion requests) | Match, champ select or Live Client data |
| Scrubbed messages, their fields and module (GymDiff's own code only) | `DEBUG`/`TRACE` logs, file paths with user names |
| Panic message (scrubbed) | Machine name, IP addresses (not stored by the backend), e-mails, tokens |

- **Retention**: 30 days, then deleted.
- **Deletion**: send the install id from Settings to the author; matching reports are deleted.
- **Processor**: Sentry, **EU region** (Frankfurt). Named here and in the Riot registration.
- **Developers** can also opt in by hand with `{"crash_reports": true, "logs": true}` in `diagnostics.json` in the data folder (see [[Overview#Configuration (environment variables)]]) and `GYMDIFF_SENTRY_DSN`.
- **Offline / failures**: local logs keep working; a failed send is dropped silently and never affects the app or the game.

## Rules for contributors
- Never log LCU or Live Client payloads, names or ids, even at `debug`.
- Anything that changes what is sent updates this note, the ADR and the tests in `reporting.rs`.
