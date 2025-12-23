use crate::types::{Opening, RoundOutcome, SwipeDir};

pub fn correct_for(opening: Opening) -> SwipeDir {
    match opening {
        Opening::HighGuard => SwipeDir::Down,
        Opening::LowGuard => SwipeDir::Up,
        Opening::LeftGuard => SwipeDir::Right,
        Opening::RightGuard => SwipeDir::Left,
    }
}

pub fn is_correct(opening: Opening, dir: SwipeDir) -> bool {
    correct_for(opening) == dir
}

pub struct Resolution {
    pub outcome: RoundOutcome,
    pub is_clash: bool,
}

pub fn resolve(
    opening: Opening,
    player_dir: SwipeDir,
    ai_dir: SwipeDir,
    player_rt_ms: i32,
    ai_rt_ms: i32,
    equal_tolerance_ms: i32,
) -> Resolution {
    let p_correct = is_correct(opening, player_dir);
    let a_correct = is_correct(opening, ai_dir);

    if !p_correct && a_correct {
        return Resolution { outcome: RoundOutcome::AIWin, is_clash: false };
    }
    if !a_correct && p_correct {
        return Resolution { outcome: RoundOutcome::PlayerWin, is_clash: false };
    }
    if !p_correct && !a_correct {
        return Resolution { outcome: RoundOutcome::Timeout, is_clash: false };
    }

    let diff = player_rt_ms - ai_rt_ms;
    if diff > equal_tolerance_ms {
        Resolution { outcome: RoundOutcome::AIWin, is_clash: false }
    } else if diff < -equal_tolerance_ms {
        Resolution { outcome: RoundOutcome::PlayerWin, is_clash: false }
    } else {
        Resolution { outcome: RoundOutcome::Clash, is_clash: true }
    }
}

