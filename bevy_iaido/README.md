# IAIDO (MVP) — Bevy Framework

What’s here
- Deterministic duel loop (best-of-3) as Bevy systems.
- Swipe-only input (UP/DOWN/LEFT/RIGHT) with DPI-scaled thresholds.
- Authoritative timing with monotonic clock (unscaled), GO and input timestamps.
- Combat resolver (openings matrix) and AI profiles.
- JSON match logs for deterministic replay.
- Minimal view/audio hooks via events.

Run (desktop)
- `cargo run -p bevy_iaido`

Replay a saved log
- `cargo run -p bevy_iaido -- --replay replays/iaido_log_<seed>.json`

Mobile notes
- Add platform build toolchains (iOS/Android) and assets as needed.
- Target high refresh rate if available; Bevy’s time is monotonic by default.

Key files
- `src/main.rs` — App setup, plugins.
- `src/types.rs` — Enums and types.
- `src/config.rs` — Timing constants and device metrics.
- `src/input.rs` — Swipe detector.
- `src/combat.rs` — Openings + resolver.
- `src/ai.rs` — AI agent.
- `src/duel.rs` — State machine systems and logging integration.
- `src/logging.rs` — JSON replay logs.
- `src/events.rs` — Minimal event hooks for visuals/audio.
