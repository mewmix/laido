using System.Collections;
using IAIDO.AI;
using IAIDO.Combat;
using IAIDO.Core;
using IAIDO.Inputs;
using UnityEngine;

namespace IAIDO.Game
{
    public sealed class DuelStateMachine : MonoBehaviour
    {
        [Header("Refs")] public TimingConfig config;
        public SwipeDetector swipeDetector;
        public MonoBehaviour viewController; // optional hook
        public MonoBehaviour audioController; // optional hook

        [Header("AI")] public AIProfile aiProfile = AIProfile.Skilled;

        [Header("Match")] public int bestOf = 3;

        private DuelState state = DuelState.Reset;
        private System.Random rng;
        private AIAgent ai;
        private DuelLogger logger;

        // Round
        private int roundIndex;
        private int playerWins;
        private int aiWins;
        private Opening opening;
        private double goTs;
        private bool inputLocked;
        private bool inClash;

        private void Awake()
        {
            if (config == null)
            {
                Debug.LogError("TimingConfig not assigned.");
            }
            int seed = UnityEngine.Random.Range(int.MinValue, int.MaxValue);
            rng = new System.Random(seed);
            ai = new AIAgent(config, seed ^ 0xA11CE, aiProfile);
            logger = new DuelLogger(seed);
        }

        private void Start()
        {
            StartCoroutine(Loop());
        }

        private IEnumerator Loop()
        {
            while (true)
            {
                // Match reset
                state = DuelState.Reset;
                playerWins = 0; aiWins = 0; roundIndex = 0; inClash = false;

                while (state != DuelState.MatchEnd)
                {
                    yield return RunRound();
                    // Check match end
                    int needed = (bestOf / 2) + 1;
                    if (playerWins >= needed || aiWins >= needed || roundIndex >= bestOf)
                        state = DuelState.MatchEnd;
                    else
                        state = DuelState.NextRound;

                    // Between-round delay
                    if (state == DuelState.NextRound)
                        yield return UnscaledDelay(config.nextRoundMs);
                }

                // Flush log and wait for manual restart (wired in UI)
                logger.FlushToDisk();
                yield break;
            }
        }

        private IEnumerator RunRound()
        {
            roundIndex++;
            state = DuelState.Standoff;
            swipeDetector.ResetSwipe();
            inputLocked = true;

            // Random delay
            state = DuelState.RandomDelay;
            int delay = inClash ? Rng(config.clashDelayMinMs, config.clashDelayMaxMs) : Rng(config.delayMinMs, config.delayMaxMs);
            double standoffStart = Time.unscaledTimeAsDouble;
            double endTime = standoffStart + delay / 1000.0;

            // Early input = auto-loss
            while (Time.unscaledTimeAsDouble < endTime)
            {
                var dir = ReadEarlySwipe();
                if (dir != SwipeDir.None)
                {
                    LogAndApplyOutcome(RoundOutcome.EarlyPlayerLoss, dir, SwipeDir.None, 0, 0, inClash);
                    aiWins++;
                    yield return UnscaledDelay(config.resultFlashMs);
                    inClash = false;
                    yield break;
                }
                yield return null;
            }

            // GO signal
            state = DuelState.GoSignal;
            goTs = Time.unscaledTimeAsDouble;
            swipeDetector.BeginTracking();
            // TODO: audioController?.PlayGoCue();

            // Input window
            state = DuelState.InputWindow;
            int window = inClash ? config.clashInputWindowMs : config.inputWindowMs;
            double windowEnd = goTs + window / 1000.0;

            SwipeDir playerDir = SwipeDir.None;
            double playerTs = 0;
            while (Time.unscaledTimeAsDouble < windowEnd && playerDir == SwipeDir.None)
            {
                playerDir = swipeDetector.PollDirection();
                if (playerDir != SwipeDir.None)
                {
                    playerTs = Time.unscaledTimeAsDouble;
                    break;
                }
                yield return null;
            }

            // AI decision within window
            SwipeDir aiDir = SwipeDir.None;
            double aiTs = 0;
            int aiRt = ai.SampleReactionMs();
            double aiAbsTs = goTs + aiRt / 1000.0;
            if (aiAbsTs <= windowEnd)
            {
                aiDir = ai.DecideDirection(opening);
                aiTs = aiAbsTs;
            }

            // Determine resolution
            state = DuelState.Resolution;
            bool playerProvided = playerDir != SwipeDir.None;
            bool aiProvided = aiDir != SwipeDir.None;

            // If opening not set yet, choose now for this round
            if (opening == default)
            {
                opening = (Opening)Rng(0, 4); // 0..3
            }

            RoundOutcome outcome;
            bool clash;

            if (!playerProvided && !aiProvided)
            {
                outcome = RoundOutcome.Timeout;
                clash = false;
            }
            else if (!playerProvided && aiProvided)
            {
                // Player failed to input within window
                outcome = RoundOutcome.AIWin;
                clash = false;
            }
            else if (playerProvided && !aiProvided)
            {
                // AI failed to input within window
                outcome = RoundOutcome.PlayerWin;
                clash = false;
            }
            else
            {
                int playerRtMs = (int)((playerTs - goTs) * 1000.0);
                int aiRtMs = (int)((aiTs - goTs) * 1000.0);
                var res = CombatResolver.Resolve(opening, playerDir, aiDir, playerRtMs, aiRtMs, config.equalToleranceMs);
                outcome = res.outcome;
                clash = res.isClash;
            }

            ApplyOutcome(outcome);
            LogRound(outcome, playerDir, playerTs, aiDir, aiTs);

            state = DuelState.ResultFlash;
            yield return UnscaledDelay(config.resultFlashMs);

            // Prepare next round or clash rematch
            if (clash)
            {
                inClash = true; // narrowed window, reduced delay
            }
            else
            {
                inClash = false;
                opening = default; // new opening next time
            }
        }

        private void ApplyOutcome(RoundOutcome outcome)
        {
            switch (outcome)
            {
                case RoundOutcome.PlayerWin:
                    playerWins++;
                    // TODO: viewController?.OnPlayerSlash(); audioController?.OnHit();
                    break;
                case RoundOutcome.AIWin:
                    aiWins++;
                    // TODO: viewController?.OnAISlash(); audioController?.OnHit();
                    break;
                case RoundOutcome.Clash:
                    // TODO: viewController?.OnClash(); audioController?.OnClash();
                    break;
                case RoundOutcome.EarlyPlayerLoss:
                case RoundOutcome.EarlyAILoss:
                case RoundOutcome.Timeout:
                default:
                    break;
            }
        }

        private void LogRound(RoundOutcome outcome, SwipeDir playerDir, double playerTs, SwipeDir aiDir, double aiTs)
        {
            var log = new RoundLog
            {
                roundIndex = roundIndex,
                opening = opening,
                goTimestamp = goTs,
                playerDir = playerDir,
                playerInputTs = playerTs,
                aiDir = aiDir,
                aiInputTs = aiTs,
                outcome = outcome,
                clash = outcome == RoundOutcome.Clash,
                seed = 0
            };
            logger.Append(log);
        }

        private void LogAndApplyOutcome(RoundOutcome outcome, SwipeDir playerDir, SwipeDir aiDir, double playerTs, double aiTs, bool clash)
        {
            ApplyOutcome(outcome);
            LogRound(outcome, playerDir, playerTs, aiDir, aiTs);
        }

        private SwipeDir ReadEarlySwipe()
        {
            // Any movement before GO is considered early. We just check quickly.
            // Using mouse for editor: click-drag will register.
            if (UnityEngine.Input.touchCount > 0)
            {
                var t = UnityEngine.Input.GetTouch(0);
                if (t.phase == TouchPhase.Moved)
                {
                    var delta = t.deltaPosition;
                    if (delta.sqrMagnitude > 0.01f)
                        return delta.x != 0 || delta.y != 0 ? (Mathf.Abs(delta.x) > Mathf.Abs(delta.y) ? (delta.x > 0 ? SwipeDir.Right : SwipeDir.Left) : (delta.y > 0 ? SwipeDir.Up : SwipeDir.Down)) : SwipeDir.None;
                }
            }
            else if (UnityEngine.Input.GetMouseButton(0))
            {
                if (Mathf.Abs(UnityEngine.Input.GetAxis("Mouse X")) + Mathf.Abs(UnityEngine.Input.GetAxis("Mouse Y")) > 0.001f)
                    return SwipeDir.Right; // any movement → treat as early; direction irrelevant
            }
            return SwipeDir.None;
        }

        private int Rng(int minInclusive, int maxExclusive)
        {
            return rng.Next(minInclusive, maxExclusive);
        }

        private static IEnumerator UnscaledDelay(int ms)
        {
            double end = Time.unscaledTimeAsDouble + ms / 1000.0;
            while (Time.unscaledTimeAsDouble < end) yield return null;
        }
    }
}

