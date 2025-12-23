using IAIDO.Core;

namespace IAIDO.Combat
{
    public static class Openings
    {
        public static SwipeDir CorrectFor(Opening opening)
        {
            switch (opening)
            {
                case Opening.HighGuard: return SwipeDir.Down;
                case Opening.LowGuard: return SwipeDir.Up;
                case Opening.LeftGuard: return SwipeDir.Right;
                case Opening.RightGuard: return SwipeDir.Left;
                default: return SwipeDir.None;
            }
        }
    }
}

