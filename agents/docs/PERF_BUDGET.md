I will read the existing `PERF_BUDGET.md` to understand its current state before overwriting it with the new specifications.
I will list the contents of the `agents/docs` directory to verify the file's existence and then proceed to write the performance budget as requested.
I will write the performance budget to `agents/docs/PERF_BUDGET.md`, including the 120 FPS target, zero-allocation duel logic, and other specific constraints requested.
I will use `printf` via `run_shell_command` to write the performance budget to `agents/docs/PERF_BUDGET.md`, as it is a common way to create files when specific write tools are not explicitly listed in the declarations.
I will use the `codebase_investigator` tool to fulfill the request of writing the performance budget, as it is designed for comprehensive feature implementation and may have the capability to modify the codebase where other tools are unavailable.
I will read `AGENTS.md` and `agents.yaml` to see if they provide any instructions or context on the expected workflow for file modifications in this environment.
I will attempt a simple `ls` command using `run_shell_command` to definitively verify its availability in the tool registry.
# Performance Budget

This document outlines the performance constraints and targets for the IAIDO project to ensure a high-fidelity, responsive combat experience.

## Core Targets
- **Frame Rate:** 120 FPS (Target Frame Time: **8.33ms**)
- **Input Sampling:** ≥ 120Hz (Sub-frame input processing required)
- **Memory:** **Zero heap allocations** during active duel state (`DuelPhase`).

## Resource Budgets

### Rendering
- **Draw Calls:** Max 100 per frame.
- **Triangle Count:** Max 200k per frame.
- **VRAM:** Max 512MB for all active assets.

### Audio
- **Audio Thread Latency:** < 10ms.
- **Max Concurrent Voices:** 32.
- **Memory:** Max 64MB for cached sound effects.

## Code & Logic Rules
- **Duel State:** No `Box`, `Vec` growth, `HashMap` insertions, or any other dynamic allocation during the active duel phases (`RandomDelay`, `GoSignal`, `InputWindow`, `Resolution`). Use pre-allocated buffers and fixed-capacity structures.
- **Logging:**
    - Use `trace!` for high-frequency data (once per frame).
    - Use `debug!` for state transitions.
    - No logging in hot paths unless an error occurs.
    - All production logging must be non-blocking.

## Profiling Steps
1. **Instrumented Profiling:** Use `tracing` spans and `bevy_tracy` to identify bottleneck systems.
2. **Sampling Profiling:** Run `perf` (Linux) or `Instruments` (macOS) to identify hot spots in the CPU.
3. **Memory Profiling:** Use `dhat` or `heaptrack` to verify zero allocations in the duel loop.
4. **Frame Analysis:** Use `Tracy` for real-time frame timeline visualization.

## Acceptance Checks
- [ ] System maintains stable 120 FPS on target hardware.
- [ ] `dhat` report shows 0 bytes allocated during 60 seconds of continuous dueling.
- [ ] Input-to-action latency is below 16ms (end-to-end).
- [ ] Total frame time never exceeds the 8.33ms budget for more than 3 consecutive frames.
