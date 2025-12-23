use crate::types::{Opening, RoundOutcome, SwipeDir};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundLog {
    pub round_index: u32,
    pub opening: Opening,
    pub go_timestamp: f64,
    pub player_dir: SwipeDir,
    pub player_input_ts: f64,
    pub ai_dir: SwipeDir,
    pub ai_input_ts: f64,
    pub outcome: RoundOutcome,
    pub clash: bool,
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchLog {
    pub match_seed: u64,
    pub rounds: Vec<RoundLog>,
}

#[derive(Default)]
pub struct DuelLogger {
    match_seed: u64,
    rounds: Vec<RoundLog>,
}

impl DuelLogger {
    pub fn new(seed: u64) -> Self { Self { match_seed: seed, rounds: Vec::with_capacity(8) } }
    pub fn append(&mut self, log: RoundLog) { self.rounds.push(log); }
    pub fn flush_to_disk(&self) {
        let m = MatchLog { match_seed: self.match_seed, rounds: self.rounds.clone() };
        let json = serde_json::to_string_pretty(&m).unwrap_or_else(|_| "{}".into());
        let mut path = PathBuf::from("replays");
        let _ = fs::create_dir_all(&path);
        path.push(format!("iaido_log_{}.json", self.match_seed));
        let _ = fs::write(&path, json);
        println!("IAIDO log written: {}", path.display());
    }
}

// Simple replayer: verifies outcomes deterministically using logged data
pub fn replay_match(log: &MatchLog) -> bool {
    use crate::combat::resolve;
    let mut ok = true;
    for r in &log.rounds {
        let prt = ((r.player_input_ts - r.go_timestamp) * 1000.0) as i32;
        let art = ((r.ai_input_ts - r.go_timestamp) * 1000.0) as i32;
        let res = resolve(r.opening, r.player_dir, r.ai_dir, prt, art, 5);
        if res.outcome != r.outcome || res.is_clash != r.clash {
            eprintln!(
                "Replay mismatch in round {}: expected {:?}/{}, got {:?}/{}",
                r.round_index, r.outcome, r.clash, res.outcome, res.is_clash
            );
            ok = false;
        }
    }
    ok
}

pub fn load_log(path: &str) -> Option<MatchLog> {
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str::<MatchLog>(&data).ok()
}
