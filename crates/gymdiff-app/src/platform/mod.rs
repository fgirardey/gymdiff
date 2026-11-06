//! OS-specific code lives here, and only here.
//!
//! Each platform module exposes the same functions. Windows is the target
//! platform; macOS is the development platform (features may be stubbed).

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod other;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub use other::*;
#[cfg(target_os = "windows")]
pub use windows::*;
