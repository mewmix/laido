use serde::{Deserialize, Serialize};
use std::fs;

use crate::types::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DuelLog {
    pub seed: u32,
    pub opening: Opening,
    pub go: GoEvent,
    pub human: Option<SwipeEvent>,
    pub ai: Option<SwipeEvent>,
    pub outcome: Outcome,
}

impl DuelLog {
    pub fn to_json(&self) -> String { serde_json::to_string(self).unwrap() }
    pub fn from_json(s: &str) -> serde_json::Result<Self> { serde_json::from_str(s) }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchLog {
    pub seed: u32,
    pub rounds: Vec<DuelLog>,
}

impl MatchLog {
    pub fn to_json(&self) -> String { serde_json::to_string(self).unwrap() }
    pub fn from_json(s: &str) -> serde_json::Result<Self> { serde_json::from_str(s) }
}

#[derive(Debug)]
pub enum ReplayError { OutcomeMismatch, OpeningMismatch }

pub fn replay_round(log: &DuelLog) -> Result<(), ReplayError> {
    use crate::state_machine::{DuelConfig, DuelMachine};
    let mut dm = DuelMachine::new(DuelConfig { seed: log.seed, clash: true }, log.go.ts_ms);
    // Force opening identity to match
    if dm.opening != log.opening { return Err(ReplayError::OpeningMismatch); }
    // Force into input window at GO
    dm.open_input(log.go.ts_ms);
    // Feed inputs
    if let Some(h) = &log.human { dm.on_swipe(Actor::Human, h.dir, h.ts_ms); }
    if let Some(a) = &log.ai { dm.on_swipe(Actor::Ai, a.dir, a.ts_ms); }
    // Resolve immediately after window
    dm.tick(log.go.ts_ms + 1000);
    let last = dm.round_results.last().expect("round result exists");
    if last.outcome != log.outcome { return Err(ReplayError::OutcomeMismatch); }
    Ok(())
}

pub fn load_log(path: &str) -> Option<MatchLog> {
    if let Ok(content) = fs::read_to_string(path) {
        MatchLog::from_json(&content).ok()
    } else {
        None
    }
}

pub fn replay_match(log: &MatchLog) -> bool {
    for round in &log.rounds {
        if replay_round(round).is_err() {
            return false;
        }
    }
    true
}
