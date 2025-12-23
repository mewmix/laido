mod config;
mod types;
mod rng;
mod combat;
mod input;
mod state_machine;
mod ai;
mod logging;

#[cfg(feature = "bevy")]
mod plugin;

pub use config::*;
pub use types::*;
pub use rng::*;
pub use combat::*;
pub use input::*;
pub use state_machine::*;
pub use ai::*;
pub use logging::*;

#[cfg(feature = "bevy")]
pub use plugin::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn dm_at(now: u64) -> DuelMachine {
        DuelMachine::new(DuelConfig { seed: 12345, clash: true }, now)
    }

    #[test]
    fn combat_correct_direction_map() {
        assert_eq!(correct_direction_for(Opening::HighGuard), Direction::Down);
        assert_eq!(correct_direction_for(Opening::LowGuard), Direction::Up);
        assert_eq!(correct_direction_for(Opening::LeftGuard), Direction::Right);
        assert_eq!(correct_direction_for(Opening::RightGuard), Direction::Left);
    }

    #[test]
    fn wrong_direction_is_instant_loss() {
        let opening = Opening::HighGuard; // requires Down
        let out = judge_outcome(opening, Some(Direction::Up), Some(Direction::Down), Some(50), Some(80), TIE_WINDOW_MS);
        assert_eq!(out, Outcome::WrongHuman);
    }

    #[test]
    fn faster_reaction_wins() {
        let opening = Opening::LowGuard; // requires Up
        let out = judge_outcome(opening, Some(Direction::Up), Some(Direction::Up), Some(90), Some(120), TIE_WINDOW_MS);
        assert_eq!(out, Outcome::HumanWin);
    }

    #[test]
    fn tie_within_5ms_is_clash() {
        let opening = Opening::RightGuard; // requires Left
        let out = judge_outcome(opening, Some(Direction::Left), Some(Direction::Left), Some(100), Some(103), TIE_WINDOW_MS);
        assert_eq!(out, Outcome::Clash);
    }

    #[test]
    fn early_swipe_is_auto_loss() {
        let mut dm = dm_at(1000);
        dm.force_go(1200);
        // Swipe early at 1100 (< GO at 1200): human loses
        dm.on_swipe(Actor::Human, Direction::Up, 1100);
        assert!(matches!(dm.round_results.last().unwrap().outcome, Outcome::EarlyHuman));
    }

    #[test]
    fn clash_reduces_window() {
        let mut dm = dm_at(0);
        let opening = dm.current_opening();
        let correct = correct_direction_for(opening);
        dm.open_input(1000);
        dm.on_swipe(Actor::Human, correct, 1100);
        dm.on_swipe(Actor::Ai, correct, 1103);
        dm.tick(1200); // resolve
        assert!(matches!(dm.round_results.last().unwrap().outcome, Outcome::Clash));
        assert_eq!(dm.input_window_ms, CLASH_INPUT_WINDOW_MS);
    }

    #[test]
    fn swipe_detector_locks_and_threshold() {
        let cfg = SwipeConfig { dpi: 320.0 };
        let mut sd = SwipeDetector::new();
        // Move for 25ms to lock direction, accumulate to exceed 7mm threshold (~88 px at 320dpi)
        let mut dir = None;
        for _ in 0..3 { // 3 samples of ~10ms each
            dir = sd.update(&cfg, SwipeSample { dt_ms: 10, dx: 40.0, dy: 0.0 });
        }
        // Not yet beyond min distance squared (~88^2)
        assert!(dir.is_none());
        // Push over threshold
        dir = sd.update(&cfg, SwipeSample { dt_ms: 10, dx: 60.0, dy: 0.0 });
        assert_eq!(dir, Some(Direction::Right));
    }

    #[test]
    fn log_replay_is_deterministic() {
        let mut dm = dm_at(0);
        let opening = dm.current_opening();
        let correct = correct_direction_for(opening);
        let go = 1000;
        dm.open_input(go);
        dm.on_swipe(Actor::Human, correct, go + 120);
        dm.on_swipe(Actor::Ai, correct, go + 130);
        dm.tick(go + 2000);
        let log = dm.last_duel_log().expect("has log");
        replay_round(&log).expect("replay matches");
    }
}
