use crate::types::{Direction, Opening, Outcome};

pub fn correct_direction_for(opening: Opening) -> Direction {
    match opening {
        Opening::Up => Direction::Up,
        Opening::UpRight => Direction::UpRight,
        Opening::Right => Direction::Right,
        Opening::DownRight => Direction::DownRight,
        Opening::Down => Direction::Down,
        Opening::DownLeft => Direction::DownLeft,
        Opening::Left => Direction::Left,
        Opening::UpLeft => Direction::UpLeft,
        Opening::UpDown => Direction::UpDown,
        Opening::LeftRight => Direction::LeftRight,
    }
}

fn dir_to_index(d: Direction) -> usize {
    match d {
        Direction::Up => 0,
        Direction::UpRight => 1,
        Direction::Right => 2,
        Direction::DownRight => 3,
        Direction::Down => 4,
        Direction::DownLeft => 5,
        Direction::Left => 6,
        Direction::UpLeft => 7,
        Direction::UpDown => 8,
        Direction::LeftRight => 9,
    }
}

pub fn judge_outcome(
    human_opening: Opening,
    ai_opening: Opening,
    human_dir: Option<Direction>,
    ai_dir: Option<Direction>,
    human_react_ms: Option<u64>,
    ai_react_ms: Option<u64>,
    tie_window_ms: u64,
) -> Outcome {
    let correct_human = correct_direction_for(human_opening);
    let correct_ai = correct_direction_for(ai_opening);

    // 1. Validate Constraints
    // Note: If no input provided (None), it's treated as timeout/wrong if the other played?
    // Usually judge_outcome is called when both played OR window ended.
    // If one played and other didn't:
    if let Some(h) = human_dir {
        if h != correct_human { return Outcome::WrongHuman; }
    }
    if let Some(a) = ai_dir {
        if a != correct_ai { return Outcome::WrongAi; }
    }

    match (human_dir, ai_dir) {
        (Some(h), Some(a)) => {
            // Both played correctly (or we wouldn't be here).
            // RPS Resolution
            let h_idx = dir_to_index(h);
            let a_idx = dir_to_index(a);
            
            if h_idx == a_idx {
                return Outcome::Clash; // Parry
            }

            let diff = (a_idx as i32 - h_idx as i32 + 10) % 10;
            if diff == 1 || diff == 2 {
                return Outcome::HumanWin;
            }
            if diff == 8 || diff == 9 {
                return Outcome::AiWin;
            }

            // Neutral -> Speed Check
            if let (Some(ht), Some(at)) = (human_react_ms, ai_react_ms) {
                if ht + tie_window_ms < at { Outcome::HumanWin }
                else if at + tie_window_ms < ht { Outcome::AiWin }
                else { Outcome::Clash }
            } else {
                Outcome::Clash // Should not happen if dirs are Some
            }
        },
        (Some(_), None) => Outcome::HumanWin,
        (None, Some(_)) => Outcome::AiWin,
        (None, None) => Outcome::Clash,
    }
}