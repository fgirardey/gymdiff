//! `latest.json`: the alpha build CI published last (see
//! `.github/scripts/publish-alpha.mjs` and ADR-0010).

use serde::Deserialize;

/// A published build, as described by its manifest.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Manifest {
    /// `SemVer` of the build, e.g. `0.2.0`.
    pub version: String,
    /// Build number: commits reachable from the built commit.
    pub build: u32,
    pub commit: String,
    /// Target triple the executable was built for.
    pub target: String,
    /// Where to download the executable.
    pub url: String,
    /// Size of the executable in bytes.
    pub size: u64,
    /// SHA-256 of the executable, lower-case hex.
    pub sha256: String,
}

impl Manifest {
    /// Parses a manifest; `None` when it is not one.
    pub fn parse(json: &[u8]) -> Option<Self> {
        serde_json::from_slice(json).ok()
    }

    /// True when this build should replace `installed` on `target`: same
    /// target, higher build number.
    pub fn is_newer_than(&self, installed: u32, target: &str) -> bool {
        self.target == target && self.build > installed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON: &str = r#"{
        "version": "0.1.0",
        "build": 58,
        "commit": "a1b2c3d4e5f60718293a4b5c6d7e8f9012345678",
        "target": "x86_64-pc-windows-msvc",
        "url": "https://github.com/fgirardey/gymdiff/releases/download/alpha/gymdiff-alpha-windows-x64.exe",
        "size": 20240888,
        "sha256": "00ff",
        "published_at": "2026-11-08T12:00:00Z"
    }"#;

    #[test]
    fn parses_the_published_manifest() {
        let manifest = Manifest::parse(JSON.as_bytes()).expect("valid manifest");
        assert_eq!(manifest.build, 58);
        assert_eq!(manifest.size, 20_240_888);
        assert_eq!(manifest.target, "x86_64-pc-windows-msvc");
    }

    #[test]
    fn rejects_what_is_not_a_manifest() {
        assert!(Manifest::parse(b"<html>Not Found</html>").is_none());
        assert!(Manifest::parse(br#"{"build": 58}"#).is_none());
    }

    #[test]
    fn newer_means_same_target_and_higher_build() {
        let manifest = Manifest::parse(JSON.as_bytes()).unwrap();
        let windows = "x86_64-pc-windows-msvc";
        assert!(manifest.is_newer_than(57, windows));
        assert!(!manifest.is_newer_than(58, windows));
        assert!(!manifest.is_newer_than(60, windows));
        assert!(!manifest.is_newer_than(57, "aarch64-apple-darwin"));
    }
}
