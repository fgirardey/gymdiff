---
status: implemented
moment: any time
design: "sketched: App shell / Settings / Default, App shell / Settings / Crash reports on"
---

# Settings

## User story
As a player, I want to decide whether GymDiff sends error reports, and change my mind at any time.

## Why
Error reports are opt-in ([[ADR-0006 Opt-in error reports]]): the choice must be visible and reversible.

## Behaviour
- Opens from the gear button in the top bar; "‹ History" goes back. See [[Screens]].
- Section **Error reports**, two switches, both **off** by default:
  - **Send crash reports**: scrubbed errors and panics with the app version and OS (Sentry issues). No names, Riot IDs or game data.
  - **Send all logs**: everyday logs (`INFO` and above), scrubbed the same way (Sentry logs). Independent of the first switch.
- Note under the switches: reports are deleted after 30 days.
- Flipping a switch saves the consent (`diagnostics.json`). The install id is created when a switch is turned on and forgotten when both are off. Reporting starts or stops at the **next launch**.

- **Debug events** (throwaway, debug builds only, no design): a link at the bottom of Settings opens a screen with buttons that log a warning, log an error, log an error with personal data (to check scrubbing) or panic in a thread, to test the error reports end to end. Delete it once the integration is verified (`ui/screens/debug.slint`, `src/debug_events.rs`).

## Data
Preferences live in `settings.json` ([[ADR-0007 Local persistence]]). The consent is separate on purpose: `diagnostics.json` in the data folder (`ConsentStore` in `gymdiff-diagnostics`).

## Design
`App shell / Settings / Default` and `App shell / Settings / Crash reports on` in `design/gymdiff.pen`; components `Switch / Off`, `Switch / On`.

## Open questions
- Where is the install id shown? Not in the design yet.
- Say that the choice applies at the next launch, or restart reporting on the fly?
