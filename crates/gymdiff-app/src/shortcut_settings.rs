//! Settings → Shortcuts: shows and changes GymDiff's Start menu and Desktop
//! shortcuts on builds installed by the installer
//! (`docs/Product/Features/Installation.md`).

use gymdiff_shortcuts::Shortcut;
use slint::ComponentHandle;

use crate::AppWindow;

/// Shows the section when GymDiff is installed and keeps its switches in step
/// with the shortcuts on disk.
pub fn bind(window: &AppWindow, installed: bool) {
    window.set_shortcuts_visible(installed);
    if !installed {
        return;
    }
    refresh(window);
    let weak = window.as_weak();
    window.on_shortcut_start_menu_changed(move |wanted| {
        change(&weak, Shortcut::StartMenu, wanted);
    });
    let weak = window.as_weak();
    window.on_shortcut_desktop_changed(move |wanted| change(&weak, Shortcut::Desktop, wanted));
}

/// Creates or removes the shortcut, then shows what is really on disk (a
/// failed change flips the switch back).
fn change(window: &slint::Weak<AppWindow>, shortcut: Shortcut, wanted: bool) {
    match shortcut.set(wanted) {
        Ok(()) => tracing::info!(?shortcut, wanted, "shortcut changed"),
        Err(error) => tracing::warn!(%error, ?shortcut, wanted, "cannot change the shortcut"),
    }
    if let Some(window) = window.upgrade() {
        refresh(&window);
    }
}

fn refresh(window: &AppWindow) {
    window.set_shortcut_start_menu(Shortcut::StartMenu.exists());
    window.set_shortcut_desktop(Shortcut::Desktop.exists());
}
