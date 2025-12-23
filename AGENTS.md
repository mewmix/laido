# BEVVY: IAIDO — Build Agent Guide (MVP)
 
Audience: Internal build agent
Target: iOS + Android (portrait)
Engine: Bevy (Rust), mobile-optimized
Objective: Prove ultra-fast directional duel gameplay is compelling, fair, and replayable.
 
 
## Non-Negotiable Goals
- Sub-second duels feel fair and decisive.
- One-thumb, swipe-only input.
- Rounds resolve in ≤300 ms after GO.
- Playable in under 30 seconds from install.
- Skill > progression.
- If a feature threatens latency, clarity, or determinism — cut it.
 
 
## Authoritative Duel Loop (Deterministic)
RESET → STANDOFF (input locked) → RANDOM_DELAY (600–1400 ms) → GO_SIGNAL → INPUT_WINDOW (120 ms) → RESOLUTION → RESULT_FLASH (≤300 ms) → NEXT ROUND (≤500 ms).
 
Timekeeping
- Use a monotonic clock (engine time since startup, unscaled).
- Timestamp input on first motion frame.
- reaction_time = input_timestamp - go_timestamp.
- Fixed timestep simulation, deterministic state transitions.
 
Determinism & Logs
- Log opening seed, GO timestamp, swipe direction + timestamp.
- Replay of logs must reproduce outcome exactly; otherwise, it’s a bug.
 
 
## Input Rules
- Input type: Swipe (UP | DOWN | LEFT | RIGHT).
- First valid swipe after GO is locked.
- Swipes before GO = auto-loss.
- No correction allowed after direction lock.
- Minimum swipe distance: device-scaled (~6–8 mm), tuned by DPI.
- Direction locked after ~20 ms of motion; ignore velocity after commit.
 
 
## Combat Logic
Openings
- HIGH_GUARD → correct swipe DOWN
- LOW_GUARD → correct swipe UP
- LEFT_GUARD → correct swipe RIGHT
- RIGHT_GUARD → correct swipe LEFT
 
Rules
- Wrong direction = instant loss.
- Correct direction → compare reaction times; faster timestamp wins.
- Equal timestamp (±5 ms) → CLASH.
 
Clash (MVP)
- Immediate rematch: reduced delay (300–600 ms), input window 80 ms.
 
 
## AI (MVP)
Profiles
- Novice: mean reaction 280 ms, 15% wrong direction.
- Skilled: mean reaction 190 ms, 5% wrong direction.
- Master: mean reaction 140 ms, 0% wrong direction.
 
Requirements
- Respect same input window.
- Lose to faster humans.
- Never cheat on direction.
 
 
## Visuals (Minimum)
- Fixed side-on, silhouettes, no camera motion.
- States: Idle, Guard, Slash (1–2 frames), Death (freeze + fade).
- Feedback: Correct+fast → clean slash + red accent; Wrong → instant hit; Clash → sparks + sound only.
 
 
## Audio (Essential)
- Ambient wind loop (low), GO cue (non-verbal), sword draw, hit, clash. No music.
 
 
## UI / UX
- Portrait, no HUD during duel.
- Round count between rounds only.
- Restart button after match end.
- Onboarding: one tutorial duel, text “Swipe when you hear the sound.” No guard explanation.
 
 
## Tech Constraints
- Performance: 120 FPS target, input sampling ≥120 Hz, zero allocations during duel.
- Architecture: deterministic duel logic, state machine only, timings via constants.
 
 
## Out of Scope (MVP)
- Online multiplayer, cosmetics, progression, feints, tells, long animations (>4 frames), monetization, accounts, stats.
 
 
## Deliverables from This Repo
- Bevy (Rust) framework under `bevy_iaido/` with:
  - Configurable timing constants.
  - Deterministic duel state machine.
  - Swipe input detector with DPI scaling.
  - Combat resolver and openings.
  - AI profiles and simulator.
  - JSON replay logger and replayer.
  - Minimal view/audio event hooks.
- Minimal usage notes in `bevy_iaido/README.md`.
 
 
## Definition of Done (MVP)
- New player understands game in <30 seconds.
- Duels feel “unfair only when I hesitated”.
- Testers voluntarily replay immediately.
- Reaction time variance clearly matters.
- No input ambiguity complaints.
- Log replay deterministically reproduces outcomes.
