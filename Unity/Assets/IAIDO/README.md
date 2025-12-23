# IAIDO (MVP) — Unity Framework

What’s here
- Deterministic duel state machine (best-of-3) with clash handling.
- Swipe-only input (UP/DOWN/LEFT/RIGHT) with DPI-scaled thresholds.
- Authoritative timing: monotonic clock, input timestamps, GO timestamps.
- Combat resolver (openings matrix) and AI profiles.
- JSON match logs for deterministic replay.
- Minimal view/audio hooks (stubs) for visuals and sounds.

How to use
1) Drop `Assets/IAIDO/` into a Unity mobile project (2021+).
2) Create an empty scene, add a `GameObject` named `GameRoot`.
3) Attach `GameController` to `GameRoot`.
4) Optionally hook `DuelViewController` and `AudioController` to visuals/audio.
5) Build to device; ensure target framerate set to 120 if supported.

Key constants: `TimingConfig` under `IAIDO/Scripts/Core`.
Logs: written to `Application.persistentDataPath` with filename `iaido_log_*.json` via `DuelLogger`.

Note: This is an MVP scaffold prioritizing determinism and low latency. Integrate art/audio later.

