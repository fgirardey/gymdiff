//! Settings → Startup: shows and changes whether Windows starts cooldown.lol, on
//! installed builds (`docs/Product/Features/Running in the background.md`).

use cooldown_shortcuts::StartWithWindows;
use slint::ComponentHandle;

use crate::AppWindow;

/// Shows the section when cooldown.lol is installed and keeps its switch in step
/// with the registry (Task Manager can change it too).
pub fn bind(window: &AppWindow, installed: bool) {
    window.set_startup_visible(installed);
    if !installed {
        return;
    }
    window.set_start_with_windows(StartWithWindows::is_enabled());
    let weak = window.as_weak();
    window.on_start_with_windows_changed(move |wanted| {
        match StartWithWindows::set(wanted) {
            Ok(()) => tracing::info!(wanted, "start with Windows changed"),
            Err(error) => tracing::warn!(%error, wanted, "cannot change start with Windows"),
        }
        // A failed change flips the switch back.
        if let Some(window) = weak.upgrade() {
            window.set_start_with_windows(StartWithWindows::is_enabled());
        }
    });
}
