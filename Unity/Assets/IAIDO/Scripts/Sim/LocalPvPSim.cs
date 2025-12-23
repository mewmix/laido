using IAIDO.AI;
using IAIDO.Combat;
using IAIDO.Core;
using UnityEngine;

namespace IAIDO.Sim
{
    // Simple local PvP sim: two bots w/ different reaction curves.
    public sealed class LocalPvPSim : MonoBehaviour
    {
        public TimingConfig config;
        public int rounds = 10;

        private void Start()
        {
            int seed = UnityEngine.Random.Range(int.MinValue, int.MaxValue);
            var rng = new System.Random(seed);
            var a = new AIAgent(config, seed ^ 0x1234, AIProfile.Skilled);
            var b = new AIAgent(config, seed ^ 0x5678, AIProfile.Master);
            int aWins = 0, bWins = 0, clashes = 0;

            for (int i = 0; i < rounds; i++)
            {
                var opening = (Opening)rng.Next(0, 4);
                int aRt = a.SampleReactionMs();
                int bRt = b.SampleReactionMs();
                var aDir = a.DecideDirection(opening);
                var bDir = b.DecideDirection(opening);

                var res = CombatResolver.Resolve(opening, aDir, bDir, aRt, bRt, config.equalToleranceMs);
                if (res.outcome == RoundOutcome.PlayerWin) aWins++;
                else if (res.outcome == RoundOutcome.AIWin) bWins++;
                else if (res.outcome == RoundOutcome.Clash) clashes++;
            }

            Debug.Log($"Sim done: A(Skilled)={aWins}, B(Master)={bWins}, Clashes={clashes}");
        }
    }
}

