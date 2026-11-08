//! GitHub releases of GymDiff, as listed by the REST API, and the one an
//! update channel installs (ADR-0010).

use serde::Deserialize;

/// Which releases a build updates from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateChannel {
    /// Pre-releases `v<version>-alpha.<build>`, one per build of `main`.
    Alpha,
    /// Tagged releases `v<version>` (not switched on yet).
    Stable,
}

/// A release in the API's list.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Release {
    pub tag_name: String,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
}

impl Release {
    /// Where to download this release's `manifest.json`.
    pub fn manifest_url(&self) -> Option<&str> {
        self.assets
            .iter()
            .find(|asset| asset.name == "manifest.json")
            .map(|asset| asset.browser_download_url.as_str())
    }

    /// `v0.2.0-alpha.57` → 57.
    fn alpha_build(&self) -> Option<u32> {
        let (version, build) = self.tag_name.strip_prefix('v')?.split_once("-alpha.")?;
        is_semver_core(version).then_some(())?;
        build.parse().ok()
    }

    /// `v0.2.0`, nothing after the version.
    fn is_stable_tag(&self) -> bool {
        self.tag_name.strip_prefix('v').is_some_and(is_semver_core)
    }
}

/// Parses the API's release list; `None` when it is not one.
pub fn parse(json: &[u8]) -> Option<Vec<Release>> {
    serde_json::from_slice(json).ok()
}

/// The release `channel` installs: the highest alpha build, or the newest
/// stable release (the API lists newest first). Drafts and releases without a
/// manifest are skipped.
pub fn latest(releases: &[Release], channel: UpdateChannel) -> Option<&Release> {
    let mut published = releases
        .iter()
        .filter(|release| !release.draft && release.manifest_url().is_some());
    match channel {
        UpdateChannel::Alpha => published
            .filter(|release| release.prerelease)
            .filter_map(|release| release.alpha_build().map(|build| (build, release)))
            .max_by_key(|(build, _)| *build)
            .map(|(_, release)| release),
        UpdateChannel::Stable => {
            published.find(|release| !release.prerelease && release.is_stable_tag())
        }
    }
}

/// `0.2.0`: three dot-separated numbers.
fn is_semver_core(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str, prerelease: bool, with_manifest: bool) -> Release {
        let mut assets = vec![Asset {
            name: format!("gymdiff-{tag}-windows-x64.exe"),
            browser_download_url: format!("https://example.invalid/{tag}/gymdiff.exe"),
        }];
        if with_manifest {
            assets.push(Asset {
                name: "manifest.json".to_owned(),
                browser_download_url: format!("https://example.invalid/{tag}/manifest.json"),
            });
        }
        Release {
            tag_name: tag.to_owned(),
            draft: false,
            prerelease,
            assets,
        }
    }

    #[test]
    fn alpha_picks_the_highest_build_not_the_newest() {
        // Published out of order: build 59 finished before 58.
        let releases = vec![
            release("v0.1.0-alpha.58", true, true),
            release("v0.1.0-alpha.59", true, true),
            release("v0.1.0", false, true),
            release("v0.1.0-alpha.57", true, true),
        ];
        let latest = latest(&releases, UpdateChannel::Alpha).unwrap();
        assert_eq!(latest.tag_name, "v0.1.0-alpha.59");
    }

    #[test]
    fn stable_picks_the_newest_release() {
        let releases = vec![
            release("v0.2.0-alpha.70", true, true),
            release("v0.2.0", false, true),
            release("v0.1.0", false, true),
        ];
        let latest = latest(&releases, UpdateChannel::Stable).unwrap();
        assert_eq!(latest.tag_name, "v0.2.0");
    }

    #[test]
    fn skips_drafts_releases_without_manifest_and_foreign_tags() {
        let mut draft = release("v0.1.0-alpha.60", true, true);
        draft.draft = true;
        let releases = vec![
            draft,
            release("v0.1.0-alpha.59", true, false),
            release("alpha", true, true),
            release("v0.1-alpha.61", true, true),
            release("v0.1.0-alpha.58", true, true),
        ];
        let latest = latest(&releases, UpdateChannel::Alpha).unwrap();
        assert_eq!(latest.tag_name, "v0.1.0-alpha.58");
        assert_eq!(
            latest.manifest_url(),
            Some("https://example.invalid/v0.1.0-alpha.58/manifest.json")
        );
        assert!(latest_of_nothing().is_none());
    }

    fn latest_of_nothing() -> Option<Release> {
        latest(&[release("v0.1.0", false, true)], UpdateChannel::Alpha).cloned()
    }

    #[test]
    fn parses_the_api_list() {
        let json = r#"[{
            "tag_name": "v0.1.0-alpha.57", "draft": false, "prerelease": true,
            "name": "v0.1.0-alpha.57", "html_url": "https://github.com/x",
            "assets": [{"name": "manifest.json", "size": 383,
                        "browser_download_url": "https://github.com/m.json"}]
        }]"#;
        let releases = parse(json.as_bytes()).unwrap();
        assert_eq!(
            releases[0].manifest_url(),
            Some("https://github.com/m.json")
        );
        assert!(parse(br#"{"message": "API rate limit exceeded"}"#).is_none());
    }
}
