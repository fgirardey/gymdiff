//! Opt-in reports to Sentry (see `docs/Decisions/ADR-0006 Opt-in error reports.md`).
//!
//! Two Sentry products, each behind its own Settings switch ([`Consent`]):
//! - **Issues** ("Send crash reports"): `ERROR` events and panics with their
//!   stack trace, and native crashes as minidumps, captured by a crash
//!   reporter child process,
//! - **Logs** ("Send all logs"): structured `INFO`, `WARN` and `ERROR` events.
//!
//! Stack traces are symbolicated by Sentry from the debug files CI uploads
//! (see `docs/Decisions/ADR-0008 Versioning and releases.md`).
//!
//! Nothing is sent unless the player opted in **and** the build knows a DSN.
//! `before_send` and `before_send_log` scrub every message and field, and drop
//! what the SDK could add about the machine or the user. Sending is
//! asynchronous and best-effort: it never blocks the UI or the watcher.

use std::borrow::Cow;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

use gymdiff_diagnostics::{Consent, ConsentStore, scrub};
use sentry::integrations::backtrace::{AttachStacktraceIntegration, ProcessStacktraceIntegration};
use sentry::integrations::contexts::ContextIntegration;
use sentry::integrations::debug_images::DebugImagesIntegration;
use sentry::integrations::minidump::MinidumpIntegration;
use sentry::integrations::panic::PanicIntegration;
use sentry::integrations::tracing::{ErrorLayer, LogLayer};
use sentry::protocol::{
    Context, DebugImage, DebugMeta, Event, Exception, Frame, Log, Stacktrace, Thread, Value,
};
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
    let dsn = dsn()?;

    // In the crash reporter process `sentry::init` never returns.
    let guard = sentry::init((dsn, options(&consent, data_dir)));
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

/// In the crash reporter process (re-executed by [`MinidumpIntegration`]),
/// runs the reporter and never returns. Call it before anything else: the
/// child must open no window and no log file.
pub fn run_crash_reporter_if_child(data_dir: &Path) {
    if MinidumpIntegration::new().is_crash_reporter_process() {
        start(data_dir);
        // `start` returns only if consent or the DSN vanished meanwhile.
        std::process::exit(0);
    }
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

fn dsn() -> Option<String> {
    std::env::var("GYMDIFF_SENTRY_DSN")
        .ok()
        .or_else(|| option_env!("GYMDIFF_SENTRY_DSN").map(str::to_owned))
        .filter(|dsn| !dsn.is_empty())
}

/// Our crates only: dependencies (`reqwest`, `hyper`…) never reach Sentry.
fn is_ours(meta: &Metadata<'_>) -> bool {
    meta.target().starts_with("gymdiff")
}

/// `tracing` target of the panic line in the local log: not ours, so the
/// [`ErrorLayer`] skips it and [`PanicIntegration`] alone sends the panic.
const PANIC_TARGET: &str = "panic";

/// Only the integrations we chose (no device context, no host name, no
/// sessions), no breadcrumbs, no traces. Stack traces on issues.
fn options(consent: &Consent, data_dir: &Path) -> sentry::ClientOptions {
    let tags = Tags::of_this_build(consent);
    let log_tags = tags.clone();
    let (crash_reports, logs) = (consent.crash_reports, consent.logs);
    let mut options = sentry::ClientOptions::new()
        .release(BUILD.sentry_release())
        .environment(BUILD.channel.environment())
        // Set, so `ContextIntegration` does not fill in the host name.
        .server_name("")
        .send_default_pii(false)
        .default_integrations(false)
        .max_breadcrumbs(0)
        .attach_stacktrace(true)
        .in_app_include(vec!["gymdiff"])
        .add_integration(ProcessStacktraceIntegration)
        .add_integration(AttachStacktraceIntegration)
        .add_integration(PanicIntegration::new())
        .add_integration(DebugImagesIntegration::new())
        .add_integration(ContextIntegration::new().add_device(false))
        .before_send(move |event| {
            (crash_reports && take_issue_slot()).then(|| sanitize_event(event, &tags))
        })
        .before_send_log(move |log| logs.then(|| sanitize_log(log, &log_tags)));
    if crash_reports {
        options = options.add_integration(
            MinidumpIntegration::new()
                .crashes_dir(data_dir.join("crashes"))
                .process_name("GymDiff crash reporter"),
        );
    }
    options
}

/// Tags added to every issue and attribute added to every log.
#[derive(Clone)]
struct Tags(Vec<(&'static str, String)>);

impl Tags {
    fn of_this_build(consent: &Consent) -> Self {
        Self(vec![
            ("install_id", consent.install_id.clone().unwrap_or_default()),
            ("os", std::env::consts::OS.to_owned()),
            ("build", BUILD.number.to_string()),
            ("commit", BUILD.commit.to_owned()),
            ("channel", BUILD.channel.environment().to_owned()),
            ("target", BUILD.target.to_owned()),
        ])
    }
}

/// Last line of defence for issues: keep only what we need, scrubbed.
fn sanitize_event(event: Event<'static>, extra_tags: &Tags) -> Event<'static> {
    let mut tags = event.tags;
    for (key, value) in &extra_tags.0 {
        tags.insert((*key).to_owned(), value.clone());
    }
    let contexts = event
        .contexts
        .into_iter()
        .filter_map(|(key, context)| match context {
            // Links the issue to the logs of the same session.
            Context::Trace(_) => Some((key, context)),
            // OS name and version, compiler version: nothing about the machine.
            Context::Os(mut os) => {
                os.other.clear();
                Some((key, Context::Os(os)))
            }
            Context::Runtime(mut runtime) => {
                runtime.other.retain(|key, _| key == "channel");
                Some((key, Context::Runtime(runtime)))
            }
            // The `tracing` fields of the event.
            Context::Other(fields) => Some((key, Context::Other(scrub_map(fields)))),
            _ => None,
        })
        .collect();
    let mut exception = event.exception;
    for Exception {
        value, stacktrace, ..
    } in &mut exception.values
    {
        *value = value.as_deref().map(scrub);
        sanitize_stacktrace(stacktrace.as_mut());
    }
    let mut threads = event.threads;
    for Thread {
        name, stacktrace, ..
    } in &mut threads.values
    {
        *name = name.as_deref().map(scrub);
        sanitize_stacktrace(stacktrace.as_mut());
    }
    let mut stacktrace = event.stacktrace;
    sanitize_stacktrace(stacktrace.as_mut());
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
        threads,
        stacktrace,
        debug_meta: Cow::Owned(sanitize_debug_meta(event.debug_meta.into_owned())),
        tags: tags.into_iter().map(|(k, v)| (k, scrub(&v))).collect(),
        contexts,
        ..Event::default()
    }
}

/// Frames keep function, module, line and address; paths are reduced to the
/// file name and local variables or source lines from the client are dropped
/// (Sentry adds the source from the uploaded bundle).
fn sanitize_stacktrace(stacktrace: Option<&mut Stacktrace>) {
    let Some(stacktrace) = stacktrace else { return };
    stacktrace.registers.clear();
    for frame in &mut stacktrace.frames {
        let Frame {
            filename,
            abs_path,
            vars,
            pre_context,
            context_line,
            post_context,
            ..
        } = frame;
        *filename = filename.as_deref().map(file_name);
        *abs_path = abs_path.as_deref().map(file_name);
        vars.clear();
        pre_context.clear();
        *context_line = None;
        post_context.clear();
    }
}

/// Loaded modules, needed to symbolicate: their path holds the user name, so
/// only the file name stays.
fn sanitize_debug_meta(mut meta: DebugMeta) -> DebugMeta {
    for image in &mut meta.images {
        match image {
            DebugImage::Symbolic(image) => {
                image.name = file_name(&image.name);
                image.debug_file = image.debug_file.as_deref().map(file_name);
            }
            DebugImage::Apple(image) => image.name = file_name(&image.name),
            _ => {}
        }
    }
    meta
}

/// The last component of a Windows or Unix path, scrubbed.
fn file_name(path: &str) -> String {
    scrub(path.rsplit(['\\', '/']).next().unwrap_or(path))
}

/// Last line of defence for logs: scrub the body and every attribute, drop
/// what identifies the machine or the user.
fn sanitize_log(mut log: Log, extra: &Tags) -> Log {
    log.body = scrub(&log.body);
    log.attributes
        .retain(|key, _| key != "server.address" && !key.starts_with("user."));
    for attribute in log.attributes.values_mut() {
        attribute.0 = scrub_value(std::mem::take(&mut attribute.0));
    }
    for (key, value) in &extra.0 {
        let key = if *key == "os" { "os.name" } else { key };
        log.attributes.insert(key.to_owned(), value.clone().into());
    }
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

/// Panics reach the local log through `tracing`, then [`PanicIntegration`]'s
/// hook (installed by `sentry::init`, called next) sends them with their
/// stack trace and flushes.
fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!(target: PANIC_TARGET, "panic: {info}");
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

    use sentry::protocol::{
        IpAddress, LogAttribute, LogLevel, Map, SymbolicDebugImage, User, Values,
    };

    use sentry::types::DebugId;

    use super::*;

    const PERSONAL: &str = r"cannot open C:\Users\Florian\x for Florian#EUW";
    const EXE: &str = r"C:\Users\Florian\Downloads\gymdiff.exe";

    fn assert_clean(text: &str) {
        assert!(!text.contains("Florian"), "{text}");
    }

    fn tags() -> Tags {
        Tags(vec![
            ("install_id", "abc".to_owned()),
            ("build", "57".to_owned()),
        ])
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
                ("device".to_owned(), Context::Device(Box::default())),
            ]),
            ..Default::default()
        };

        let clean = sanitize_event(event, &tags());
        assert!(clean.server_name.is_none() && clean.user.is_none());
        assert_clean(clean.message.as_deref().unwrap());
        assert_clean(clean.exception.values[0].value.as_deref().unwrap());
        assert_eq!(
            clean.tags.get("install_id").map(String::as_str),
            Some("abc")
        );
        assert_eq!(clean.tags.get("build").map(String::as_str), Some("57"));
        assert!(clean.contexts.contains_key("os"));
        assert!(!clean.contexts.contains_key("device"));
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

        let clean = sanitize_log(log, &tags());
        assert_clean(&clean.body);
        assert!(!clean.attributes.contains_key("server.address"));
        assert!(!clean.attributes.contains_key("user.name"));
        assert_clean(clean.attributes["source"].0.as_str().unwrap());
        assert_eq!(clean.attributes["consecutive_errors"].0, Value::from(2));
        assert_eq!(clean.attributes["install_id"].0, Value::from("abc"));
    }

    #[test]
    fn stack_traces_and_modules_keep_only_file_names() {
        let frame = Frame {
            function: Some("gymdiff_app::watcher::run".to_owned()),
            filename: Some(r"C:\Users\Florian\src\watcher.rs".to_owned()),
            abs_path: Some("/Users/Florian/src/watcher.rs".to_owned()),
            lineno: Some(42),
            context_line: Some(PERSONAL.to_owned()),
            ..Default::default()
        };
        let event = Event {
            exception: Values::from(vec![Exception {
                stacktrace: Some(Stacktrace {
                    frames: vec![frame.clone()],
                    ..Default::default()
                }),
                ..Default::default()
            }]),
            threads: Values::from(vec![Thread {
                name: Some("watcher".to_owned()),
                stacktrace: Some(Stacktrace {
                    frames: vec![frame],
                    ..Default::default()
                }),
                ..Default::default()
            }]),
            debug_meta: Cow::Owned(DebugMeta {
                images: vec![DebugImage::Symbolic(SymbolicDebugImage {
                    name: EXE.to_owned(),
                    arch: None,
                    image_addr: 0.into(),
                    image_size: 0,
                    image_vmaddr: 0.into(),
                    id: DebugId::default(),
                    code_id: None,
                    debug_file: Some(r"C:\Users\Florian\Downloads\gymdiff.pdb".to_owned()),
                })],
                ..Default::default()
            }),
            ..Default::default()
        };

        let clean = sanitize_event(event, &tags());
        let frames = clean.exception.values[0]
            .stacktrace
            .as_ref()
            .unwrap()
            .frames
            .iter()
            .chain(&clean.threads.values[0].stacktrace.as_ref().unwrap().frames);
        for frame in frames {
            assert_eq!(frame.filename.as_deref(), Some("watcher.rs"));
            assert_eq!(frame.abs_path.as_deref(), Some("watcher.rs"));
            assert_eq!(frame.lineno, Some(42));
            assert!(frame.context_line.is_none());
        }
        let DebugImage::Symbolic(image) = &clean.debug_meta.images[0] else {
            panic!("debug image dropped");
        };
        assert_eq!(image.name, "gymdiff.exe");
        assert_eq!(image.debug_file.as_deref(), Some("gymdiff.pdb"));
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
