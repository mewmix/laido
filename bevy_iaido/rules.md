# Rules

This document defines the rock–paper–scissors (RPS) opening rules and control expectations
for the duel loop. It is intended as a gameplay spec (no code).

## Input
- Swipe only: UP / DOWN / LEFT / RIGHT.
- First valid swipe after GO is locked.
- Swipe before GO = instant loss.
- Minimum swipe distance is DPI-scaled (~6–8 mm).
- Direction locks after ~20 ms of motion; no correction afterward.

## Openings (Player-Based)
Each round, both players (Human and AI) are assigned an opening independently.
The opening determines the correct counter-input for that player.

## Allowed Inputs (Swipe Directions)
Single directions:
- UP
- DOWN
- LEFT
- RIGHT

Two-key combinations (diagonals and opposites are allowed):
- UP+LEFT
- UP+RIGHT
- DOWN+LEFT
- DOWN+RIGHT
- UP+DOWN
- LEFT+RIGHT

## RPS-Like Rules Template
Same input vs same input = PARRY.

Common-sense v0: arrange inputs on a 10-step wheel and compare clockwise order.
Each input beats the next two clockwise inputs, and loses to the previous two.

Wheel order:
UP -> UP+RIGHT -> RIGHT -> DOWN+RIGHT -> DOWN -> DOWN+LEFT -> LEFT -> UP+LEFT -> UP+DOWN -> LEFT+RIGHT -> (back to UP)

Beats/loses table:

- UP
  - beats: UP+RIGHT, RIGHT
  - loses to: LEFT+RIGHT, UP+DOWN
- DOWN
  - beats: DOWN+LEFT, LEFT
  - loses to: DOWN+RIGHT, RIGHT
- LEFT
  - beats: UP+LEFT, UP+DOWN
  - loses to: DOWN+LEFT, DOWN
- RIGHT
  - beats: DOWN+RIGHT, DOWN
  - loses to: UP+RIGHT, UP
- UP+LEFT
  - beats: UP+DOWN, LEFT+RIGHT
  - loses to: LEFT, DOWN+LEFT
- UP+RIGHT
  - beats: RIGHT, DOWN+RIGHT
  - loses to: UP, LEFT+RIGHT
- DOWN+LEFT
  - beats: LEFT, UP+LEFT
  - loses to: DOWN, DOWN+RIGHT
- DOWN+RIGHT
  - beats: DOWN, DOWN+LEFT
  - loses to: RIGHT, UP+RIGHT
- UP+DOWN
  - beats: LEFT+RIGHT, UP
  - loses to: UP+LEFT, LEFT
- LEFT+RIGHT
  - beats: UP, UP+RIGHT
  - loses to: UP+DOWN, UP+LEFT

## Resolution
- Wrong input = instant loss.
- Correct input -> compare reaction times; faster wins.
- Same input -> PARRY (treated as CLASH: immediate rematch with shorter delay and window).
- Equal timestamps (±5 ms) -> CLASH (immediate rematch with shorter delay and window).

## Timing (Reference)
- Hidden delay: 600–1400 ms.
- Input window: 120 ms (clash window: 80 ms).
- Result flash: ≤300 ms.
- Next round: ≤500 ms.

## Notes
- Openings are per-player; each side can receive a different opening in the same round.
- Cosmetics (characters/weapons) never alter timing, openings, or resolution.
