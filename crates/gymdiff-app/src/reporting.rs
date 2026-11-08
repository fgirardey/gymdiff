//! Opt-in reports to Sentry (see `docs/Decisions/ADR-0006 Opt-in error reports.md`).
//!
//! Two Sentry products, each behind its own Settings switch ([`Consent`]):
//! - **Issues**: `ERROR` events and panics ("Send crash reports"),
//! - **Logs**: structured `INFO`, `WARN` and `ERROR` events ("Send all logs").
//!
//! Nothing is sent unless the player opted in **and** the build knows a DSN.
//! `before_send` and `before_send_log` scrub every message and field, and drop
//! what the SDK could add about the machine or the user. Sending is
//! asynchronous and best-effort: it never blocks the UI or the watcher.

use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use gymdiff_diagnostics::{Consent, ConsentStore, scrub};
use sentry::integrations::tracing::{ErrorLayer, LogLayer};
use sentry::protocol::{Context, Event, Exception, Log, Value};
use tracing::{Level, Metadata, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::registry::LookupSpan;

use crate::build_info::BUILD;

/// Starts reporting if the player opted in. Keep the guard alive until exit:
/// dropping it flushes the queued logs.
///
/// The DSN comes from `GYMDIFF_SENTRY_DSN`, at run time first (development),
/// then baked in at build time (CI). Without one, reporting stays off.
pub fn start(data_dir: &Path) -> Option<sentry::ClientInitGuard> {
    let consent = ConsentStore::new(data_dir).load();
    if !consent.any() {
        tracing::debug!("reports are off (no consent)");
        return None;
    }
    let dsn = std::env::var("GYMDIFF_SENTRY_DSN")
        .ok()
        .or_else(|| option_env!("GYMDIFF_SENTRY_DSN").map(str::to_owned))
        .filter(|dsn| !dsn.is_empty())?;

    let guard = sentry::init((dsn, options(&consent)));
    if !guard.is_enabled() {
        tracing::warn!("reports enabled but the DSN is invalid");
        return None;
    }
    install_panic_hook();
    tracing::info!(
        crash_reports = consent.crash_reports,
        logs = consent.logs,
        "reports are on"
    );
    Some(guard)
}

/// The Sentry layers of the `tracing` subscriber: GymDiff's own `INFO`+ events
/// become Sentry logs, its `ERROR` events become Sentry issues. They do nothing
/// until [`start`] binds a client.
pub fn layers<S>() -> impl Layer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    LogLayer::new()
        .with_filter(filter_fn(|meta| {
            is_ours(meta) && *meta.level() <= Level::INFO
        }))
        .and_then(ErrorLayer::new().with_filter(filter_fn(|meta| {
            is_ours(meta) && *meta.level() == Level::ERROR
        })))
}

/// Our crates only: dependencies (`reqwest`, `hyper`…) never reach Sentry.
fn is_ours(meta: &Metadata<'_>) -> bool {
    meta.target().starts_with("gymdiff")
}

/// Everything optional is off: no default integrations (no OS or device
/// context), no breadcrumbs, no traces, no sessions, no stack traces.
fn options(consent: &Consent) -> sentry::ClientOptions {
    let install_id = consent.install_id.clone().unwrap_or_default();
    let log_install_id = install_id.clone();
    let (crash_reports, logs) = (consent.crash_reports, consent.logs);
    sentry::ClientOptions::new()
        .release(BUILD.sentry_release())
        .environment(BUILD.channel.environment())
        .send_default_pii(false)
        .default_integrations(false)
        .max_breadcrumbs(0)
        .attach_stacktrace(false)
        .before_send(move |event| {
            (crash_reports && take_issue_slot()).then(|| sanitize_event(event, &install_id))
        })
        .before_send_log(move |log| logs.then(|| sanitize_log(log, &log_install_id)))
}

/// Last line of defence for issues: keep only what we need, scrubbed.
fn sanitize_event(event: Event<'static>, install_id: &str) -> Event<'static> {
    let mut tags = event.tags;
    tags.insert("install_id".to_owned(), install_id.to_owned());
    tags.insert("os".to_owned(), std::env::consts::OS.to_owned());
    let contexts = event
        .contexts
        .into_iter()
        .filter_map(|(key, context)| match context {
            // Links the issue to the logs of the same session.
            Context::Trace(_) => Some((key, context)),
            // The `tracing` fields of the event.
            Context::Other(fields) => Some((key, Context::Other(scrub_map(fields)))),
            _ => None,
        })
        .collect();
    let mut exception = event.exception;
    for Exception { value, .. } in &mut exception.values {
        *value = value.as_deref().map(scrub);
    }
    Event {
        event_id: event.event_id,
        timestamp: event.timestamp,
        level: event.level,
        platform: event.platform,
        logger: event.logger,
        release: event.release,
        environment: event.environment,
        message: event.message.map(|message| scrub(&message)),
        exception,
        tags: tags.into_iter().map(|(k, v)| (k, scrub(&v))).collect(),
        contexts,
        ..Event::default()
    }
}

/// Last line of defence for logs: scrub the body and every attribute, drop
/// what identifies the machine or the user.
fn sanitize_log(mut log: Log, install_id: &str) -> Log {
    log.body = scrub(&log.body);
    log.attributes
        .retain(|key, _| key != "server.address" && !key.starts_with("user."));
    for attribute in log.attributes.values_mut() {
        attribute.0 = scrub_value(std::mem::take(&mut attribute.0));
    }
    log.attributes
        .insert("install_id".to_owned(), install_id.to_owned().into());
    log.attributes
        .insert("os.name".to_owned(), std::env::consts::OS.into());
    log
}

fn scrub_map<M>(map: M) -> M
where
    M: IntoIterator<Item = (String, Value)> + FromIterator<(String, Value)>,
{
    map.into_iter().map(|(k, v)| (k, scrub_value(v))).collect()
}

fn scrub_value(value: Value) -> Value {
    match value {
        Value::String(text) => Value::String(scrub(&text)),
        Value::Array(items) => Value::Array(items.into_iter().map(scrub_value).collect()),
        Value::Object(map) => Value::Object(scrub_map(map)),
        other => other,
    }
}

/// Panics go through `tracing`, so they reach the local log and, through the
/// [`ErrorLayer`], Sentry as an issue. Flushes before the process may die.
fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!("panic: {info}");
        if let Some(client) = sentry::Hub::current().client() {
            client.flush(Some(Duration::from_secs(2)));
        }
        previous(info);
    }));
}

/// Most issues sent per launch: a bug in a loop must not burn the Sentry
/// quota. Logs keep the full picture.
const MAX_ISSUES_PER_LAUNCH: u32 = 10;

static ISSUES_SENT: AtomicU32 = AtomicU32::new(0);

/// True, and counts it, while the per-launch issue budget is not spent.
fn take_issue_slot() -> bool {
    ISSUES_SENT.fetch_add(1, Ordering::Relaxed) < MAX_ISSUES_PER_LAUNCH
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use sentry::protocol::{IpAddress, LogAttribute, LogLevel, Map, User, Values};

    use super::*;

    const PERSONAL: &str = r"cannot open C:\Users\Florian\x for Florian#EUW";

    fn assert_clean(text: &str) {
        assert!(!text.contains("Florian"), "{text}");
    }

    #[test]
    fn issues_keep_only_scrubbed_essentials() {
        let mut fields = Map::new();
        fields.insert(
            "path".to_owned(),
            Value::from(r"C:\Users\Florian\gymdiff.log"),
        );
        let event = Event {
            message: Some(PERSONAL.to_owned()),
            server_name: Some("florian-pc".into()),
            user: Some(User {
                ip_address: Some(IpAddress::Auto),
                username: Some("Florian".into()),
                ..Default::default()
            }),
            exception: Values::from(vec![Exception {
                value: Some(PERSONAL.to_owned()),
                ..Default::default()
            }]),
            contexts: BTreeMap::from([
                ("Rust Tracing Fields".to_owned(), Context::Other(fields)),
                ("os".to_owned(), Context::Os(Box::default())),
            ]),
            ..Default::default()
        };

        let clean = sanitize_event(event, "abc");
        assert!(clean.server_name.is_none() && clean.user.is_none());
        assert_clean(clean.message.as_deref().unwrap());
        assert_clean(clean.exception.values[0].value.as_deref().unwrap());
        assert_eq!(
            clean.tags.get("install_id").map(String::as_str),
            Some("abc")
        );
        assert!(!clean.contexts.contains_key("os"));
        let Some(Context::Other(fields)) = clean.contexts.get("Rust Tracing Fields") else {
            panic!("tracing fields dropped");
        };
        assert_clean(fields["path"].as_str().unwrap());
    }

    #[test]
    fn logs_are_scrubbed_and_anonymous() {
        let log = Log {
            level: LogLevel::Info,
            body: PERSONAL.to_owned(),
            trace_id: None,
            timestamp: std::time::SystemTime::now(),
            severity_number: None,
            attributes: BTreeMap::from([
                (
                    "server.address".to_owned(),
                    LogAttribute::from("florian-pc"),
                ),
                ("user.name".to_owned(), LogAttribute::from("Florian")),
                ("source".to_owned(), LogAttribute::from(PERSONAL)),
                ("consecutive_errors".to_owned(), LogAttribute::from(2)),
            ]),
        };

        let clean = sanitize_log(log, "abc");
        assert_clean(&clean.body);
        assert!(!clean.attributes.contains_key("server.address"));
        assert!(!clean.attributes.contains_key("user.name"));
        assert_clean(clean.attributes["source"].0.as_str().unwrap());
        assert_eq!(clean.attributes["consecutive_errors"].0, Value::from(2));
        assert_eq!(clean.attributes["install_id"].0, Value::from("abc"));
    }

    #[test]
    fn issues_stop_after_the_per_launch_budget() {
        ISSUES_SENT.store(0, Ordering::Relaxed);
        let sent = (0..MAX_ISSUES_PER_LAUNCH + 5)
            .filter(|_| take_issue_slot())
            .count();
        assert_eq!(sent, MAX_ISSUES_PER_LAUNCH as usize);
    }
}
