using UnityEngine;

namespace IAIDO.Core
{
    public sealed class DeterministicClock : MonoBehaviour
    {
        // Use unscaled time to avoid timescale effects. Unity provides double precision.
        public static double Now => Time.unscaledTimeAsDouble;
    }
}

