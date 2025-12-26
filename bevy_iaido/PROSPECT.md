Animation Playground Pivot — Prospect

Goal
Make the animation playground the primary workflow, with flexible, user-defined control sequences and clear visibility of the currently loaded frame.

Motivation
- Rapid iteration on sprite sequences without touching code.
- Consistent preview behavior: input triggers a sequence and returns to a selectable stance.
- Open-ended key bindings for new actions as assets evolve.

Proposed Features
- Frame name display: show the loaded filename alongside the index in the debug HUD.
- Open-ended key bindings:
  - Bind any key to a sequence of frame filenames.
  - Optional “press sequence” and “release sequence”.
  - Optional “hold sequence” when a key is held past a threshold.
- Starting stance:
  - Per-binding “return stance” (filename) or global stance for all bindings.
  - Return to stance after the sequence finishes (or after a timeout).
- JSON-backed controller:
  - Store all bindings, sequences, and stance info in a single controller JSON.
  - No code changes required to add new actions.

Suggested JSON Shape (Example)
{
  "default_stance": "forward-idle.png",
  "bindings": {
    "Z": {
      "press": ["up_attack_seq_1.png"],
      "release": ["up_attack_seq_2.png"]
    },
    "X": {
      "press": ["up_attack_extended_seq_1.png"],
      "release": ["up_attack_extended_seq_2.png", "up_attack_extended_seq_3.png"]
    },
    "C": {
      "press": ["block_forward.png"],
      "hold": {
        "threshold_ms": 200,
        "sequence": ["block_forward_2.png"]
      }
    }
  }
}

Interaction Model
- Cycle frames with Left/Right.
- Press a key to play its press sequence.
- Release a key to play its release sequence.
- Hold a key past threshold to play its hold sequence.
- After sequence completion, return to the stance (per-binding or global).

Implementation Notes
- Resolve filenames to indices at load time for fast playback.
- Sequence runner should allow variable-length sequences.
- If a bound filename is missing, log it and skip.

Open Questions
- Should stance be per-binding, or only a global default?
- Should hold sequences loop while held, or play once?
- Should bindings be limited to a whitelist of keys?
