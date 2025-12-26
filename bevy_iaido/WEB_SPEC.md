# Web Spec (Playable + Mobile Touch)

Goal: Run the animation playground in-browser with responsive sizing, mobile soft-touch controls, and orientation gating.

## Build + Hosting
- Target: `wasm32-unknown-unknown`
- Build tool: `trunk`
- Commands:
  - `rustup target add wasm32-unknown-unknown`
  - `cargo install trunk`
  - `trunk serve` (dev)
  - `trunk build --release` (deploy)
- Output: static assets + `index.html` + wasm bundle.

## Canvas + Layout
- Canvas fills parent container: `width: 100%`, `height: 100%`.
- Container fills viewport: `100vw` x `100vh`.
- Use a resize observer (JS) or Bevy window resizing to match canvas size.
- Keep aspect by scaling background to container width.

## Touch Controls (Mobile)
- One-finger swipe input for movement (left/right) and action gestures if needed.
- On-screen buttons mapped to:
  - `Z` (attack up)
  - `X` (attack extended)
  - `S` (stance/fast/heavy)
  - `C` (block)
  - `Space` (dash)
- Buttons should support press + hold (for block) and press + release (for combos).
- Set `touch-action: none` on container/canvas to prevent scroll.

## Orientation + Rotate Gate
- Default target: portrait.
- If landscape required, show full-screen overlay: “Rotate your device”.
- Use Screen Orientation API when available:
  - `screen.orientation.lock("portrait")` (best-effort).
- If API fails, fall back to overlay detection via `innerWidth/innerHeight`.

## Input Routing
- Use Bevy `TouchInput` for touch, `ButtonInput<KeyCode>` for desktop.
- Map touch buttons to the same input flags used by keyboard.
- Keep gameplay determinism by treating touch as virtual keypresses.

## Performance
- Disable unneeded logs in release.
- Avoid allocations per-frame in input handlers.

## Acceptance
- Loads and runs in mobile Safari/Chrome.
- Responsive layout with no background stretch.
- Touch controls are functional with visible feedback.
- Orientation lock or rotate overlay always appears when needed.
