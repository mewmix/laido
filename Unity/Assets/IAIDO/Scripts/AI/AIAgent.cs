using IAIDO.Core;
using IAIDO.Combat;
using UnityEngine;

namespace IAIDO.AI
{
    public enum AIProfile { Novice, Skilled, Master }

    public sealed class AIAgent
    {
        private readonly TimingConfig config;
        private readonly System.Random rng;
        public AIProfile Profile { get; }

        public AIAgent(TimingConfig config, int seed, AIProfile profile)
        {
            this.config = config;
            this.rng = new System.Random(seed);
            this.Profile = profile;
        }

        private (int meanMs, float wrongPct) Params()
        {
            return Profile switch
            {
                AIProfile.Novice => (config.noviceMeanMs, config.noviceWrongPct),
                AIProfile.Skilled => (config.skilledMeanMs, config.skilledWrongPct),
                AIProfile.Master => (config.masterMeanMs, config.masterWrongPct),
                _ => (config.skilledMeanMs, config.skilledWrongPct)
            };
        }

        public SwipeDir DecideDirection(Opening opening)
        {
            var p = Params();
            bool wrong = rng.NextDouble() < p.wrongPct;
            if (!wrong) return Openings.CorrectFor(opening);

            // Pick a wrong direction uniformly among the other three
            SwipeDir correct = Openings.CorrectFor(opening);
            SwipeDir[] all = { SwipeDir.Up, SwipeDir.Down, SwipeDir.Left, SwipeDir.Right };
            int idx = rng.Next(0, 3);
            int count = 0;
            foreach (var d in all)
            {
                if (d == correct) continue;
                if (count == idx) return d;
                count++;
            }
            return SwipeDir.Left;
        }

        public int SampleReactionMs()
        {
            // Simple bounded noise around mean using triangular distribution
            var p = Params();
            double u = rng.NextDouble();
            // Triangle (-1..1)
            double tri = (rng.NextDouble() - rng.NextDouble());
            int jitter = (int)(tri * 40.0); // ±40ms jitter
            int rt = Mathf.Max(0, p.meanMs + jitter);
            return rt;
        }
    }
}

