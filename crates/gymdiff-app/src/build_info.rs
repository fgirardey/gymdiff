//! What identifies this build: version, build number, commit and channel,
//! stamped by `build.rs` (see `docs/Decisions/ADR-0008 Versioning and releases.md`).

/// Identity of a build of the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildInfo {
    /// `SemVer` of the workspace, bumped by release-please.
    pub version: &'static str,
    /// Commits reachable from the built commit: grows with every commit on `main`.
    pub number: u32,
    /// Full hash of the built commit, or `unknown`.
    pub commit: &'static str,
    pub channel: Channel,
    /// Target triple, e.g. `x86_64-pc-windows-msvc`.
    pub target: &'static str,
    /// `rustc --version` of the compiler.
    pub rustc: &'static str,
}

impl BuildInfo {
    /// The first 7 characters of the commit.
    pub fn short_commit(&self) -> &'static str {
        self.commit.get(..7).unwrap_or(self.commit)
    }

    /// `SemVer` with build metadata: `0.2.0+57.a1b2c3d`.
    pub fn full_version(&self) -> String {
        format!("{}+{}.{}", self.version, self.number, self.short_commit())
    }

    /// For the About screen: `0.2.0 (build 57 · a1b2c3d)`, plus the channel
    /// outside releases.
    pub fn display(&self) -> String {
        let suffix = match self.channel {
            Channel::Release => "",
            Channel::Alpha => " · alpha",
            Channel::Preview => " · preview",
            Channel::Development => " · dev",
        };
        format!(
            "{} (build {} · {}{suffix})",
            self.version,
            self.number,
            self.short_commit()
        )
    }

    /// Sentry release: `gymdiff@0.2.0+57`, which Sentry orders as `SemVer`.
    pub fn sentry_release(&self) -> String {
        format!("gymdiff@{}+{}", self.version, self.number)
    }
}

/// The running build.
pub const BUILD: BuildInfo = BuildInfo {
    version: env!("CARGO_PKG_VERSION"),
    number: parse_number(env!("GYMDIFF_BUILD_NUMBER")),
    commit: env!("GYMDIFF_BUILD_COMMIT"),
    channel: Channel::parse(env!("GYMDIFF_BUILD_CHANNEL")),
    target: env!("GYMDIFF_BUILD_TARGET"),
    rustc: env!("GYMDIFF_BUILD_RUSTC"),
};

/// Who the build is for, set by CI with `GYMDIFF_CHANNEL`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// Tagged release, published on GitHub.
    Release,
    /// CI build of `main`, published on the `alpha` pre-release; updates itself
    /// (see `docs/Product/Features/Auto-update.md`).
    Alpha,
    /// CI build of a pull request labelled `build`.
    Preview,
    /// Built on a developer machine.
    Development,
}

impl Channel {
    /// The Sentry environment of the channel.
    pub fn environment(self) -> &'static str {
        match self {
            Self::Release => "production",
            Self::Alpha => "alpha",
            Self::Preview => "preview",
            Self::Development => "development",
        }
    }

    const fn parse(name: &str) -> Self {
        match name.as_bytes() {
            b"release" => Self::Release,
            b"alpha" => Self::Alpha,
            b"preview" => Self::Preview,
            _ => Self::Development,
        }
    }
}

/// Decimal digits to a number; anything else counts as 0.
const fn parse_number(digits: &str) -> u32 {
    let bytes = digits.as_bytes();
    let mut number: u32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        let digit = bytes[i];
        if !digit.is_ascii_digit() {
            return 0;
        }
        number = number
            .saturating_mul(10)
            .saturating_add((digit - b'0') as u32);
        i += 1;
    }
    number
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: BuildInfo = BuildInfo {
        version: "0.2.0",
        number: 57,
        commit: "a1b2c3d4e5f60718293a4b5c6d7e8f9012345678",
        channel: Channel::Preview,
        target: "x86_64-pc-windows-msvc",
        rustc: "rustc 1.91.0",
    };

    #[test]
    fn formats_the_version_strings() {
        assert_eq!(SAMPLE.full_version(), "0.2.0+57.a1b2c3d");
        assert_eq!(SAMPLE.display(), "0.2.0 (build 57 · a1b2c3d · preview)");
        assert_eq!(SAMPLE.sentry_release(), "gymdiff@0.2.0+57");
        let release = BuildInfo {
            channel: Channel::Release,
            ..SAMPLE
        };
        assert_eq!(release.display(), "0.2.0 (build 57 · a1b2c3d)");
    }

    #[test]
    fn short_commit_survives_an_unknown_commit() {
        let unknown = BuildInfo {
            commit: "unknown",
            ..SAMPLE
        };
        assert_eq!(unknown.short_commit(), "unknown");
    }

    #[test]
    fn parses_build_stamps() {
        assert_eq!(parse_number("57"), 57);
        assert_eq!(parse_number(""), 0);
        assert_eq!(parse_number("5x"), 0);
        assert_eq!(Channel::parse("release"), Channel::Release);
        assert_eq!(Channel::parse("alpha"), Channel::Alpha);
        assert_eq!(Channel::parse("anything"), Channel::Development);
        assert_eq!(Channel::Release.environment(), "production");
    }
}
