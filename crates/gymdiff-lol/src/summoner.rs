//! Model for the active summoner: `GET /lol-summoner/v1/current-summoner` (LCU).

use serde::Deserialize;

/// The summoner logged into the client.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summoner {
    /// Stable, global player id: the key History stores games under.
    pub puuid: String,
    #[serde(default)]
    pub game_name: String,
    #[serde(default)]
    pub tag_line: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_summoner_and_ignores_unknown_fields() {
        let json = r#"{
            "accountId": 1,
            "puuid": "a1b2-c3",
            "gameName": "Florian",
            "tagLine": "EUW",
            "summonerLevel": 312,
            "someFutureField": {"nested": true}
        }"#;
        let summoner: Summoner = serde_json::from_str(json).unwrap();
        assert_eq!(summoner.puuid, "a1b2-c3");
        assert_eq!(summoner.game_name, "Florian");
        assert_eq!(summoner.tag_line, "EUW");
    }
}
