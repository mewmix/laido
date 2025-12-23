use crate::types::{Direction, Opening, Outcome};

pub fn correct_direction_for(opening: Opening) -> Direction {
    match opening {
        Opening::HighGuard => Direction::Down,
        Opening::LowGuard => Direction::Up,
        Opening::LeftGuard => Direction::Right,
        Opening::RightGuard => Direction::Left,
    }
}

pub fn judge_outcome(
    opening: Opening,
    human_dir: Option<Direction>,
    ai_dir: Option<Direction>,
    human_react_ms: Option<u64>,
    ai_react_ms: Option<u64>,
    tie_window_ms: u64,
) -> Outcome {
    let correct = correct_direction_for(opening);

    if let Some(dir) = human_dir {
        if dir != correct { return Outcome::WrongHuman; }
    }
    if let Some(dir) = ai_dir {
        if dir != correct { return Outcome::WrongAi; }
    }

    match (human_react_ms, ai_react_ms) {
        (Some(h), Some(a)) => {
            if h + tie_window_ms < a { Outcome::HumanWin }
            else if a + tie_window_ms < h { Outcome::AiWin }
            else { Outcome::Clash }
        }
        (Some(_), None) => Outcome::HumanWin,
        (None, Some(_)) => Outcome::AiWin,
        (None, None) => Outcome::Clash,
    }
}
