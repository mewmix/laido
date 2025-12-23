using System;

namespace IAIDO.Core
{
    public enum Opening
    {
        HighGuard,
        LowGuard,
        LeftGuard,
        RightGuard
    }

    public enum SwipeDir
    {
        None,
        Up,
        Down,
        Left,
        Right
    }

    public enum DuelState
    {
        Reset,
        Standoff,
        RandomDelay,
        GoSignal,
        InputWindow,
        Resolution,
        ResultFlash,
        NextRound,
        MatchEnd
    }

    public enum RoundOutcome
    {
        None,
        PlayerWin,
        AIWin,
        Clash,
        EarlyPlayerLoss,
        EarlyAILoss,
        Timeout
    }
}

