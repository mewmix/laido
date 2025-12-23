using System.Collections.Generic;
using System.IO;
using UnityEngine;

namespace IAIDO.Core
{
    [System.Serializable]
    public struct RoundLog
    {
        public int roundIndex;
        public Opening opening;
        public double goTimestamp;
        public SwipeDir playerDir;
        public double playerInputTs;
        public SwipeDir aiDir;
        public double aiInputTs;
        public RoundOutcome outcome;
        public bool clash;
        public int seed;
    }

    [System.Serializable]
    public struct MatchLog
    {
        public int matchSeed;
        public List<RoundLog> rounds;
    }

    public sealed class DuelLogger
    {
        private readonly List<RoundLog> rounds = new List<RoundLog>(8);
        private readonly int matchSeed;

        public DuelLogger(int seed)
        {
            matchSeed = seed;
        }

        public void Append(RoundLog log)
        {
            rounds.Add(log);
        }

        public void FlushToDisk()
        {
            var m = new MatchLog { matchSeed = matchSeed, rounds = rounds };
            string json = JsonUtility.ToJson(m, true);
            string file = Path.Combine(Application.persistentDataPath, $"iaido_log_{matchSeed}.json");
            File.WriteAllText(file, json);
            Debug.Log($"IAIDO log written: {file}");
        }
    }
}

