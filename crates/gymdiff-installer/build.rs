//! Compiles the installer window (its `.slint` files live with the app's UI, in
//! `crates/gymdiff-app/ui/`), embeds Velopack's setup and stamps the version
//! (see `docs/Decisions/ADR-0012 Installer window in Slint over Velopack.md`):
//! - `GYMDIFF_VELOPACK_SETUP`: absolute path of `GymDiff-<channel>-Setup.exe`
//!   from `vpk pack` (a build script runs in the crate's folder), embedded as
//!   the payload; unset (local builds, previews), the
//!   installer has nothing to install and says so,
//! - `GYMDIFF_INSTALLER_VERSION`: what the window shows, e.g. `0.1.0 (build 62)`;
//!   defaults to the crate version.
//!
//! On Windows it also embeds the crest icon, the version resource and a
//! manifest asking for no administrator rights.

use std::path::{Path, PathBuf};

fn main() {
    slint_build::compile("../gymdiff-app/ui/installer-window.slint")
        .expect("Slint UI compilation failed");

    let out_dir = PathBuf::from(env("OUT_DIR").expect("cargo sets OUT_DIR"));
    let payload = out_dir.join("payload.exe");
    match env("GYMDIFF_VELOPACK_SETUP").filter(|path| !path.is_empty()) {
        Some(setup) => {
            println!("cargo:rerun-if-changed={setup}");
            std::fs::copy(&setup, &payload).expect("cannot read GYMDIFF_VELOPACK_SETUP");
        }
        None => std::fs::write(&payload, []).expect("cannot write the empty payload"),
    }
    println!("cargo:rerun-if-env-changed=GYMDIFF_VELOPACK_SETUP");

    let version = env("GYMDIFF_INSTALLER_VERSION")
        .filter(|version| !version.is_empty())
        .or_else(|| env("CARGO_PKG_VERSION"))
        .unwrap_or_default();
    println!("cargo:rustc-env=GYMDIFF_INSTALLER_VERSION={version}");
    println!("cargo:rerun-if-env-changed=GYMDIFF_INSTALLER_VERSION");
    println!("cargo:rerun-if-changed={APP_ICON}");

    // Host and target both Windows: `just check-windows` from macOS does not link.
    if cfg!(windows) && env("CARGO_CFG_TARGET_OS").as_deref() == Some("windows") {
        embed_windows_resource(&version, &out_dir);
    }
}

/// The crest at 256, 48, 32, 24 and 16 px, shared with the app.
const APP_ICON: &str = "../../design/brand/gymdiff.ico";

/// Runs as the user who opened it: GymDiff installs per user.
const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>
"#;

fn embed_windows_resource(version: &str, out_dir: &Path) {
    let manifest = out_dir.join("installer.manifest");
    std::fs::write(&manifest, MANIFEST).expect("cannot write the manifest");
    let mut res = winresource::WindowsResource::new();
    res.set_icon(APP_ICON)
        .set_manifest_file(&manifest.to_string_lossy())
        .set("ProductName", "GymDiff")
        .set("FileDescription", "GymDiff Setup")
        .set("ProductVersion", version)
        .set("CompanyName", "Florian Girardey")
        .set("LegalCopyright", "© Florian Girardey")
        .set("OriginalFilename", "GymDiff-Setup.exe");
    if let Err(error) = res.compile() {
        println!("cargo:warning=no Windows resource for the installer: {error}");
    }
}

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok()
}
