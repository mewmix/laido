using UnityEngine;

namespace IAIDO.Core
{
    [CreateAssetMenu(fileName = "TimingConfig", menuName = "IAIDO/TimingConfig", order = 0)]
    public class TimingConfig : ScriptableObject
    {
        [Header("Random Delay (ms)")] public int delayMinMs = 600;
        public int delayMaxMs = 1400;

        [Header("Input Window (ms)")] public int inputWindowMs = 120;
        [Header("Clash Input Window (ms)")] public int clashInputWindowMs = 80;

        [Header("Clash Delay (ms)")] public int clashDelayMinMs = 300;
        public int clashDelayMaxMs = 600;

        [Header("Result Flash (ms)")] public int resultFlashMs = 300;
        [Header("Next Round (ms)")] public int nextRoundMs = 500;

        [Header("Swipe Detection")]
        [Tooltip("Minimum swipe distance in millimeters before direction lock.")]
        public float minSwipeDistanceMM = 7.0f; // 6–8mm
        [Tooltip("Milliseconds after motion to lock direction.")]
        public int directionLockMs = 20;
        [Tooltip("Tolerance for equal timestamps in ms.")]
        public int equalToleranceMs = 5;

        [Header("AI Profiles (ms)")]
        public int noviceMeanMs = 280;
        public float noviceWrongPct = 0.15f;
        public int skilledMeanMs = 190;
        public float skilledWrongPct = 0.05f;
        public int masterMeanMs = 140;
        public float masterWrongPct = 0.0f;
    }
}

