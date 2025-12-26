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

Android Build
- Install `cargo-apk`: `cargo install cargo-apk`
- Install target: `rustup target add aarch64-linux-android`
- Set NDK root (example): `export ANDROID_NDK_ROOT=$ANDROID_HOME/ndk/26.1.10909125`
- Build APK: `cargo apk build`
- Install to device: `adb install target/debug/apk/bevy_iaido.apk`


Tools
- Grid slicer + labeler (default 2x2): `python3 tools/sprite_grid_slicer.py --input assets/atlas/*.png --out-dir assets/atlas/slices`
- Black background sheets (Gemini): `python3 tools/sprite_grid_slicer.py --input assets/atlas/Gemini_Generated_Image_*.png --out-dir assets/atlas/white_samurai --grid 2x2 --bg black`
- Generate HTML + labels: outputs `index.html` + `labels.json` in the output dir for preview/renaming.
- Apply labels to rename files: `python3 tools/sprite_grid_slicer.py --apply-labels assets/atlas/slices/labels.json`
- Rembg slicer (slower, better isolation): `python3 tools/rembg_grid_slicer.py --input assets/atlas/*.png --out-dir assets/atlas/slices --grid 2x2`

Animation Playground (dev)
- Default mode on launch; animation edit mode toggled with `D`.
- Frame cycling (edit mode): `Left/Right` arrows.
- Actions: `Z` press/release, `X` press/release, `S` press/release (double-tap S for heavy spin), `C` block (hold for second frame), `Space` dash.
- Arrow keys move the player when edit mode is off; no direction flip (always left-to-right).

Definition of Done (MVP)
- New players understand in <30s; hesitation loses; players replay; timing variance matters; log replay deterministic.
