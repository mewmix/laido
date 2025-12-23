using IAIDO.Core;
using UnityEngine;

namespace IAIDO.Game
{
    public sealed class GameController : MonoBehaviour
    {
        [Header("Config")]
        public TimingConfig config;

        [Header("Wiring")]
        public DuelStateMachine duel;

        private void Reset()
        {
            duel = GetComponentInChildren<DuelStateMachine>();
        }

        private void Awake()
        {
            if (Application.targetFrameRate < 120)
                Application.targetFrameRate = 120;
            QualitySettings.vSyncCount = 0;
        }
    }
}

