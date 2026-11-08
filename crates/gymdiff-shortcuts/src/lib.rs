//! How GymDiff fits into Windows once installed, shared by the app and the
//! installer: its Start menu and Desktop shortcuts
//! (`docs/Decisions/ADR-0012 Installer window in Slint over Velopack.md`), its
//! startup entry and where Velopack put it
//! (`docs/Decisions/ADR-0013 Notification area, startup and uninstall from the app.md`).
//! On other systems nothing exists and changes fail with
//! [`ErrorKind::Unsupported`](std::io::ErrorKind::Unsupported).

mod install;
mod shortcut;
mod start_with_windows;

pub use install::{install_root, launcher, update_exe};
pub use shortcut::Shortcut;
pub use start_with_windows::{BACKGROUND_ARG, StartWithWindows};
