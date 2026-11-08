//! Removes personal data from free text.

use std::sync::LazyLock;

use regex::Regex;

/// Replaces anything that could identify the player or their machine.
///
/// Covers home folders and user names in paths, Riot IDs (`Name#TAG`), UUIDs
/// (`puuid`), IPv4 addresses, e-mail addresses and `key=value` secrets
/// (tokens, passwords). The game payloads are never logged on purpose; this is
/// the safety net.
#[must_use]
pub fn scrub(text: &str) -> String {
    let mut out = text.to_owned();
    for (re, replacement) in RULES.iter() {
        out = re.replace_all(&out, *replacement).into_owned();
    }
    out
}

static RULES: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    let rule = |pattern: &str, replacement| {
        (
            Regex::new(pattern).expect("static scrub pattern is valid"),
            replacement,
        )
    };
    vec![
        // Secrets first, so nothing below sees them.
        rule(r"(?i)\bBearer\s+[A-Za-z0-9._~+/=-]+", "Bearer <redacted>"),
        rule(
            r#"(?i)\b(password|passwd|token|auth|authorization|secret|api[_-]?key)(["']?\s*[:=]\s*["']?)[^\s"',;&]+"#,
            "$1$2<redacted>",
        ),
        // Paths: Windows (`C:\Users\name\...`), macOS and Linux homes.
        rule(r"(?i)([A-Z]:[\\/]+Users[\\/]+)[^\\/\s]+", "${1}<user>"),
        rule(r"(/Users/|/home/)[^/\s]+", "${1}<user>"),
        rule(r"(?i)(%USERPROFILE%[\\/]|~[\\/])", "<home>/"),
        rule(r"[\w.+-]+@[\w-]+(\.[\w-]+)+", "<email>"),
        rule(
            r"(?i)\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b",
            "<id>",
        ),
        // Riot ID: game name (may contain spaces) + '#' + 3-5 alphanumerics.
        rule(r"[\p{L}\p{N}_. ]{3,16}#[\p{L}\p{N}]{3,5}\b", "<riot-id>"),
        rule(r"\b\d{1,3}(\.\d{1,3}){3}\b", "<ip>"),
    ]
});

#[cfg(test)]
mod tests {
    use super::scrub;

    #[test]
    fn scrubs_windows_and_unix_user_paths() {
        assert_eq!(
            scrub(r"cannot open C:\Users\Florian\AppData\Local\GymDiff\x.log"),
            r"cannot open C:\Users\<user>\AppData\Local\GymDiff\x.log"
        );
        assert_eq!(
            scrub("at /Users/florian/Developer/gymdiff/src/main.rs:3"),
            "at /Users/<user>/Developer/gymdiff/src/main.rs:3"
        );
    }

    #[test]
    fn scrubs_riot_ids_puuids_emails_and_ips() {
        let text = "player Faker#KR1 puuid 123e4567-e89b-12d3-a456-426614174000 \
                    mail a.b@example.com from 192.168.1.20";
        let out = scrub(text);
        for leaked in ["Faker", "KR1", "123e4567", "example.com", "192.168"] {
            assert!(!out.contains(leaked), "{leaked} leaked: {out}");
        }
    }

    #[test]
    fn scrubs_secrets_in_key_value_pairs() {
        let out = scrub("lockfile password=hunter2 and Authorization: Bearer abc.def-ghi");
        assert!(
            !out.contains("hunter2") && !out.contains("abc.def"),
            "{out}"
        );
    }

    #[test]
    fn leaves_harmless_text_alone() {
        let text = "phase changed to ChampSelect after 3 consecutive errors";
        assert_eq!(scrub(text), text);
    }
}
