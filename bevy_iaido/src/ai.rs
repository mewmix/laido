use crate::combat::correct_direction_for;
use crate::rng::XorShift32;
use crate::types::{Direction, Opening};

#[derive(Copy, Clone, Debug)]
pub struct AiProfile {
    pub mean_reaction_ms: u64,
    pub wrong_percent: u8, // 0..=100
}

pub const NOVICE: AiProfile = AiProfile { mean_reaction_ms: 280, wrong_percent: 15 };
pub const SKILLED: AiProfile = AiProfile { mean_reaction_ms: 190, wrong_percent: 5 };
pub const MASTER: AiProfile = AiProfile { mean_reaction_ms: 140, wrong_percent: 0 };

#[derive(Clone, Debug)]
pub struct AiPlan {
    pub reaction_ms: u64,
    pub wrong: bool,
}

impl AiPlan {
    pub fn decide_dir(&self, opening: Opening, mut rng: XorShift32) -> Direction {
        if self.wrong {
            // Pick a wrong direction uniformly among the 3 incorrect
            let correct = correct_direction_for(opening);
            let pool = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
            let mut choices: [Direction; 3] = [Direction::Up, Direction::Down, Direction::Left];
            let mut ix = 0;
            for d in pool.iter().copied() {
                if d != correct { choices[ix] = d; ix += 1; }
            }
            let k = (rng.next_u32() as usize) % 3;
            choices[k]
        } else {
            correct_direction_for(opening)
        }
    }
}

pub fn plan_for_go(profile: AiProfile, rng: &mut XorShift32) -> AiPlan {
    // Reaction time: simple jitter in ±20 ms around mean, not Gaussian to avoid deps
    let jitter = (rng.next_u32() % 41) as i64 - 20; // -20..=20
    let base = profile.mean_reaction_ms as i64 + jitter;
    let reaction_ms = base.max(60) as u64; // clamp to sensible minimum
    let wrong = (rng.next_u32() % 100) < profile.wrong_percent as u32;
    AiPlan { reaction_ms, wrong }
}
