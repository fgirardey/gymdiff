# Logging

How to pick a log level in GymDiff. Logs use `tracing`. Where each level ends up is described in [[Overview#Logging]] and [[Privacy]].

## The rule: log state changes, not ticks
The app runs loops (the watcher polls the League client every second). A line logged on every iteration hides everything else.

- A log inside a loop is `DEBUG` or `TRACE`, **unless** it marks a state change: connected, lost, phase changed.
- A condition that repeats is logged **once**, when it starts. Later repeats go to `DEBUG`.
- A normal situation is never a warning: no League client, a client that is starting or closing, a Live Client Data API that answers 404 while the game loads, a stale lockfile.

## Levels
| Level | Meaning | Examples |
|---|---|---|
| `ERROR` | A bug the author must fix. The app cannot recover from it alone. Each one becomes a Sentry issue. | cannot build the HTTP client, panic |
| `WARN` | Unexpected but handled, rare, and someone can act on it. | an LCU payload no longer matches our models, `settings.json` unreadable |
| `INFO` | Lifecycle milestones. A few lines per game, not per second. | app started, connected to the League client, phase changed, client lost |
| `DEBUG` | Details to diagnose a problem from the local log file. | lockfile found, a single failed request |
| `TRACE` | One line per poll or tick. Off by default. | client not found yet, state pushed to the UI |

## Writing a log line
- Structured fields, not formatted strings: `tracing::warn!(%error, consecutive_errors, "LCU request failed")`. The message stays constant so lines group well; the details go in fields.
- Short messages, lowercase, no trailing period.
- Never log LCU or Live Client payloads, summoner names, Riot IDs, `puuid` or account ids, at any level (see [[Privacy#Rules for contributors]]). Scrubbing is a safety net, not a licence.
