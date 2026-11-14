//! Removes personal data from a scenario before it is written to disk.
//!
//! Player names are replaced by `Player N` everywhere they appear (including
//! inside event texts and plain-string bodies), identifiers by stable fakes,
//! so the recording stays internally consistent.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// Keys whose string value is a player name.
const NAME_KEYS: &[&str] = &[
    "summonerName",
    "riotId",
    "riotIdGameName",
    "gameName",
    "KillerName",
    "VictimName",
    "Recipient",
];
/// Keys that are player names only next to an identifier (items also have a `displayName`).
const NAME_KEYS_NEXT_TO_ID: &[&str] = &["displayName", "internalName", "summonerInternalName"];
const ID_KEYS: &[&str] = &["puuid", "summonerId", "accountId"];
const TAG_KEYS: &[&str] = &["tagLine", "riotIdTagLine"];
/// Plain-string bodies that are a player name.
const NAME_BODY_SUFFIX: &str = "activeplayername";
const MIN_SUBSTRING_LEN: usize = 3;

/// Anonymises a whole scenario in place.
pub fn anonymize(scenario: &mut Value) {
    let mut names = BTreeSet::new();
    collect_names(scenario, &mut names);
    // Longest first, so "Name#TAG" is replaced before "Name".
    let mut names: Vec<String> = names.into_iter().collect();
    names.sort_by_key(|name| std::cmp::Reverse(name.len()));
    let replacements: Vec<(String, String)> = names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.clone(), format!("Player {}", index + 1)))
        .collect();
    let mut ids = BTreeMap::new();
    rewrite(scenario, &replacements, &mut ids);
}

fn collect_names(value: &mut Value, names: &mut BTreeSet<String>) {
    match value {
        Value::Object(object) => {
            let has_id = ID_KEYS.iter().any(|key| object.contains_key(*key))
                || NAME_KEYS.iter().any(|key| object.contains_key(*key));
            for (key, child) in object.iter() {
                let is_name = NAME_KEYS.contains(&key.as_str())
                    || (has_id && NAME_KEYS_NEXT_TO_ID.contains(&key.as_str()))
                    || (key.ends_with(NAME_BODY_SUFFIX));
                if is_name {
                    add_name(child, names);
                }
                if key == "Assisters"
                    && let Some(list) = child.as_array()
                {
                    for name in list {
                        add_name(name, names);
                    }
                }
            }
            object
                .values_mut()
                .for_each(|child| collect_names(child, names));
        }
        Value::Array(list) => list
            .iter_mut()
            .for_each(|child| collect_names(child, names)),
        _ => {}
    }
}

fn add_name(value: &Value, names: &mut BTreeSet<String>) {
    let Some(name) = value.as_str().filter(|name| !name.is_empty()) else {
        return;
    };
    names.insert(name.to_owned());
    // "Name#TAG": the bare name appears on its own elsewhere.
    if let Some((bare, _tag)) = name.split_once('#')
        && !bare.is_empty()
    {
        names.insert(bare.to_owned());
    }
}

fn rewrite(
    value: &mut Value,
    replacements: &[(String, String)],
    ids: &mut BTreeMap<String, String>,
) {
    match value {
        Value::String(text) => *text = replace_names(text, replacements),
        Value::Array(list) => list
            .iter_mut()
            .for_each(|child| rewrite(child, replacements, ids)),
        Value::Object(object) => {
            for (key, child) in object.iter_mut() {
                if ID_KEYS.contains(&key.as_str()) && !child.is_null() {
                    let original = child.to_string();
                    let next = ids.len() + 1;
                    let fake = ids
                        .entry(original)
                        .or_insert_with(|| format!("anon-{next}"));
                    *child = if child.is_number() {
                        Value::from(next as u64)
                    } else {
                        Value::from(fake.clone())
                    };
                } else if TAG_KEYS.contains(&key.as_str()) && child.is_string() {
                    *child = Value::from("0000");
                } else {
                    rewrite(child, replacements, ids);
                }
            }
        }
        _ => {}
    }
}

fn replace_names(text: &str, replacements: &[(String, String)]) -> String {
    let mut text = text.to_owned();
    for (name, fake) in replacements {
        if text == *name {
            return fake.clone();
        }
        if name.chars().count() >= MIN_SUBSTRING_LEN {
            text = text.replace(name.as_str(), fake);
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn replaces_names_everywhere_consistently() {
        let mut scenario = json!({ "steps": [{
            "live": {
                "/liveclientdata/activeplayername": "Florian#EUW",
                "/liveclientdata/eventdata": { "Events": [
                    { "KillerName": "Florian", "VictimName": "Enemy", "Assisters": ["Ally"] }
                ]},
                "/liveclientdata/playerlist": [
                    { "summonerName": "Florian#EUW", "riotIdTagLine": "EUW",
                      "items": [{ "displayName": "Doran's Blade" }] }
                ]
            },
            "lcu": { "/lol-summoner/v1/current-summoner": {
                "displayName": "Florian", "puuid": "secret-puuid", "summonerId": 42, "tagLine": "EUW" } }
        }]});
        anonymize(&mut scenario);
        let text = scenario.to_string();
        for leaked in ["Florian", "Enemy", "Ally", "secret-puuid", "EUW"] {
            assert!(!text.contains(leaked), "{leaked} leaked: {text}");
        }
        assert!(text.contains("Doran's Blade"), "items must be untouched");
        let events = &scenario["steps"][0]["live"]["/liveclientdata/eventdata"]["Events"][0];
        let me = &scenario["steps"][0]["live"]["/liveclientdata/playerlist"][0]["summonerName"];
        assert_ne!(events["KillerName"], events["VictimName"]);
        assert!(me.as_str().unwrap().starts_with("Player "));
    }
}
