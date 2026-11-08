---
status: proposed
date: 2026-11-08
---

# ADR-0006 Opt-in error reports

## Context
GymDiff runs on friends' Windows PCs next to the game. Today logs stay on the PC (`%LOCALAPPDATA%\GymDiff\logs`, fetched with `just win-logs` over SSH), which only works for the author's own machine. To fix problems seen elsewhere we want error reports to reach the author, without surveilling players.

Constraints: the app is not registered with Riot yet and will never be monetized ([[Vision#Non-goals]]); the local APIs carry personal data (Riot IDs, `puuid`, match data) that must never leave the PC ([[Riot policies]]); the app must keep working offline and never be slowed or crashed by reporting.

## Decision
Add **opt-in** central reporting to Sentry, with two independent switches. Decided with the author on 2026-11-08, revised the same day to use Sentry Logs:

| Question | Decision |
|---|---|
| Consent | **Opt-in, off by default**, two switches in Settings: **Send crash reports** and **Send all logs**. Changeable at any time. |
| What is sent | **Send crash reports** → Sentry **Issues**: `ERROR` events and panics. **Send all logs** → Sentry **Logs**: structured `INFO`, `WARN` and `ERROR` events. Both carry the app version, OS name and install id. Never `DEBUG`/`TRACE`, never events from dependencies. Nothing else: no traces, breadcrumbs, sessions or stack traces. |
| Backend | **Sentry** (EU region or self-hosted; `send_default_pii = false`, no user, no server name, IP not stored). |
| Retention | **30 days**, then purged. |
| Deletion | By the random **install id** shown in Settings, on request. |

Design rules:
1. **Scrub before sending.** `before_send` (issues) and `before_send_log` (logs) run `gymdiff-diagnostics::scrub` on every message and every field (paths and user names, Riot IDs, UUIDs/`puuid`, IPv4, e-mails, tokens and passwords). Game payloads are never logged on purpose; scrubbing is the safety net.
2. **Nothing about the machine or the user.** SDK default integrations are off (no OS or device context); `server.address` and `user.*` are dropped; issues keep only their message, exception values, level, logger, release, environment, tags, the `tracing` fields and the trace context (which links an issue to the logs of the same session).
3. **Low noise by policy.** Log levels follow [[Logging]]: `ERROR` only for bugs, state changes rather than loop iterations, so a normal session sends a few logs per game and no issue. At most 10 issues per launch, so a bug in a loop cannot burn the quota.
4. **Consent is checked on doubt, off.** Missing or corrupt consent file means off. The install id exists while at least one switch is on and is forgotten when both are off.
5. **Sending is best-effort and isolated.** The SDK queues and sends in the background, never blocks the UI or the watcher, never touches the game process. Local file logs are unchanged and work offline.

## Alternatives considered
- **Opt-out** — weaker consent, harder to defend, not needed for a friend group.
- **Manual "send diagnostics" only** — most private, but misses crashes nobody reports. May stay as the export path.
- **Own HTTPS endpoint** — full control, but we would run and secure a server.
- **Prefilled GitHub issue** — zero backend, but public and fully manual.
- **Errors only, with the last warnings attached to each report** (the first version of this ADR) — dropped the same day: one blob per error, mostly noise, and no view of what happened without an error. Sentry Logs give that view, behind their own switch.
- **Custom report schema with a preview** — the SDK-native events are scrubbed the same way; the preview was never shown in the UI.

## Consequences
- Crate `gymdiff-diagnostics` (pure, no network): scrubber and consent store (`diagnostics.json`: `crash_reports`, `logs`, `install_id`).
- `gymdiff-app/src/reporting.rs` is the only code that talks to Sentry (SDK `sentry` 0.49 with `tracing` and `logs`). It starts only with consent **and** a DSN. `LogLayer` (`INFO`+) feeds Sentry Logs and `ErrorLayer` (`ERROR`) feeds Sentry Issues, both limited to GymDiff's own crates; each is gated by its switch in `before_send` / `before_send_log`. Panics go through `tracing` and become issues.
- Environment `development` for debug builds, `production` for release builds.
- The DSN is not committed: `GYMDIFF_SENTRY_DSN` at run time, or baked in at build time from the CI repository variable of the same name. It is a write-only key, not a secret, but keeping it out of the source avoids noise from forks and clones.
- Sentry project in the **EU** region (`ingest.de.sentry.io`), created 2026-11-08. An earlier project in the US region was dropped before any event was sent.
- The Settings screen ([[Settings]]) sets the consent; a change applies at the next launch.
- A new third-party processor (Sentry) must be named in [[Privacy]] and in the product's Riot registration.
- Status stays `proposed` until the author accepts it after the Sentry project is chosen.
