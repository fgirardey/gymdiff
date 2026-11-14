//! Models for the Live Client Data API (`https://127.0.0.1:2999/liveclientdata/...`).
//!
//! Only the fields cooldown.lol uses are modelled; unknown fields are ignored so
//! that Riot can add fields without breaking deserialization.

use serde::{Deserialize, Serialize};

/// `GET /liveclientdata/gamestats`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStats {
    pub game_mode: String,
    /// Seconds since the game started.
    pub game_time: f64,
    pub map_name: String,
    pub map_number: u32,
    pub map_terrain: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gamestats_and_ignores_unknown_fields() {
        let json = r#"{
            "gameMode": "CLASSIC",
            "gameTime": 754.3,
            "mapName": "Map11",
            "mapNumber": 11,
            "mapTerrain": "Default",
            "someFutureField": true
        }"#;
        let stats: GameStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.game_mode, "CLASSIC");
        assert!((stats.game_time - 754.3).abs() < f64::EPSILON);
    }
}
