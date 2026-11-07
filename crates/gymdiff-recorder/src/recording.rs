//! Turns successive observations of the endpoints into scenario steps.

use std::collections::{BTreeMap, BTreeSet};

use gymdiff_lol::Api;
use serde_json::{Map, Value, json};

const GAME_TIME_FIELD: &str = "gameTime";
const PHASE_PATH: &str = "/lol-gameflow/v1/gameflow-phase";

/// What one endpoint answered during one poll. `None`: not available.
#[derive(Debug, Clone)]
pub struct Observation {
    pub api: Api,
    pub path: String,
    pub body: Option<Value>,
}

/// The scenario being recorded. Only changes are kept, like in a scenario file.
#[derive(Debug, Default)]
pub struct Recording {
    /// Last seen body per endpoint, without `gameTime` (it changes on every poll).
    last: BTreeMap<(Api, String), Option<Value>>,
    /// Live endpoints currently carrying a `gameTime` field.
    clocked: BTreeSet<String>,
    clock_running: bool,
    steps: Vec<Value>,
}

impl Recording {
    /// Records the observations made at `at` seconds. Returns whether a step was added.
    pub fn observe(&mut self, at: f64, observations: Vec<Observation>) -> bool {
        let mut lcu = Map::new();
        let mut live = Map::new();
        let mut notes = Vec::new();

        for Observation { api, path, body } in observations {
            if api == Api::Live {
                let has_clock = body
                    .as_ref()
                    .is_some_and(|body| body.get(GAME_TIME_FIELD).is_some());
                if has_clock {
                    self.clocked.insert(path.clone());
                } else {
                    self.clocked.remove(&path);
                }
            }
            let normalized = body.as_ref().map(without_game_time);
            let key = (api, path.clone());
            let previous = self.last.get(&key).cloned().flatten();
            if previous == normalized {
                self.last.entry(key).or_insert(normalized);
                continue;
            }
            self.last.insert(key, normalized);
            if path == PHASE_PATH
                && let Some(phase) = body.as_ref().and_then(Value::as_str)
            {
                notes.push(format!("Phase: {phase}"));
            }
            let target = if api == Api::Lcu { &mut lcu } else { &mut live };
            target.insert(path, body.unwrap_or(Value::Null));
        }

        let clock = self.clock_change();
        if lcu.is_empty() && live.is_empty() && clock.is_none() {
            return false;
        }
        let mut step = Map::new();
        step.insert("at".into(), json!((at * 10.0).round() / 10.0));
        if !notes.is_empty() {
            step.insert("note".into(), notes.join(", ").into());
        }
        if !lcu.is_empty() {
            step.insert("lcu".into(), lcu.into());
        }
        if !live.is_empty() {
            step.insert("live".into(), live.into());
        }
        if let Some(clock) = clock {
            step.insert("game_clock".into(), clock.into());
        }
        self.steps.push(step.into());
        true
    }

    #[must_use]
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// The scenario file content (not anonymised yet).
    #[must_use]
    pub fn to_scenario(&self) -> Value {
        json!({
            "description": "Recorded real session (anonymised)",
            "synthetic": false,
            "steps": self.steps,
        })
    }

    /// `"start"` / `"stop"` when the in-game clock appeared or disappeared.
    fn clock_change(&mut self) -> Option<&'static str> {
        let should_run = !self.clocked.is_empty();
        let change = match (self.clock_running, should_run) {
            (false, true) => Some("start"),
            (true, false) => Some("stop"),
            _ => None,
        };
        self.clock_running = should_run;
        change
    }
}

fn without_game_time(body: &Value) -> Value {
    let mut body = body.clone();
    if let Some(object) = body.as_object_mut() {
        object.remove(GAME_TIME_FIELD);
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lcu(path: &str, body: Option<Value>) -> Observation {
        Observation {
            api: Api::Lcu,
            path: path.into(),
            body,
        }
    }

    fn live(path: &str, body: Option<Value>) -> Observation {
        Observation {
            api: Api::Live,
            path: path.into(),
            body,
        }
    }

    #[test]
    fn keeps_only_changes() {
        let mut recording = Recording::default();
        assert!(recording.observe(0.0, vec![lcu(PHASE_PATH, Some(json!("Lobby")))]));
        assert!(!recording.observe(2.0, vec![lcu(PHASE_PATH, Some(json!("Lobby")))]));
        assert!(recording.observe(4.0, vec![lcu(PHASE_PATH, Some(json!("ChampSelect")))]));
        assert_eq!(recording.step_count(), 2);
        assert_eq!(recording.steps[1]["note"], "Phase: ChampSelect");
    }

    #[test]
    fn unavailable_from_the_start_is_not_recorded_but_disappearing_is() {
        let mut recording = Recording::default();
        assert!(!recording.observe(0.0, vec![lcu("/x", None)]));
        assert!(recording.observe(2.0, vec![lcu("/x", Some(json!({"a": 1})))]));
        assert!(recording.observe(4.0, vec![lcu("/x", None)]));
        assert_eq!(recording.steps[1]["lcu"]["/x"], Value::Null);
    }

    #[test]
    fn game_time_drives_the_clock_instead_of_creating_steps() {
        let stats = |time: f64| Some(json!({"gameMode": "CLASSIC", "gameTime": time}));
        let mut recording = Recording::default();
        assert!(recording.observe(10.0, vec![live("/s", stats(0.5))]));
        assert_eq!(recording.steps[0]["game_clock"], "start");
        assert!(!recording.observe(12.0, vec![live("/s", stats(2.5))]));
        assert!(recording.observe(14.0, vec![live("/s", None)]));
        assert_eq!(recording.steps[1]["game_clock"], "stop");
    }
}
