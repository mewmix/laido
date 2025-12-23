use core::fmt;
use core::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Direction { Up, Down, Left, Right }

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Opening { HighGuard, LowGuard, LeftGuard, RightGuard }

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Actor { Human, Ai }

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Outcome { HumanWin, AiWin, Clash, EarlyHuman, EarlyAi, WrongHuman, WrongAi }

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DuelPhase {
    Reset,
    Standoff,
    RandomDelay,
    GoSignal,
    InputWindow,
    Resolution,
    ResultFlash,
    NextRound,
    Finished,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MatchState { InProgress, HumanWon, AiWon }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RoundResult {
    pub opening: Opening,
    pub outcome: Outcome,
    pub human_reaction_ms: Option<u32>,
    pub ai_reaction_ms: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoEvent { pub ts_ms: u64 }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SwipeEvent { pub dir: Direction, pub ts_ms: u64 }

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Direction::Up => "UP",
            Direction::Down => "DOWN",
            Direction::Left => "LEFT",
            Direction::Right => "RIGHT",
        };
        write!(f, "{}", s)
    }
}

pub fn dur_ms(d: Duration) -> u64 { d.as_millis() as u64 }
