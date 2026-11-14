//! Compiles the Slint UI and stamps the build with its identity (see
//! `docs/Decisions/ADR-0008 Versioning and releases.md`):
//! - `COOLDOWN_BUILD_NUMBER`: commits reachable from `HEAD` (monotonic on `main`),
//! - `COOLDOWN_BUILD_COMMIT`: the commit built,
//! - `COOLDOWN_BUILD_CHANNEL`: `release`, `alpha`, `preview` or `development`,
//! - `COOLDOWN_BUILD_TARGET` / `COOLDOWN_BUILD_RUSTC`: target triple and compiler.
//!
//! On Windows it also embeds the app icon (`design/brand/cooldown.ico`) and the
//! version resource shown in the file properties. Without `git` (a source tarball) the build still succeeds, with
//! build number 0 and commit `unknown`.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    slint_build::compile("ui/app-window.slint").expect("Slint UI compilation failed");

    let build = Build::detect();
    for (key, value) in [
        ("NUMBER", build.number.to_string()),
        ("COMMIT", build.commit.clone()),
        ("CHANNEL", build.channel.clone()),
        ("TARGET", env("TARGET").unwrap_or_default()),
        ("RUSTC", rustc_version()),
    ] {
        println!("cargo:rustc-env=COOLDOWN_BUILD_{key}={value}");
    }
    println!("cargo:rerun-if-env-changed=BUILD_SHA");
    println!("cargo:rerun-if-env-changed=COOLDOWN_CHANNEL");
    println!("cargo:rerun-if-changed={APP_ICON}");
    for path in git_watch_paths() {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    // Host and target both Windows: `just check-windows` from macOS has no
    // resource compiler, and a check does not link anyway.
    if cfg!(windows) && env("CARGO_CFG_TARGET_OS").as_deref() == Some("windows") {
        embed_windows_resource(&build);
    }
}

/// Windows icon of the executable: the crest at 256, 48, 32, 24 and 16 px
/// (`docs/Design/Brand.md`). Relative to the crate directory.
const APP_ICON: &str = "../../design/brand/cooldown.ico";

/// What identifies this build, besides the crate version.
struct Build {
    number: u32,
    commit: String,
    channel: String,
}

impl Build {
    fn detect() -> Self {
        let number = git(&["rev-list", "--count", "HEAD"])
            .and_then(|count| count.parse().ok())
            .unwrap_or(0);
        // CI names the PR head (`github.sha` is a temporary merge commit there).
        let commit = env("BUILD_SHA")
            .filter(|sha| !sha.is_empty())
            .or_else(|| git(&["rev-parse", "HEAD"]))
            .unwrap_or_else(|| "unknown".to_owned());
        let channel = match env("COOLDOWN_CHANNEL").as_deref() {
            Some(channel @ ("release" | "alpha" | "preview")) => channel.to_owned(),
            _ => "development".to_owned(),
        };
        Self {
            number,
            commit,
            channel,
        }
    }
}

/// The icon, `FILEVERSION` and the strings of Explorer's Details tab.
fn embed_windows_resource(build: &Build) {
    let version = env("CARGO_PKG_VERSION").unwrap_or_default();
    let short = build.commit.get(..7).unwrap_or(&build.commit);
    let product_version = format!("{version}+{}.{short}", build.number);

    let mut parts = version
        .split(['.', '-', '+'])
        .take(3)
        .map(|part| part.parse::<u64>().unwrap_or(0));
    let mut next = || parts.next().unwrap_or(0);
    let (major, minor, patch) = (next(), next(), next());
    // Each part of a Windows version is 16 bits.
    let packed =
        (major << 48) | (minor << 32) | (patch << 16) | u64::from(build.number.min(0xFFFF));

    let mut res = winresource::WindowsResource::new();
    res.set_icon(APP_ICON)
        .set_version_info(winresource::VersionInfo::FILEVERSION, packed)
        .set_version_info(winresource::VersionInfo::PRODUCTVERSION, packed)
        .set("ProductName", "cooldown.lol")
        .set("FileDescription", "cooldown.lol")
        .set("ProductVersion", &product_version)
        .set("FileVersion", &product_version)
        .set("CompanyName", "Florian Girardey")
        .set("LegalCopyright", "© Florian Girardey")
        .set("OriginalFilename", "cooldown.exe");
    if let Err(error) = res.compile() {
        println!("cargo:warning=no Windows version resource: {error}");
    }
}

/// Files whose change means a new commit or another branch. Not the index:
/// every `git add` would rerun the Slint compiler.
fn git_watch_paths() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = git(&["rev-parse", "--git-path", "HEAD"])
        .map(PathBuf::from)
        .into_iter()
        .chain(git(&["rev-parse", "--git-path", "packed-refs"]).map(PathBuf::from))
        .collect();
    if let Some(reference) = git(&["symbolic-ref", "-q", "HEAD"])
        && let Some(path) = git(&["rev-parse", "--git-path", &reference])
    {
        paths.push(PathBuf::from(path));
    }
    // Relative paths from `git rev-parse` are relative to the crate directory,
    // which is the build script's working directory.
    paths.into_iter().filter(|path| path.exists()).collect()
}

fn rustc_version() -> String {
    let rustc = env("RUSTC").unwrap_or_else(|| "rustc".to_owned());
    Command::new(rustc)
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|version| version.trim().to_owned())
        .unwrap_or_default()
}

/// Output of a `git` command, trimmed; `None` if git is missing or fails.
fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok()
}
