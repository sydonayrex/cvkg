# cvkg-anim

**cvkg-anim** (Project Sleipnir) provides the physics-based animation engine for CVKG.

## 🚀 Quick Start

```rust
use cvkg_anim::{runic_emitter::RunicEmitter, SleipnirSolver, SleipnirParams};
use cvkg_core::{View, Rect};
use std::time::Duration;

// Create animation solver
let params = SleipnirParams::snappy();
let mut solver = SleipnirSolver::new(params, 1.0, 0.0);

// Animate over time
let dt = 0.016; // 60 FPS
for _ in 0..60 {
    let value = solver.tick(dt);
    println!(