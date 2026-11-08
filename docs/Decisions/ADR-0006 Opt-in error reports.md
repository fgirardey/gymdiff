---
status: proposed
date: 2026-11-08
---

# ADR-0006 Opt-in error reports

## Context
GymDiff runs on friends' Windows PCs next to the game. Today logs stay on the PC (`%LOCALAPPDATA%\GymDiff\logs`, fetched with `just win-logs` over SSH), which only works for the author's own machine. To fix problems seen elsewhere we want error reports to reach the author, without surveilling players.

Constraints: the app is not registered with Riot yet and will never be monetized ([[Vision#Non-goals]]); the local APIs carry personal data (Riot IDs, `puuid`, match data) that must never leave the PC ([[Riot policies]]); the app must keep working offline and never be slowed or crashed by reporting.

## Decision
Add **opt-in, errors-only** central reporting. Decided with the author on 2026-11-08:

| Question | Decision |
|---|---|
| Consent | **Opt-in, off by default.** Explained on the welcome screen, switchable in Settings at any time. |
| What is sent | **Crash and error reports only**: panics and `WARN`/`ERROR` events, app version, OS name. No `INFO`/`DEBUG`, no periodic uploads. |
| Backend | **Sentry** (EU region or self-hosted; `send_default_pii = false`, no user, no server name, IP not stored). |
| Retention | **30 days**, then purged. |
| Deletion | By the random **install id** shown in Settings, on request. |

Design rules:
1. **Scrub at capture.** `gymdiff-diagnostics` scrubs every message when the event happens (paths and user names, Riot IDs, UUIDs/`puuid`, IPv4, e-mails, tokens and passwords). Game payloads are never logged on purpose; scrubbing is the safety net.
2. **Bounded and in memory.** An `ErrorBuffer` keeps the last N entries; nothing is written for reports beyond the consent file.
3. **Preview first.** The player can see the exact JSON (`Report::to_preview_json`) and export it before anything is sent.
4. **Consent is checked on doubt, off.** Missing or corrupt consent file means off. Opting out forgets the install id.
5. **Sending is best-effort and isolated.** Background thread, short timeouts, errors only logged locally, never blocks the UI or the watcher, never touches the game process. Local file logs are unchanged and work offline.
6. **Fixed schema.** The report has five fields (`schema`, `app_version`, `os`, `install_id`, `entries`); a test fails if one is added silently. The Sentry `before_send` hook rebuilds events from this schema only.

## Alternatives considered
- **Opt-out** — weaker consent, harder to defend, not needed for a friend group.
- **Manual "send diagnostics" only** — most private, but misses crashes nobody reports. May stay as the export path.
- **Own HTTPS endpoint** — full control, but we would run and secure a server.
- **Prefilled GitHub issue** — zero backend, but public and fully manual.
- **Full diagnostics / log tail** — more useful, larger privacy surface; revisit with a new ADR if errors alone prove too thin.

## Consequences
- New crate `gymdiff-diagnostics` (pure, no network). Slice 1: scrubber, consent store, capture layer, report and preview. Tested; not yet wired into the app.
- Next slices: wire the capture layer and a panic hook into `init_logging`; Settings and welcome UI (needs a Pencil frame first); Sentry transport behind the consent check (needs a Sentry project and DSN, kept out of the repo).
- A new third-party processor (Sentry) must be named in [[Privacy]] and in the product's Riot registration.
- Status stays `proposed` until the author accepts it after the Sentry project is chosen.
