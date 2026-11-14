//! Discovery of the League client through its `lockfile`.
//!
//! When the League client starts, it writes a one-line file named `lockfile`
//! in its installation directory:
//!
//! ```text
//! LeagueClient:<pid>:<port>:<password>:<protocol>
//! ```
//!
//! The password changes on every client restart. Requests to the LCU API use
//! HTTP Basic auth with the user `riot` and this password.
//!
//! The simulator writes a lockfile with the same format (protocol `http`), so
//! the app discovers it exactly like the real client.

use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::LolError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lockfile {
    pub process_name: String,
    pub pid: u32,
    pub port: u16,
    pub password: String,
    /// `https` for the real client, `http` for the simulator.
    pub protocol: String,
}

impl Lockfile {
    /// Base URL of the LCU API, e.g. `https://127.0.0.1:54321`.
    #[must_use]
    pub fn base_url(&self) -> String {
        format!("{}://127.0.0.1:{}", self.protocol, self.port)
    }

    /// Serializes back to the lockfile line format.
    #[must_use]
    pub fn to_line(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            self.process_name, self.pid, self.port, self.password, self.protocol
        )
    }

    /// Reads and parses a lockfile.
    pub fn read(path: &Path) -> Result<Self, LolError> {
        std::fs::read_to_string(path)?.parse()
    }

    /// Finds the lockfile: `COOLDOWN_LOCKFILE` first, then the platform defaults.
    pub fn discover() -> Result<Self, LolError> {
        let candidates = candidate_paths();
        for path in &candidates {
            if path.is_file() {
                tracing::debug!(path = %path.display(), "lockfile found");
                return Self::read(path);
            }
        }
        Err(LolError::LockfileNotFound {
            searched: candidates,
        })
    }
}

impl FromStr for Lockfile {
    type Err = LolError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let invalid = |reason: &str| LolError::LockfileInvalid(reason.to_owned());
        let parts: Vec<&str> = raw.trim().split(':').collect();
        let [process_name, pid, port, password, protocol] = parts.as_slice() else {
            return Err(invalid("expected 5 colon-separated fields"));
        };
        Ok(Self {
            process_name: (*process_name).to_owned(),
            pid: pid.parse().map_err(|_| invalid("pid is not a number"))?,
            port: port.parse().map_err(|_| invalid("port is not a number"))?,
            password: (*password).to_owned(),
            protocol: (*protocol).to_owned(),
        })
    }
}

/// Environment variable overriding the lockfile location (used by the simulator).
pub const LOCKFILE_ENV: &str = "COOLDOWN_LOCKFILE";

/// Paths where the lockfile is looked for, in order.
#[must_use]
pub fn candidate_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(custom) = std::env::var_os(LOCKFILE_ENV) {
        paths.push(PathBuf::from(custom));
    }
    paths.extend(platform_default_paths());
    paths
}

/// Typical install locations. The install directory can be changed by the
/// user, so a process-based discovery will be needed later (see roadmap).
#[cfg(target_os = "windows")]
fn platform_default_paths() -> Vec<PathBuf> {
    vec![PathBuf::from(r"C:\Riot Games\League of Legends\lockfile")]
}

#[cfg(target_os = "macos")]
fn platform_default_paths() -> Vec<PathBuf> {
    vec![PathBuf::from(
        "/Applications/League of Legends.app/Contents/LoL/lockfile",
    )]
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn platform_default_paths() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_real_looking_lockfile() {
        let lockfile: Lockfile = "LeagueClient:12345:54321:s3cr3t:https\n".parse().unwrap();
        assert_eq!(lockfile.pid, 12345);
        assert_eq!(lockfile.port, 54321);
        assert_eq!(lockfile.password, "s3cr3t");
        assert_eq!(lockfile.base_url(), "https://127.0.0.1:54321");
    }

    #[test]
    fn round_trips() {
        let line = "LeagueClient:1:2999:pw:http";
        assert_eq!(line.parse::<Lockfile>().unwrap().to_line(), line);
    }

    #[test]
    fn rejects_malformed_lockfiles() {
        assert!("LeagueClient:12345".parse::<Lockfile>().is_err());
        assert!(
            "LeagueClient:abc:54321:pw:https"
                .parse::<Lockfile>()
                .is_err()
        );
    }
}
