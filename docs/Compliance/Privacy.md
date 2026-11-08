# Privacy

What GymDiff keeps, what leaves the PC and how to stop it. Decision: [[ADR-0006 Opt-in error reports]]. Working summary, not legal advice.

## By default
Nothing leaves the PC except requests to the League client on `127.0.0.1` and to Data Dragon for champion images ([[ADR-0004 Champion images from Data Dragon]]). Logs are local files only.

## Error reports (opt-in)
Off until the player turns them on (welcome screen or Settings); turning them off stops sending and forgets the install id.

| Sent | Never sent |
|---|---|
| App version, OS name | Summoner names, Riot IDs, `puuid`, account ids |
| Random install id (to answer deletion requests) | Match, champ select or Live Client data |
| Scrubbed `WARN`/`ERROR` messages and their module | `INFO`/`DEBUG` logs, file paths with user names |
| Panic message (scrubbed) | IP addresses (not stored by the backend), e-mails, tokens |

- **Preview**: the exact report can be shown and exported before sending.
- **Retention**: 30 days, then deleted.
- **Deletion**: send the install id from Settings to the author; matching reports are deleted.
- **Processor**: Sentry. The project is currently in Sentry's **US** region; moving it to the EU region is an open decision. Named here and in the Riot registration.
- **Today**: there is no UI to opt in yet, so nothing is sent. Developers can opt in with `{"enabled": true}` in `diagnostics.json` in the data folder (see [[Overview#Configuration (environment variables)]]) and `GYMDIFF_SENTRY_DSN`.
- **Offline / failures**: local logs keep working; a failed send is dropped silently and never affects the app or the game.

## Rules for contributors
- Never log LCU or Live Client payloads, names or ids, even at `debug`.
- Anything that changes the report fields updates this note, the ADR and the schema test.
