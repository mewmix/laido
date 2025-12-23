I will generate the `GOALS.md` file capturing the project vision, constraints, and roadmap.

I will read the existing `agents/logs/goals.md` to ensure the new `GOALS.md` content aligns with any existing context before generating the output.

# BEVVY: IAIDO — Mobile Duel MVP

## Vision
**Iaido** is a high-stakes, high-speed mobile dueling game where a single stroke determines life or death. The experience centers on the tension of the draw and the precision of the strike. The player embodies the samurai spirit: calm, precise, and deadly. We are building a deterministic, rollback-ready core capable of 120 FPS performance on mobile, driven by an intuitive one-thumb portrait control scheme.

## Non-Negotiables
*   **Determinism**: The duel simulation must be bit-perfect deterministic across platforms.
*   **Monotonic Time**: Game logic advances only via fixed time steps.
*   **Zero Allocations**: No garbage collection allocations allowed during the active duel phase.
*   **Performance**: Hard target of 120 FPS on high-refresh mobile devices.
*   **Input**: One-thumb swipe interaction; portrait orientation only.
*   **Scope**: Out-of-scope features (multiplayer networking, complex meta-game) are strictly forbidden for MVP.

## System Architecture
The system is divided into the deterministic core (Rust/Bevy) and the presentation/platform layer (Unity/Mobile).

### 1. Duel Machine (`bevy_iaido`)
*   **State Machine**: Finite State Machine (FSM) managing duel phases (Sheathed, Drawing, Striking, Recovery, Result).
*   **Time**: Fixed-point math for all simulation time and physics.

### 2. Input
*   **Swipe Detector**: Analyzes touch vectors for angle, speed, and curvature.
*   **Buffer**: Inputs are stamped with a tick and buffered for the simulation.

### 3. Combat
*   **Resolver**: Determines the outcome of intersecting attacks based on timing, angle, and openings.
*   **Openings**: Dynamic target zones exposed by stance and previous actions.

### 4. AI
*   **Agent**: Utility-based AI operating on the same inputs as the player.

### 5. Logging
*   **DuelLogger**: Records every tick state and input for replay and debugging.

### 6. Integration
*   **Bevy Plugin**: Exposes the core logic as a C-compatible library for the Unity host.

## Roadmap

### Milestone 1: MVP Polish
*   **Goal**: A rock-solid, fun duel against a basic AI in a purely local environment.
*   **Focus**: Combat feel, input tightness, and deterministic validation.

### Milestone 2: Mobile Build Stubs
*   **Goal**: Running on actual Android/iOS hardware.
*   **Focus**: Build pipelines, touch input latency, screen frequency handling.

### Milestone 3: Asset Hooks
*   **Goal**: Replacing placeholders with final audio/visual assets.
*   **Focus**: Audio timing sync, animation state machine hooks.

### Milestone 4: Field Test
*   **Goal**: Performance verification.
*   **Focus**: Profiling, optimizing hot paths, zero-allocation enforcement.

## Task Ledger

### Immediate
*   [ ] **Refine State Machine**: solidify transition rules for Draw -> Strike. (Owner: Codex)
*   [ ] **Input Config**: Define constants for swipe detection thresholds. (Owner: Gemini)

### Near-Term
*   [ ] **Unity Bridge**: Generate C# bindings for the Rust FFI. (Owner: Gemini)
*   [ ] **Mobile Scaffold**: Create minimal Android Studio / XCode project exports. (Owner: Gemini)

### Later
*   [ ] **Replay System**: Implement playback from `DuelLogger` data. (Owner: Codex)
*   [ ] **UI Polish**: Minimalist HUD overlay. (Owner: Gemini)

## Risk Ledger
*   **Latency**: Input processing delay on mobile.
    *   *Mitigation*: Asynchronous input polling thread; prediction visuals.
*   **Determinism**: Floating point drift.
    *   *Mitigation*: Strict use of fixed-point math library; banning `f32`/`f64` in simulation logic.
*   **Input Ambiguity**: Misinterpreting user intent (e.g., tap vs. short swipe).
    *   *Mitigation*: Visualizers for swipe detection; adjustable sensitivity settings.

## Acceptance Criteria
*   [ ] Application runs at a stable 120 FPS on target reference device.
*   [ ] Zero GC allocations occur between "Duel Start" and "Duel End".
*   [ ] A recorded input log replays deterministically to the exact same outcome 100% of the time.
*   [ ] Input-to-action latency is perceived as instantaneous (<1 frame lag).

## Work Protocol
1.  **Directives**: Codex issues architectural directives and review constraints.
2.  **Execution**: Gemini generates scaffolding, specs, and boilerplate code.
3.  **Review**: All code changes are verified against the *Non-Negotiables*.
4.  **Commits**: Use Conventional Commits (e.g., `feat: add input buffer`, `fix: deterministic seed`).

## Appendices

### Timing Constants (Draft)
| Phase | Duration (Ticks) | Notes |
| :--- | :--- | :--- |
| `ROUND_START` | 60 | Countdown |
| `DRAW_WINDOW` | 30 | Reaction window |
| `STRIKE_FRAME`| 5 | Active hurtbox |
| `RECOVERY` | 45 | Post-attack vulnerability |

### Event List
*   `DuelStart`
*   `InputRegistered(tick, vector)`
*   `PhaseChange(from, to)`
*   `Hit(source, target, damage)`
*   `Parry`
*   `DuelEnd(winner)`

### Log Schema
```json
{
  "seed": 12345,
  "ticks": [
    { "t": 1, "inputs": [...] },
    { "t": 2, "inputs": [...] }
  ]
}
```
