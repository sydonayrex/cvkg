# cvkg-anim

**cvkg-anim** (Project Sleipnir) provides physics-based animation and transition systems for CVKG using a 4th-order Runge-Kutta (RK4) solver.

## What This Crate Does

- Provides spring physics animation via `SleipnirSolver`
- Defines keyframe-based animations via `Animation` enum
- Implements rubber-banding physics for scroll/drag via `RubberBand`
- Provides motion controller for lifecycle events via `Motion`

## What This Crate Does NOT Do

- Does not provide rendering (see cvkg-render-gpu)
- Does not provide layout (see cvkg-layout)
- Does not handle user input directly

## Public API Overview

### SleipnirSolver

```rust
/// Spring parameters for physics-based animation
pub struct SleipnirParams {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

impl SleipnirParams {
    pub fn snappy() -> Self;   // Fast, responsive spring
    pub fn fluid() -> Self;    // Smooth, flowing spring
    pub fn heavy() -> Self;    // Slow, weighted spring
    pub fn bouncy() -> Self;   // Oscillating spring
}

/// RK4 physics solver for spring animations
pub struct SleipnirSolver {
    // private fields
}

impl SleipnirSolver {
    /// Create a new solver with target value and starting value
    pub fn new(params: SleipnirParams, target: f32, current: f32) -> Self;
    
    /// Set a new target value
    pub fn set_target(&mut self, target: f32);
    
    /// Advance simulation by dt seconds, returns new value
    pub fn tick(&mut self, dt: f32) -> f32;
}
```

### Animation Enum

```rust
pub enum Animation {
    /// No animation (instant transition)
    Ginnungagap,
    /// Linear animation
    Linear { duration: Duration },
    /// Organic spring animation
    Sleipnir(SleipnirParams),
    /// Keyframe path followed by spring settle
    Hybrid { keyframes: Vec<Keyframe>, settle: SleipnirParams },
    /// Multiple animations in parallel
    Parallel(Vec<Animation>),
    /// Multiple animations in sequence
    Sequence(Vec<Animation>),
    /// Staggered start for multiple animations
    Stagger { animations: Vec<Animation>, interval: Duration },
    /// Glass-aware fade transition
    BifrostFade { duration: Duration },
    /// Geometric slice transition
    MjolnirSlice { duration: Duration },
    /// Physical shatter transition
    MjolnirShatter { duration: Duration, pieces: u32, force: f32 },
}
```

### RubberBand

```rust
/// Elastic resistance for scroll/drag boundaries
pub struct RubberBand {
    pub min: f32,
    pub max: f32,
    pub constant: f32,
}

impl RubberBand {
    /// Create a new rubber band with default resistance
    pub fn new(min: f32, max: f32) -> Self;
    
    /// Calculate resisted value for input that may exceed bounds
    pub fn solve(&self, input: f32) -> f32;
}
```

### Motion Controller

```rust
/// Motion controller for lifecycle events
pub struct Motion {
    pub animation: Animation,
    pub on_start: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_settle: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_interrupt: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl Motion {
    pub fn new(animation: Animation) -> Self;
}
```

## Usage Example

```rust
use cvkg_anim::{SleipnirSolver, SleipnirParams};
use std::time::Duration;

// Create a snappy spring animation
let params = SleipnirParams::snappy();
let mut solver = SleipnirSolver::new(params, 100.0, 0.0);

// Animate over time at 60 FPS
let dt = 1.0 / 60.0;
for _ in 0..60 {
    let value = solver.tick(dt);
    // Update UI with new value
    println!("Position: {}", value);
}
```

## Known Limitations

- Solver assumes constant dt; variable framerates may produce inconsistent results
- No built-in animation sequencing; use `Sequence` or `Parallel` manually
- Keyframe interpolation uses linear easing only; no custom easing functions