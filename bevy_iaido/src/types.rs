use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Opening {
    HighGuard,
    LowGuard,
    LeftGuard,
    RightGuard,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SwipeDir {
    None,
    Up,
    Down,
    Left,
    Right,
}

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
    MatchEnd,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoundOutcome {
    None,
    PlayerWin,
    AIWin,
    Clash,
    EarlyPlayerLoss,
    EarlyAILoss,
    Timeout,
}

