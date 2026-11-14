---
status: proposed
date: 2026-11-08
---

# ADR-0006 Opt-in error reports

## Context
cooldown.lol runs on friends' Windows PCs next to the game. Today logs stay on the PC (`%LOCALAPPDATA%\cooldown.lol\logs`, fetched with `just win-logs` over SSH), which only works for the author's own machine. To fix problems seen elsewhere we want error reports to reach the author, without surveilling players.

Constraints: the app is not registered with Riot yet and will never be monetized ([[Vision#Non-goals]]); the local APIs carry personal data (Riot IDs, `puuid`, match data) that must never leave the PC ([[Riot policies]]); the app must keep working offline and never be slowed or crashed by reporting.

## Decision
Add **opt-in** central reporting to Sentry, with two independent switches. Decided with the author on 2026-11-08, revised the same day to use Sentry Logs, then to add stack traces and native crash reports ([[ADR-0008 Versioning and releases]]):

| Question | Decision |
|---|---|
| Consent | **Opt-in, off by default**, two switches in Settings: **Send crash reports** and **Send all logs**. Changeable at any time. |
| What is sent | **Send crash reports** → Sentry **Issues**: `ERROR` events and panics with their **stack trace**, the list of loaded modules (file names only) and the OS name and version; **native crashes** as a **minidump**. **Send all logs** → Sentry **Logs**: structured `INFO`, `WARN` and `ERROR` events. Both carry the build (version, build number, commit, channel), OS name and install id. Never `DEBUG`/`TRACE`, never events from dependencies. Nothing else: no traces, breadcrumbs or sessions. |
| Backend | **Sentry** (EU region or self-hosted; `send_default_pii = false`, no user, no server name, IP not stored). |
| Retention | **30 days**, then purged. |
| Deletion | By the random **install id** shown in Settings, on request. |

Design rules:
1. **Scrub before sending.** `before_send` (issues) and `before_send_log` (logs) run `cooldown-diagnostics::scrub` on every message and every field (paths and user names, Riot IDs, UUIDs/`puuid`, IPv4, e-mails, tokens and passwords). Game payloads are never logged on purpose; scrubbing is the safety net.
2. **Nothing about the machine or the user.** SDK default integrations are off; only chosen ones are added (panics, stack traces, loaded modules, OS and Rust contexts, minidumps; no device context, no host name). `server.address` and `user.*` are dropped; issues keep only their message, exception values, stack traces, loaded modules, OS and Rust contexts, level, logger, release, environment, tags, the `tracing` fields and the trace context (which links an issue to the logs of the same session). In stack frames and modules, paths are cut to the file name (an executable path holds the Windows user name), and local variables and source lines from the client are dropped (Sentry adds the source from the bundle CI uploads).
6. **Minidumps cannot be scrubbed on the PC.** A native crash (access violation, stack overflow) is caught by a crash reporter child process (`sentry-minidump`) that sends a minidump: the threads' registers and stack memory at the crash, which may hold a Riot ID or the LCU token while a game runs. Mitigations: sent only under **Send crash reports**, whose text says so; the Sentry project keeps *Store Native Crash Reports* **off**, so the dump is processed into a stack trace and then discarded; server-side data scrubbing is on.
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
- Crate `cooldown-diagnostics` (pure, no network): scrubber and consent store (`diagnostics.json`: `crash_reports`, `logs`, `install_id`).
- `cooldown-app/src/reporting.rs` is the only code that talks to Sentry (SDK `sentry` 0.49 with `tracing`, `logs`, `backtrace`, `debug-images`, `contexts`, `panic` and `minidump`). The crash reporter re-executes `cooldown` with a marker variable; `main` hands over to it before opening a window or a log file. It starts only with consent **and** a DSN. `LogLayer` (`INFO`+) feeds Sentry Logs and `ErrorLayer` (`ERROR`) feeds Sentry Issues, both limited to cooldown.lol's own crates; each is gated by its switch in `before_send` / `before_send_log`. Panics are written to the local log through `tracing` and sent by the SDK's panic integration, with their stack trace.
- Release `cooldown@<version>+<build>` and environment = channel (`production` for tagged releases, `preview` for CI artifacts, `development` for local builds), see [[ADR-0008 Versioning and releases]].
- The DSN is not committed: `COOLDOWN_SENTRY_DSN` at run time, or baked in at build time from the CI repository variable of the same name. It is a write-only key, not a secret, but keeping it out of the source avoids noise from forks and clones.
- Sentry project in the **EU** region (`ingest.de.sentry.io`), created 2026-11-08. An earlier project in the US region was dropped before any event was sent.
- The Settings screen ([[Settings]]) sets the consent; a change applies at the next launch.
- A new third-party processor (Sentry) must be named in [[Privacy]] and in the product's Riot registration.
- Status stays `proposed` until the author accepts it after the Sentry project is chosen.
