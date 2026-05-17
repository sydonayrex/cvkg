# cvkg-anim

![GPU Shader Pipeline](../docs/images/gpu_shader_pipeline.png)

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