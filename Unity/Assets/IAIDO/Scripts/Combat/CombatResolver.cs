using IAIDO.Core;

namespace IAIDO.Combat
{
    public static class CombatResolver
    {
        public struct Resolution
        {
            public RoundOutcome outcome;
            public bool isClash;
        }

        public static bool IsCorrect(Opening opening, SwipeDir dir)
        {
            return Openings.CorrectFor(opening) == dir;
        }

        public static Resolution Resolve(Opening opening, SwipeDir playerDir, SwipeDir aiDir, int playerRtMs, int aiRtMs, int equalToleranceMs)
        {
            // Wrong directions: instant loss for that side.
            bool pCorrect = IsCorrect(opening, playerDir);
            bool aCorrect = IsCorrect(opening, aiDir);

            if (!pCorrect && aCorrect) return new Resolution { outcome = RoundOutcome.AIWin };
            if (!aCorrect && pCorrect) return new Resolution { outcome = RoundOutcome.PlayerWin };
            if (!pCorrect && !aCorrect) return new Resolution { outcome = RoundOutcome.Timeout }; // both wrong; treat as no valid input

            // Both correct: compare reaction times
            int diff = playerRtMs - aiRtMs;
            if (diff > equalToleranceMs) return new Resolution { outcome = RoundOutcome.AIWin };
            if (diff < -equalToleranceMs) return new Resolution { outcome = RoundOutcome.PlayerWin };
            return new Resolution { outcome = RoundOutcome.Clash, isClash = true };
        }
    }
}

