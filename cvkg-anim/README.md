# cvkg-anim

```mermaid
graph TD
    cvkg-core["cvkg-core"]
    cvkg-vdom["cvkg-vdom"]
    cvkg-scene["cvkg-scene"]
    cvkg-layout["cvkg-layout"]
    cvkg-render-gpu["cvkg-render-gpu"]
    cvkg-render-native["cvkg-render-native"]
    cvkg-compositor["cvkg-compositor"]
    cvkg-themes["cvkg-themes"]
    cvkg-anim["cvkg-anim"]
    cvkg-flow["cvkg-flow"]
    cvkg-runic-text["cvkg-runic-text"]
    cvkg-svg-filters["cvkg-svg-filters"]
    cvkg-svg-serialize["cvkg-svg-serialize"]
    cvkg-components["cvkg-components"]
    cvkg-macros["cvkg-macros"]
    cvkg-cli["cvkg-cli"]
    cvkg-webkit-server["cvkg-webkit-server"]
    cvkg-test["cvkg-test"]
    cvkg-physics["cvkg-physics"]
    cvkg["cvkg (umbrella)"]

    cvkg-vdom --> cvkg-core
    cvkg-vdom --> cvkg-scene
    cvkg-layout --> cvkg-core
    cvkg-layout --> cvkg-anim
    cvkg-scene --> cvkg-core

    cvkg-render-gpu --> cvkg-core
    cvkg-render-gpu --> cvkg-compositor
    cvkg-render-gpu --> cvkg-svg-filters
    cvkg-render-gpu --> cvkg-svg-serialize
    cvkg-render-gpu --> cvkg-runic-text

    cvkg-render-native --> cvkg-core
    cvkg-render-native --> cvkg-render-gpu
    cvkg-render-native --> cvkg-vdom
    cvkg-render-native --> cvkg-themes

    cvkg-compositor --> cvkg-core

    cvkg-themes --> cvkg-core
    cvkg-themes --> cvkg-anim
    cvkg-anim --> cvkg-core
    cvkg-flow --> cvkg-core
    cvkg-flow --> cvkg-scene
    cvkg-flow --> cvkg-themes

    cvkg-runic-text --> cvkg-core
    cvkg-svg-filters --> cvkg-core

    cvkg-components --> cvkg-core
    cvkg-components --> cvkg-vdom
    cvkg-components --> cvkg-layout
    cvkg-components --> cvkg-themes
    cvkg-components --> cvkg-anim
    cvkg-components --> cvkg-runic-text

    cvkg-macros --> cvkg-core
    cvkg-cli --> cvkg-core
    cvkg-cli --> cvkg-physics
    cvkg-cli --> cvkg-anim
    cvkg-cli --> cvkg-macros
    cvkg-webkit-server --> cvkg-cli
    cvkg-physics --> cvkg-core
    cvkg-physics --> cvkg-scene

    cvkg --> cvkg-core
    cvkg --> cvkg-vdom
    cvkg --> cvkg-scene
    cvkg --> cvkg-layout
    cvkg --> cvkg-themes
    cvkg --> cvkg-anim
    cvkg --> cvkg-macros
    cvkg --> cvkg-components
    cvkg --> cvkg-render-gpu
    cvkg --> cvkg-render-native
```

`cvkg-anim` provides the Sleipnir animation engine, a high-fidelity physics-based motion system for CVKG using RK4 integration for superior stability and responsiveness.

## Boundaries and Responsibilities

This crate manages the temporal evolution of UI properties. It does NOT handle layout or rendering. Its responsibilities include:
- Solving spring-mass-damper systems for organic motion.
- Implementing "Rubber Banding" for elastic boundary constraints.
- Managing complex animation sequences, parallel groups, and staggered starts.
- Providing specialized transition solvers for Bifrost and Mjolnir effects.

## Public API Overview

### Animation Solvers
- `SleipnirSolver`: Implements a 4th-order Runge-Kutta (RK4) integrator for stable, high-frequency spring simulation.
- `RubberBand`: A logarithmic resistance solver for scroll and drag interactions.

### Animation Types
- `Animation`: A comprehensive enum covering Linear, Sleipnir (Spring), Hybrid, Parallel, Sequence, and Staggered animations.
- `SleipnirParams`: Configuration for springs (Stiffness, Damping, Mass), with presets like `snappy()`, `fluid()`, and `bouncy()`.

### Controllers
- `Motion`: Handles the lifecycle of an animation (Start, Settle, Interrupt).
- `ActiveAnimation`: Tracks the runtime state and elapsed time of an executing animation.

### Spring Snap Events
- `SnapTracker`: Tracks spring animation values and emits snap events (CrossedTarget, Overshoot, Settled, DirectionChange) for haptic/audio feedback.
- `HapticBinding`: Configures which snap events trigger callbacks with intensity control.

## Usage Example

```rust
use cvkg_anim::{SleipnirSolver, SleipnirParams};

// Create a snappy spring solver
let params = SleipnirParams::snappy();
let mut solver = SleipnirSolver::new(params, 100.0, 0.0);

// Advance the simulation (typically in the render loop)
let dt = 0.016; // 60 FPS
let current_value = solver.tick(dt);

if solver.is_settled() {
    println!("Target reached!");
}
```

## Known Limitations
- The solver is currently optimized for single-variable (f32) animations; multi-dimensional vector animations are achieved via multiple solvers.
- Animation frames are tied to the renderer's `delta_time` provider.
