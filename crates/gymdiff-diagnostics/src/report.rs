//! The report the player can preview, and that would be sent.

use serde::Serialize;

/// One captured event, already scrubbed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    /// `WARN` or `ERROR`.
    pub level: String,
    /// Module path, e.g. `gymdiff_app::watcher`.
    pub target: String,
    /// Scrubbed message and fields.
    pub message: String,
}

/// Everything that would leave the machine. Nothing else is ever added.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    pub schema: u32,
    pub app_version: String,
    pub os: String,
    /// Random id, not linked to the player; used only to answer a deletion request.
    pub install_id: String,
    pub entries: Vec<Entry>,
}

impl Report {
    /// Current report layout version.
    pub const SCHEMA: u32 = 1;

    #[must_use]
    pub fn new(install_id: &str, entries: Vec<Entry>) -> Self {
        Self {
            schema: Self::SCHEMA,
            app_version: env!("CARGO_PKG_VERSION").to_owned(),
            os: std::env::consts::OS.to_owned(),
            install_id: install_id.to_owned(),
            entries,
        }
    }

    /// Pretty JSON, exactly what would be sent. Used for the preview and export.
    #[must_use]
    pub fn to_preview_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_contains_only_the_documented_fields() {
        let report = Report::new(
            "abc",
            vec![Entry {
                level: "ERROR".into(),
                target: "t".into(),
                message: "m".into(),
            }],
        );
        let value: serde_json::Value = serde_json::from_str(&report.to_preview_json()).unwrap();
        let mut keys: Vec<_> = value.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        assert_eq!(
            keys,
            ["app_version", "entries", "install_id", "os", "schema"]
        );
    }
}
