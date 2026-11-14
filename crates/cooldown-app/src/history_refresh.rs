//! The History refresh: the button that reads the client's match history again, and the "Updated
//! 2 min ago" label next to it (`docs/Product/Features/History.md`, refresh).
//!
//! The button only raises a flag; the [`crate::watcher`] takes it on its next poll and runs the
//! import, then reports when it is done.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use slint::Weak;

use crate::AppWindow;

/// Shared between the UI thread (the button, the label) and the watcher. Cheap to clone.
#[derive(Clone)]
pub struct HistoryRefresh {
    window: Weak<AppWindow>,
    requested: Arc<AtomicBool>,
    /// When the last import ended, in this run of the app.
    last_import: Arc<Mutex<Option<Instant>>>,
}

impl HistoryRefresh {
    pub fn new(window: Weak<AppWindow>) -> Self {
        Self {
            window,
            requested: Arc::default(),
            last_import: Arc::default(),
        }
    }

    /// Wires the button and keeps the label current. Keep the returned timer alive.
    pub fn bind(&self, window: &AppWindow) -> slint::Timer {
        let refresh = self.clone();
        window.on_history_refresh(move || {
            tracing::info!("History refresh asked");
            refresh.requested.store(true, Ordering::Relaxed);
            refresh.set_refreshing(true);
        });

        let timer = slint::Timer::default();
        let refresh = self.clone();
        timer.start(slint::TimerMode::Repeated, LABEL_PERIOD, move || {
            if let Some(window) = refresh.window.upgrade() {
                window.set_history_updated(refresh.label().into());
            }
        });
        timer
    }

    /// Whether the player asked for a refresh since the last call.
    pub fn take_request(&self) -> bool {
        self.requested.swap(false, Ordering::Relaxed)
    }

    /// An import started.
    pub fn started(&self) {
        self.set_refreshing(true);
    }

    /// An import ended: History is up to date.
    pub fn finished(&self) {
        *self
            .last_import
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(Instant::now());
        let label = self.label();
        self.update(move |window| {
            window.set_history_updated(label.into());
            window.set_history_refreshing(false);
        });
    }

    /// The import was stopped before its end (the client went away).
    pub fn stopped(&self) {
        self.set_refreshing(false);
    }

    fn label(&self) -> String {
        let last = *self
            .last_import
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        last.map(|at| updated_label(at.elapsed()))
            .unwrap_or_default()
    }

    fn set_refreshing(&self, refreshing: bool) {
        self.update(move |window| window.set_history_refreshing(refreshing));
    }

    fn update(&self, change: impl FnOnce(AppWindow) + Send + 'static) {
        if self.window.upgrade_in_event_loop(change).is_err() {
            tracing::debug!("UI event loop is gone");
        }
    }
}

/// How often the label is recomputed: it counts in minutes.
const LABEL_PERIOD: Duration = Duration::from_secs(20);

/// "Updated just now", "Updated 2 min ago", "Updated 3 h ago", "Updated 2 d ago".
fn updated_label(elapsed: Duration) -> String {
    let minutes = elapsed.as_secs() / 60;
    match minutes {
        0 => "Updated just now".to_owned(),
        1..60 => format!("Updated {minutes} min ago"),
        60..1440 => format!("Updated {} h ago", minutes / 60),
        _ => format!("Updated {} d ago", minutes / 1440),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_label_counts_minutes_then_hours_then_days() {
        let label = |seconds| updated_label(Duration::from_secs(seconds));
        assert_eq!(label(0), "Updated just now");
        assert_eq!(label(59), "Updated just now");
        assert_eq!(label(60), "Updated 1 min ago");
        assert_eq!(label(59 * 60 + 59), "Updated 59 min ago");
        assert_eq!(label(60 * 60), "Updated 1 h ago");
        assert_eq!(label(23 * 3600 + 3599), "Updated 23 h ago");
        assert_eq!(label(24 * 3600), "Updated 1 d ago");
    }
}
