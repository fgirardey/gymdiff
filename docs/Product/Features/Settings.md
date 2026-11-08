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
  - **Send crash reports**: scrubbed errors with the app version and OS. No names, Riot IDs or game data.
  - **Send all logs**: also everyday logs, scrubbed the same way.
- Note under the switches: reports are deleted after 30 days.
- Flipping "Send crash reports" saves the consent (`diagnostics.json`, creating or forgetting the install id). Reporting starts or stops at the **next launch**.

- **Debug events** (throwaway, debug builds only, no design): a link at the bottom of Settings opens a screen with buttons that log a warning, log an error, log an error with personal data (to check scrubbing) or panic in a thread, to test the error reports end to end. Delete it once the integration is verified (`ui/screens/debug.slint`, `src/debug_events.rs`).

## Data
Preferences live in `settings.json` ([[ADR-0007 Local persistence]]). The consent is separate on purpose: `diagnostics.json` in the data folder (`ConsentStore` in `gymdiff-diagnostics`).

## Design
`App shell / Settings / Default` and `App shell / Settings / Crash reports on` in `design/gymdiff.pen`; components `Switch / Off`, `Switch / On`.

## Open questions
- **Send all logs** is in the design but not backed by anything: ADR-0006 decided errors only and listed full logs as an alternative to revisit with a new ADR. The switch is shown but not saved yet.
- Where are the install id and the report preview (ADR-0006, rule 3) shown? Not in the design yet.
- Say that the choice applies at the next launch, or restart reporting on the fly?
