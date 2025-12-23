using IAIDO.Core;
using UnityEngine;

namespace IAIDO.Inputs
{
    public sealed class SwipeDetector : MonoBehaviour
    {
        [SerializeField] private TimingConfig config;

        private Vector2 startPos;
        private double startTime;
        private bool tracking;
        private bool committed;
        private SwipeDir committedDir = SwipeDir.None;

        // DPI scaling to translate mm to pixels
        private float MinPixels
        {
            get
            {
                float dpi = Screen.dpi;
                if (dpi <= 0) dpi = 160; // fallback
                return config != null ? (config.minSwipeDistanceMM * dpi / 25.4f) : (7f * dpi / 25.4f);
            }
        }

        public void ResetSwipe()
        {
            tracking = false;
            committed = false;
            committedDir = SwipeDir.None;
        }

        public void BeginTracking()
        {
            // Called by state machine at GO signal
            tracking = true;
            committed = false;
            committedDir = SwipeDir.None;
            startPos = GetPointerPos();
            startTime = Time.unscaledTimeAsDouble;
        }

        public SwipeDir PollDirection()
        {
            if (!tracking) return SwipeDir.None;

            // Support both touch and mouse for editor testing
            Vector2 current = GetPointerPos();
            Vector2 delta = current - startPos;

            if (!committed)
            {
                // Lock after directionLockMs regardless of distance
                double dtMs = (Time.unscaledTimeAsDouble - startTime) * 1000.0;
                if (dtMs >= (config?.directionLockMs ?? 20))
                {
                    committedDir = Classify(delta);
                    committed = true;
                }
                else
                {
                    // If distance already exceeds threshold before lock time, we can still classify
                    if (delta.magnitude >= MinPixels)
                    {
                        committedDir = Classify(delta);
                        committed = true;
                    }
                }
            }

            return committed ? committedDir : SwipeDir.None;
        }

        private static SwipeDir Classify(Vector2 delta)
        {
            if (delta == Vector2.zero) return SwipeDir.None;
            if (Mathf.Abs(delta.x) > Mathf.Abs(delta.y))
                return delta.x > 0 ? SwipeDir.Right : SwipeDir.Left;
            return delta.y > 0 ? SwipeDir.Up : SwipeDir.Down;
        }

        private static Vector2 GetPointerPos()
        {
            if (UnityEngine.Input.touchCount > 0)
                return UnityEngine.Input.GetTouch(0).position;
            return UnityEngine.Input.mousePosition;
        }
    }
}

