//! Scenario format and timeline evaluation (pure, unit-tested).
//!
//! A scenario is a list of timed steps. Each step sets (or removes, with
//! `null`) the JSON response served for an endpoint path. The state at time
//! `t` is the result of applying, in order, every step with `at <= t`.
//!
//! See `docs/Workflow/Simulator.md` for the full format.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Context;
use serde::Deserialize;
use serde_json::Value;

/// Field patched with the running game clock in Live Client responses.
const GAME_TIME_FIELD: &str = "gameTime";

#[derive(Debug, Clone, Deserialize)]
pub struct Scenario {
    pub description: String,
    /// `true` for hand-written scenarios, `false` for recordings of real sessions.
    pub synthetic: bool,
    pub steps: Vec<Step>,
}

impl Scenario {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("reading scenario {}", path.display()))?;
        let mut scenario: Self = serde_json::from_str(&raw)
            .with_context(|| format!("parsing scenario {}", path.display()))?;
        scenario.steps.sort_by(|a, b| a.at.total_cmp(&b.at));
        Ok(scenario)
    }

    /// Time of the last step.
    #[must_use]
    pub fn duration(&self) -> f64 {
        self.steps.last().map_or(0.0, |step| step.at)
    }

    /// Endpoints and clock state at scenario time `t` (seconds).
    #[must_use]
    pub fn snapshot_at(&self, t: f64) -> Snapshot {
        let mut snapshot = Snapshot::default();
        for step in self.steps.iter().take_while(|step| step.at <= t) {
            apply(&mut snapshot.lcu, &step.lcu);
            apply(&mut snapshot.live, &step.live);
            match step.game_clock {
                Some(GameClock::Start) => snapshot.clock_started_at = Some(step.at),
                Some(GameClock::Stop) => snapshot.clock_started_at = None,
                None => {}
            }
        }
        if let Some(started_at) = snapshot.clock_started_at {
            let game_time = (t - started_at).max(0.0);
            for body in snapshot.live.values_mut() {
                if let Some(field) = body.get_mut(GAME_TIME_FIELD) {
                    *field = Value::from(game_time);
                }
            }
        }
        snapshot
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Step {
    /// Seconds since the start of the scenario.
    pub at: f64,
    /// Optional note shown in the simulator logs.
    #[serde(default)]
    pub note: Option<String>,
    /// LCU endpoints (path -> JSON body). `null` removes the endpoint.
    #[serde(default)]
    pub lcu: BTreeMap<String, Value>,
    /// Live Client Data endpoints (path -> JSON body). `null` removes the endpoint.
    #[serde(default)]
    pub live: BTreeMap<String, Value>,
    /// Start (`"start"`) or stop (`"stop"`) the in-game clock.
    #[serde(default)]
    pub game_clock: Option<GameClock>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameClock {
    Start,
    Stop,
}

/// What the simulator serves at a given instant.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Snapshot {
    pub lcu: BTreeMap<String, Value>,
    pub live: BTreeMap<String, Value>,
    /// Scenario time at which the game clock started, if running.
    pub clock_started_at: Option<f64>,
}

fn apply(target: &mut BTreeMap<String, Value>, changes: &BTreeMap<String, Value>) {
    for (path, body) in changes {
        if body.is_null() {
            target.remove(path);
        } else {
            target.insert(path.clone(), body.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn scenario() -> Scenario {
        serde_json::from_value(json!({
            "description": "test",
            "synthetic": true,
            "steps": [
                { "at": 0, "lcu": { "/phase": "Lobby" } },
                { "at": 10, "lcu": { "/phase": "InProgress" },
                  "live": { "/stats": { "gameTime": 0.0, "gameMode": "CLASSIC" } },
                  "game_clock": "start" },
                { "at": 100, "lcu": { "/phase": "EndOfGame" },
                  "live": { "/stats": null }, "game_clock": "stop" }
            ]
        }))
        .unwrap()
    }

    #[test]
    fn applies_steps_up_to_t() {
        let s = scenario();
        assert_eq!(s.snapshot_at(5.0).lcu["/phase"], "Lobby");
        assert_eq!(s.snapshot_at(10.0).lcu["/phase"], "InProgress");
        assert!(s.snapshot_at(5.0).live.is_empty());
    }

    #[test]
    fn patches_the_game_clock() {
        let snapshot = scenario().snapshot_at(70.0);
        assert_eq!(snapshot.live["/stats"]["gameTime"], 60.0);
        assert_eq!(snapshot.live["/stats"]["gameMode"], "CLASSIC");
    }

    #[test]
    fn null_removes_endpoints_and_stop_clears_clock() {
        let snapshot = scenario().snapshot_at(150.0);
        assert!(!snapshot.live.contains_key("/stats"));
        assert_eq!(snapshot.clock_started_at, None);
    }

    #[test]
    fn bundled_scenarios_are_valid() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/scenarios");
        let mut count = 0;
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path().join("scenario.json");
            if path.is_file() {
                let scenario = Scenario::load(&path).unwrap();
                assert!(
                    !scenario.steps.is_empty(),
                    "{} has no steps",
                    path.display()
                );
                count += 1;
            }
        }
        assert!(count > 0, "no bundled scenario found");
    }
}
