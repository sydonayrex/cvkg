# cvkg-anim AGENTS.md

## Purpose
Own the Sleipnir animation engine: RK4 spring integration, easing functions, transform stacks, and the MotionScale design token system.

## Ownership
- `src/lib.rs` — Animation primitives, spring physics, easing curves
- `src/sleipnir/` — RK4 spring integration with stiffness/damping/mass parameters

## Local Contracts
- Spring animations must respect `AccessibilityPreferences::reduce_motion` via `effective_duration()`.
- All animation values must be interpolation-safe (implement `Lerp` where needed).
- MotionScale tokens (Snappy/Fluid/Heavy/Bouncy) must map to concrete spring parameters.

## Verification
- Run `cargo test -p cvkg-anim`
- Run `cargo check --workspace`
