BEVVY: IAIDO — MVP Core (Bevy)

Purpose: deterministic, ultra-fast directional duel core for mobile.

- One-thumb swipe input (UP/DOWN/LEFT/RIGHT)
- Hidden delay (600–1400 ms), GO, 120 ms input window
- Wrong direction = instant loss; correct compares reaction time; ±5 ms = CLASH
- Clash rematch: 300–600 ms delay, 80 ms window
- Best of 3. No HUD during duel. Minimal hooks for audio/visual.

Modules
- config: Tunable constants and time helpers.
- types: Directions, openings, outcomes, phases, events.
- rng: XorShift32 deterministic RNG.
- input: Swipe detector with DPI scaling and 20 ms direction lock.
- combat: Mapping from opening→truth and outcome judge.
- state_machine: Authoritative duel state machine and match rules.
- ai: Novice/Skilled/Master profiles; reaction planner.
- logging: JSON round/match logs and deterministic replayer.
- plugin (feature "bevy"): Minimal Bevy plugin wiring input, AI, and events.

Bevy Usage (desktop dev)
- Insert IaidoSettings { seed, dpi } and add IaidoPlugin.
- Subscribe to GoCue, SlashCue { actor }, ClashCue for feedback hooks.

Determinism & Logs
- DuelMachine uses monotonic time in ms and fixed transitions.
- DuelLog and MatchLog serialize to JSON; replay_round verifies outcome.
- Opening seed and GO timestamp are recorded to reproduce exactly.

Input
- SwipeDetector locks direction after ~20 ms of motion.
- Minimum distance is scaled by DPI; default 7 mm.

AI
- Profiles: Novice (280 ms, 15%), Skilled (190 ms, 5%), Master (140 ms, 0%).
- AI plans reaction on GO; never inputs before GO; respects input window.

Mobile Notes
- Portrait; target 120 FPS. Keep allocations out of the duel path.
- Audio/visual assets not included; hook to plugin events.

Build
- Requires Rust and Bevy 0.14.
- Example: cargo run --example iaido (desktop). For mobile, integrate with your runner.

Tools
- Sprite sheet labeler: `python3 tools/sprite_explorer.py assets/atlas/swordsman_laido_atlas.png --tile-w 64 --tile-h 64`
- Optional: `--labels labels.json --out-dir sprite_exports` for saving labels and exporting tiles.

Definition of Done (MVP)
- New players understand in <30s; hesitation loses; players replay; timing variance matters; log replay deterministic.
