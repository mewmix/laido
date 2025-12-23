use crate::combat::correct_for;
use crate::types::{Opening, SwipeDir};
use rand::prelude::*;

#[derive(Copy, Clone, Debug)]
pub enum AIProfile { Novice, Skilled, Master }

pub struct AIAgent {
    rng: StdRng,
    profile: AIProfile,
    // params
    pub novice_mean_ms: i32,
    pub novice_wrong_pct: f32,
    pub skilled_mean_ms: i32,
    pub skilled_wrong_pct: f32,
    pub master_mean_ms: i32,
    pub master_wrong_pct: f32,
}

impl AIAgent {
    pub fn new(seed: u64, profile: AIProfile, params: (i32, f32, i32, f32, i32, f32)) -> Self {
        let rng = StdRng::seed_from_u64(seed);
        let (n_mean, n_wrong, s_mean, s_wrong, m_mean, m_wrong) = params;
        Self {
            rng,
            profile,
            novice_mean_ms: n_mean,
            novice_wrong_pct: n_wrong,
            skilled_mean_ms: s_mean,
            skilled_wrong_pct: s_wrong,
            master_mean_ms: m_mean,
            master_wrong_pct: m_wrong,
        }
    }

    fn params(&self) -> (i32, f32) {
        match self.profile {
            AIProfile::Novice => (self.novice_mean_ms, self.novice_wrong_pct),
            AIProfile::Skilled => (self.skilled_mean_ms, self.skilled_wrong_pct),
            AIProfile::Master => (self.master_mean_ms, self.master_wrong_pct),
        }
    }

    pub fn decide_direction(&mut self, opening: Opening) -> SwipeDir {
        let (_mean, wrong) = self.params();
        let wrong_roll: f32 = self.rng.gen();
        if wrong_roll >= wrong {
            return correct_for(opening);
        }
        // choose a wrong direction uniformly without allocations
        let correct = correct_for(opening);
        let dirs = [SwipeDir::Up, SwipeDir::Down, SwipeDir::Left, SwipeDir::Right];
        let mut idx = (self.rng.gen::<f32>() * dirs.len() as f32).floor() as usize;
        if dirs[idx] == correct { idx = (idx + 1) % dirs.len(); }
        dirs[idx]
    }

    pub fn sample_reaction_ms(&mut self) -> i32 {
        // Triangular noise around mean ±40ms
        let (mean, _wrong) = self.params();
        let tri = self.rng.gen::<f32>() - self.rng.gen::<f32>();
        let jitter = (tri * 40.0).round() as i32;
        (mean + jitter).max(0)
    }
}
