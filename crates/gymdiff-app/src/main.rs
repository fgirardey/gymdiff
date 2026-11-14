// Release builds are GUI apps on Windows: no console window next to GymDiff.
// Logs go to the file in `platform::log_dir`; debug builds keep the console.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! # GymDiff desktop app
//!
//! Threads:
//! - main thread: Slint event loop (UI) and the notification area icon
//!   ([`tray`]); closing the window hides it, the loop runs until Quit GymDiff,
//! - `single-instance` thread: shows the window when GymDiff is started again
//!   ([`single_instance`]),
//! - background thread: Tokio runtime running the [`watcher`], which polls the
//!   League client (real or simulated) and pushes state updates to the UI,
//! - `updater` thread (installed alpha builds): checks for and downloads new
//!   builds, which Velopack applies at the next launch ([`updater`]).

mod build_info;
mod debug_events;
mod history_import;
mod history_store;
mod platform;
mod reporting;
mod settings;
mod shortcut_settings;
mod single_instance;
mod startup_settings;
mod tray;
mod uninstall;
mod updater;
mod watcher;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use build_info::BUILD;
use single_instance::SingleInstance;
use slint::ComponentHandle;
use slint::winit_030::{WinitWindowAccessor, winit};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    // CI reads the version of the binary it just built (see ADR-0008).
    if std::env::args().nth(1).as_deref() == Some("--version") {
        println!("gymdiff {}", BUILD.full_version());
        return Ok(());
    }
    // Started with Windows (`docs/Product/Features/Running in the background.md`):
    // only the notification area icon, no window.
    let background = std::env::args().any(|arg| arg == gymdiff_shortcuts::BACKGROUND_ARG);
    let data_dir = data_dir();
    // In the crash reporter child process this never returns: it must run
    // before anything opens a window or a log file.
    reporting::run_crash_reporter_if_child(&data_dir);
    // Installer hooks and updates downloaded earlier (Velopack, ADR-0011): may
    // exit or restart GymDiff, so before the log file and the window, and after
    // the crash reporter's own exit so that child never applies an update.
    updater::run_installer_hooks(&data_dir);
    let _log_guard = init_logging();
    let _reports_guard = reporting::start(&data_dir);
    tracing::info!(
        version = BUILD.version,
        build = BUILD.number,
        commit = BUILD.commit,
        channel = ?BUILD.channel,
        target = BUILD.target,
        rustc = BUILD.rustc,
        platform = platform::NAME,
        "starting GymDiff"
    );
    debug_events::fire_from_env();

    let window = AppWindow::new()?;
    let weak = window.as_weak();
    let show_again = move || {
        let _ = weak.upgrade_in_event_loop(|window| reveal(&window));
    };
    if let SingleInstance::AlreadyRunning =
        single_instance::claim(&data_dir, background, show_again)
    {
        return Ok(());
    }
    platform::configure_main_window(window.window());
    window.set_version(BUILD.display().into());

    let settings = settings::SettingsStore::new(&data_dir);
    let welcome_seen = settings.load().welcome_seen;
    if !welcome_seen {
        tracing::info!(data_dir = %data_dir.display(), "first launch: showing the welcome");
        window.set_screen(Screen::Welcome);
    }
    window.on_welcome_done(move || {
        if let Err(error) = settings.update(|s| s.welcome_seen = true) {
            tracing::warn!(%error, "cannot record that the welcome was seen");
        }
    });

    // Reporting starts or stops at the next launch (see `reporting::start`).
    let consent = gymdiff_diagnostics::ConsentStore::new(&data_dir);
    let choice = consent.load();
    window.set_crash_reports(choice.crash_reports);
    window.set_all_logs(choice.logs);
    let crash_consent = consent.clone();
    window.on_crash_reports_changed(move |enabled| {
        match crash_consent.set_crash_reports(enabled) {
            Ok(_) => tracing::info!(enabled, "crash reports choice saved"),
            Err(error) => tracing::warn!(%error, "cannot save the crash reports choice"),
        }
    });
    window.on_all_logs_changed(move |enabled| match consent.set_logs(enabled) {
        Ok(_) => tracing::info!(enabled, "all logs choice saved"),
        Err(error) => tracing::warn!(%error, "cannot save the all logs choice"),
    });

    // Kept alive until exit: dropping it would stop "Check now".
    let updater = updater::Updater::start(window.as_weak()).map(std::rc::Rc::new);
    window.set_update_visible(updater.is_some());
    if let Some(updater) = updater.clone() {
        window.on_check_updates(move || updater.check_now());
    }

    let installed = updater::is_installed();
    shortcut_settings::bind(&window, installed);
    startup_settings::bind(&window, installed);
    uninstall::bind(&window, &data_dir, installed);

    window.set_debug_tools(cfg!(debug_assertions));
    window.on_debug_event(|kind| debug_events::fire(kind.as_str()));

    // Without a readable history the app still works, it just has none (see `HistoryStore::open`).
    let history = match history_store::HistoryStore::open(&data_dir) {
        Ok(store) => Some(store),
        Err(error) => {
            tracing::warn!(%error, "cannot open the history: going on without it");
            None
        }
    };

    let weak = window.as_weak();
    std::thread::Builder::new()
        .name("watcher".into())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("failed to build the Tokio runtime");
            runtime.block_on(watcher::run(weak, history));
        })?;

    let tray = start_tray(&window);
    // A first launch shows the welcome even when started with Windows.
    if background && welcome_seen {
        tracing::info!("started in the background: window hidden");
    } else {
        window.show()?;
    }
    slint::run_event_loop_until_quit()?;
    drop(tray);
    Ok(())
}

/// Shows the window and brings it to the front, wherever it was: hidden in
/// the notification area, minimised or behind other windows.
fn reveal(window: &AppWindow) {
    if let Err(error) = window.show() {
        tracing::warn!(%error, "cannot show the window");
        return;
    }
    window.window().set_minimized(false);
    window
        .window()
        .with_winit_window(winit::window::Window::focus_window);
}

/// Creates the notification area icon once the event loop runs. Closing the
/// window hides it while the icon exists; without an icon, closing quits, or
/// GymDiff would be left running with no way back.
fn start_tray(window: &AppWindow) -> Rc<RefCell<Option<tray::Tray>>> {
    let tray = Rc::new(RefCell::new(None));
    let weak = window.as_weak();
    let created = tray.clone();
    slint::Timer::single_shot(Duration::ZERO, move || {
        let open = move || {
            let _ = weak.upgrade_in_event_loop(|window| reveal(&window));
        };
        let quit = || {
            tracing::info!("quit from the notification area");
            let _ = slint::quit_event_loop();
        };
        match tray::Tray::create(open, quit) {
            Ok(icon) => *created.borrow_mut() = Some(icon),
            Err(error) => tracing::warn!(%error, "no notification area icon: closing quits"),
        }
    });
    let has_tray = tray.clone();
    window.window().on_close_requested(move || {
        if has_tray.borrow().is_none() {
            let _ = slint::quit_event_loop();
        }
        slint::CloseRequestResponse::HideWindow
    });
    tray
}

/// The settings folder: [`platform::data_dir`], or `GYMDIFF_DATA_DIR` when set
/// (point it at an empty folder to see the welcome screen again).
fn data_dir() -> PathBuf {
    std::env::var_os("GYMDIFF_DATA_DIR").map_or_else(platform::data_dir, PathBuf::from)
}

/// Logs to stderr and to a daily-rotated file in [`platform::log_dir`].
/// Attach that file when reporting a Windows-only bug. Also feeds the opt-in
/// reports ([`reporting`]); levels follow `docs/Workflow/Logging.md`.
fn init_logging() -> Option<tracing_appender::non_blocking::WorkerGuard> {
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
        .with(reporting::layers())
        .init();
    tracing::info!(log_dir = %log_dir.display(), "logging initialised");
    guard
}
