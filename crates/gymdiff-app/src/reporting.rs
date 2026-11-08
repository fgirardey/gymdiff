//! Opt-in error reports to Sentry (see `docs/Decisions/ADR-0006 Opt-in error reports.md`).
//!
//! Nothing is sent unless the player opted in ([`ConsentStore`]) **and** the
//! build knows a DSN. Events are rebuilt from the fixed [`Report`] schema, so
//! the Sentry SDK can never add anything of its own. Sending is asynchronous
//! and best-effort: it never blocks the UI or the watcher.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gymdiff_diagnostics::{ConsentStore, ErrorBuffer, Report, scrub};
use sentry::protocol::{Event, Level, Value};
use tracing::Subscriber;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

/// Starts reporting if the player opted in. Keep the guard alive until exit.
///
/// The DSN comes from `GYMDIFF_SENTRY_DSN`, at run time first (development),
/// then baked in at build time (CI). Without one, reporting stays off.
pub fn start(data_dir: &Path, buffer: &ErrorBuffer) -> Option<sentry::ClientInitGuard> {
    let consent = ConsentStore::new(data_dir).load();
    if !consent.enabled {
        tracing::debug!("error reports are off (no consent)");
        return None;
    }
    let dsn = std::env::var("GYMDIFF_SENTRY_DSN")
        .ok()
        .or_else(|| option_env!("GYMDIFF_SENTRY_DSN").map(str::to_owned))
        .filter(|dsn| !dsn.is_empty())?;

    let guard = sentry::init((
        dsn,
        sentry::ClientOptions {
            release: Some(Cow::Borrowed(env!("CARGO_PKG_VERSION"))),
            send_default_pii: false,
            server_name: None,
            default_integrations: false,
            max_breadcrumbs: 0,
            attach_stacktrace: false,
            auto_session_tracking: false,
            before_send: Some(Arc::new(|event| Some(sanitize(event)))),
            ..Default::default()
        },
    ));
    if !guard.is_enabled() {
        tracing::warn!("error reports enabled but the DSN is invalid");
        return None;
    }
    install_panic_hook(buffer.clone(), consent.install_id.unwrap_or_default());
    tracing::info!("error reports are on");
    Some(guard)
}

/// `tracing` layer: sends a report when an `ERROR` is logged (at most one per
/// minute). Add it **after** the [`ErrorBuffer`] layer so the error is already
/// in the buffer. Does nothing while reporting is off.
pub struct ReportLayer {
    buffer: ErrorBuffer,
    install_id: String,
}

impl ReportLayer {
    #[must_use]
    pub fn new(buffer: ErrorBuffer, data_dir: &Path) -> Self {
        let install_id = ConsentStore::new(data_dir)
            .load()
            .install_id
            .unwrap_or_default();
        Self { buffer, install_id }
    }
}

impl<S: Subscriber> Layer<S> for ReportLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        if *event.metadata().level() != tracing::Level::ERROR
            || !sentry::Hub::current()
                .client()
                .is_some_and(|c| c.is_enabled())
        {
            return;
        }
        if rate_limited(now_secs()) {
            return;
        }
        send(&Report::new(&self.install_id, self.buffer.snapshot()));
    }
}

/// Builds the Sentry event for a report: only [`Report`] fields, nothing else.
fn event_from_report(report: &Report) -> Event<'static> {
    let last = report.entries.last();
    let mut tags = BTreeMap::new();
    tags.insert("install_id".to_owned(), report.install_id.clone());
    tags.insert("os".to_owned(), report.os.clone());
    let mut extra = BTreeMap::new();
    extra.insert(
        "entries".to_owned(),
        serde_json::to_value(&report.entries).unwrap_or(Value::Null),
    );
    Event {
        level: Level::Error,
        message: last.map(|entry| entry.message.clone()),
        logger: last.map(|entry| entry.target.clone()),
        release: Some(Cow::Owned(report.app_version.clone())),
        tags,
        extra,
        ..Event::default()
    }
}

/// Last line of defence in `before_send`: keep only what we put in ourselves.
fn sanitize(event: Event<'static>) -> Event<'static> {
    Event {
        event_id: event.event_id,
        timestamp: event.timestamp,
        level: event.level,
        message: event.message.map(|message| scrub(&message)),
        logger: event.logger,
        release: event.release,
        tags: event.tags,
        extra: event.extra,
        ..Event::default()
    }
}

fn send(report: &Report) {
    sentry::capture_event(event_from_report(report));
}

fn install_panic_hook(buffer: ErrorBuffer, install_id: String) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|l| format!(" at {}:{}", l.file(), l.line()))
            .unwrap_or_default();
        let message = scrub(&format!("panic: {info}{location}"));
        // Through `tracing` so the local log and the buffer see it too.
        tracing::error!("{message}");
        send(&Report::new(&install_id, buffer.snapshot()));
        if let Some(client) = sentry::Hub::current().client() {
            client.flush(Some(Duration::from_secs(2)));
        }
        previous(info);
    }));
}

/// Minimum delay between two automatic reports.
const MIN_INTERVAL_SECS: u64 = 60;

static LAST_SENT: AtomicU64 = AtomicU64::new(0);

/// True when a report was already sent less than a minute ago; records `now` otherwise.
fn rate_limited(now: u64) -> bool {
    let last = LAST_SENT.load(Ordering::Relaxed);
    if last != 0 && now.saturating_sub(last) < MIN_INTERVAL_SECS {
        return true;
    }
    LAST_SENT.store(now, Ordering::Relaxed);
    false
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |d| d.as_secs().max(1))
}

#[cfg(test)]
mod tests {
    use gymdiff_diagnostics::Entry;

    use super::*;

    fn report() -> Report {
        Report::new(
            "abc",
            vec![Entry {
                level: "ERROR".into(),
                target: "gymdiff_app::watcher".into(),
                message: "boom".into(),
            }],
        )
    }

    #[test]
    fn event_carries_only_report_fields() {
        let event = event_from_report(&report());
        assert_eq!(event.message.as_deref(), Some("boom"));
        assert_eq!(
            event.tags.get("install_id").map(String::as_str),
            Some("abc")
        );
        assert!(event.user.is_none() && event.server_name.is_none());
        assert!(event.contexts.is_empty() && event.breadcrumbs.is_empty());
    }

    #[test]
    fn sanitize_drops_everything_the_sdk_may_add() {
        let mut event = event_from_report(&report());
        event.server_name = Some("florian-pc".into());
        event.user = Some(sentry::protocol::User {
            ip_address: Some(sentry::protocol::IpAddress::Auto),
            username: Some("Florian".into()),
            ..Default::default()
        });
        event.message = Some(r"cannot open C:\Users\Florian\x".into());
        let clean = sanitize(event);
        assert!(clean.server_name.is_none() && clean.user.is_none());
        assert!(!clean.message.unwrap().contains("Florian"));
    }

    #[test]
    fn at_most_one_report_per_minute() {
        LAST_SENT.store(0, Ordering::Relaxed);
        assert!(!rate_limited(1_000));
        assert!(rate_limited(1_030));
        assert!(!rate_limited(1_061));
    }
}
