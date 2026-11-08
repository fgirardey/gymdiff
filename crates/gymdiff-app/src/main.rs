//! # GymDiff desktop app
//!
//! Threads:
//! - main thread: Slint event loop (UI),
//! - background thread: Tokio runtime running the [`watcher`], which polls the
//!   League client (real or simulated) and pushes state updates to the UI.

mod platform;
mod reporting;
mod settings;
mod watcher;

use std::path::PathBuf;

use gymdiff_diagnostics::ErrorBuffer;
use slint::ComponentHandle;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    let data_dir = data_dir();
    let (_log_guard, errors) = init_logging(&data_dir);
    let _reports_guard = reporting::start(&data_dir, &errors);
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        platform = platform::NAME,
        "starting GymDiff"
    );

    let window = AppWindow::new()?;
    platform::configure_main_window(window.window());
    window.set_version(env!("CARGO_PKG_VERSION").into());

    let welcome = settings::WelcomeFlag::new(&data_dir);
    if !welcome.is_seen() {
        tracing::info!(data_dir = %data_dir.display(), "first launch: showing the welcome");
        window.set_screen(Screen::Welcome);
    }
    window.on_welcome_done(move || {
        if let Err(error) = welcome.mark_seen() {
            tracing::warn!(%error, "cannot record that the welcome was seen");
        }
    });

    let weak = window.as_weak();
    std::thread::Builder::new()
        .name("watcher".into())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("failed to build the Tokio runtime");
            runtime.block_on(watcher::run(weak));
        })?;

    window.run()?;
    Ok(())
}

/// The settings folder: [`platform::data_dir`], or `GYMDIFF_DATA_DIR` when set
/// (point it at an empty folder to see the welcome screen again).
fn data_dir() -> PathBuf {
    std::env::var_os("GYMDIFF_DATA_DIR").map_or_else(platform::data_dir, PathBuf::from)
}

/// Logs to stderr and to a daily-rotated file in [`platform::log_dir`].
/// Attach that file when reporting a Windows-only bug. Also feeds the opt-in
/// error reports ([`reporting`]).
fn init_logging(
    data_dir: &std::path::Path,
) -> (
    Option<tracing_appender::non_blocking::WorkerGuard>,
    ErrorBuffer,
) {
    let errors = ErrorBuffer::new(50);
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,gymdiff=debug,gymdiff_lol=debug"));
    let stderr = fmt::layer().with_writer(std::io::stderr);

    let log_dir = platform::log_dir();
    let (file, guard) = match std::fs::create_dir_all(&log_dir) {
        Ok(()) => {
            let appender = tracing_appender::rolling::daily(&log_dir, "gymdiff.log");
            let (writer, guard) = tracing_appender::non_blocking(appender);
            (
                Some(fmt::layer().with_ansi(false).with_writer(writer)),
                Some(guard),
            )
        }
        Err(_) => (None, None),
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(stderr)
        .with(file)
        .with(errors.layer())
        .with(reporting::ReportLayer::new(errors.clone(), data_dir))
        .init();
    tracing::info!(log_dir = %log_dir.display(), "logging initialised");
    (guard, errors)
}
