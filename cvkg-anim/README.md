# cvkg-anim

**cvkg-anim** (Project Sleipnir) provides the physics-based animation engine for CVKG.

## Features

*   **RK4 Solver**: Uses a Runge-Kutta 4th order integrator for ultra-smooth physics simulations.
*   **Spring Physics**: Implements critically damped springs for natural-feeling UI transitions.
*   **Stateless Animation Hooks**: Provides modifiers like `.animation()` that automatically transition property changes over time.
*   **Reactive Binding Integration**: Can be bound to CVKG `Binding` types to animate state changes.

## Usage
Animations in CVKG are typically applied via modifiers:
```rust
view.offset(x, y)
    .animation(Spring::stiff())
```
The engine calculates the delta and applies the physical transformation during the rendering pass.
