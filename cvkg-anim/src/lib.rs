//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     — State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     — Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     — Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    — Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     — Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     — Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   — Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.

//! # Sleipnir Animation Engine
//!
//! Provides high-fidelity physics-based animation and transition systems for CVKG.

use std::time::Duration;
use std::sync::Arc;

pub mod particles;
pub use particles::*;

/// Sleipnir spring parameters for the physics solver
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SleipnirParams {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

impl SleipnirParams {
    pub fn snappy() -> Self { Self { stiffness: 230.0, damping: 22.0, mass: 1.0 } }
    pub fn fluid() -> Self { Self { stiffness: 170.0, damping: 26.0, mass: 1.0 } }
    pub fn heavy() -> Self { Self { stiffness: 90.0, damping: 20.0, mass: 1.0 } }
    pub fn bouncy() -> Self { Self { stiffness: 190.0, damping: 14.0, mass: 1.0 } }
}

impl Default for SleipnirParams {
    fn default() -> Self { Self::fluid() }
}

/// A discrete keyframe in a hybrid animation path
#[derive(Debug, Clone, PartialEq)]
pub struct Keyframe {
    pub value: f32,
    pub time: Duration,
    pub easing: Easing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

/// High-level Animation Primitive
#[derive(Clone)]
pub enum Animation {
    /// No animation (instant)
    Ginnungagap,
    /// Linear animation
    Linear { duration: Duration },
    /// Organic spring animation
    Sleipnir(SleipnirParams),
    /// Hybrid: Keyframe path followed by a Spring settle
    Hybrid {
        keyframes: Vec<Keyframe>,
        settle: SleipnirParams,
    },
    /// Coordination: Multiple animations in parallel
    Parallel(Vec<Animation>),
    /// Coordination: Multiple animations in sequence
    Sequence(Vec<Animation>),
    /// Coordination: Staggered start for multiple animations
    /// Coordination: Staggered start for multiple animations
    Stagger {
        animations: Vec<Animation>,
        interval: Duration,
    },
    /// Bifrost transition (Glass-aware fade)
    BifrostFade { duration: Duration },
    /// Mjolnir transition (Geometric slice)
    MjolnirSlice { duration: Duration },
    /// Mjolnir transition (Physical shatter)
    MjolnirShatter {
        duration: Duration,
        pieces: u32,
        force: f32,
    },
}

/// Tactile "Rubber Banding" utility for scroll/drag physics.
/// Maps an unbounded input value to a bounded range with elastic resistance.
pub struct RubberBand {
    /// Minimum bound of the valid range
    pub min: f32,
    /// Maximum bound of the valid range
    pub max: f32,
    /// Resistance constant (higher = stiffer)
    pub constant: f32,
}

impl RubberBand {
    /// Create a new RubberBand solver with default resistance.
    pub fn new(min: f32, max: f32) -> Self {
        Self { min, max, constant: 0.55 }
    }

    /// Calculate the resisted value for an input that may exceed bounds.
    pub fn solve(&self, input: f32) -> f32 {
        if input < self.min {
            self.min - self.apply_resistance(self.min - input)
        } else if input > self.max {
            self.max + self.apply_resistance(input - self.max)
        } else {
            input
        }
    }

    fn apply_resistance(&self, delta: f32) -> f32 {
        // Logarithmic resistance similar to iOS/WebKit
        (delta * self.constant).atan() * (1.0 / self.constant)
    }
}

/// Motion controller that handles lifecycle events and state transitions.
pub struct Motion {
    /// The target animation sequence or spring
    pub animation: Animation,
    /// Callback triggered when the animation starts
    pub on_start: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Callback triggered when the physics settle at the target
    pub on_settle: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Callback triggered if the animation is interrupted by a new target
    pub on_interrupt: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl Motion {
    /// Create a new Motion controller for an animation.
    pub fn new(animation: Animation) -> Self {
        Self {
            animation,
            on_start: None,
            on_settle: None,
            on_interrupt: None,
        }
    }
}

/// SleipnirSolver implements a 4th-order Runge-Kutta (RK4) integration for springs.
/// This provides superior stability for high-fidelity interactive motion.
pub struct SleipnirSolver {
    params: SleipnirParams,
    target: f32,
    state: SolverState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SolverState {
    x: f32,
    v: f32,
}

impl SleipnirSolver {
    /// Create a new solver with a target value and starting state.
    pub fn new(params: SleipnirParams, target: f32, current: f32) -> Self {
        Self {
            params,
            target,
            state: SolverState { x: current, v: 0.0 },
        }
    }

    /// Advance the simulation by dt seconds using RK4 integration.
    pub fn tick(&mut self, dt: f32) -> f32 {
        let a = self.evaluate(self.state, 0.0, SolverState { x: 0.0, v: 0.0 });
        let b = self.evaluate(self.state, dt * 0.5, a);
        let c = self.evaluate(self.state, dt * 0.5, b);
        let d = self.evaluate(self.state, dt, c);

        let dxdt = 1.0 / 6.0 * (a.x + 2.0 * (b.x + c.x) + d.x);
        let dvdt = 1.0 / 6.0 * (a.v + 2.0 * (b.v + c.v) + d.v);

        self.state.x += dxdt * dt;
        self.state.v += dvdt * dt;
        self.state.x
    }

    fn evaluate(&self, initial: SolverState, dt: f32, d: SolverState) -> SolverState {
        let state = SolverState {
            x: initial.x + d.x * dt,
            v: initial.v + d.v * dt,
        };
        let force = -self.params.stiffness * (state.x - self.target) - self.params.damping * state.v;
        // Protect against division by zero; mass must be positive.
        let mass = self.params.mass.max(0.001);
        SolverState { x: state.v, v: force / mass }
    }

    pub fn is_settled(&self) -> bool {
        (self.state.x - self.target).abs() < 0.001 && self.state.v.abs() < 0.001
    }
}

/// Active animation state tracker
pub struct ActiveAnimation {
    pub animation: Animation,
    pub elapsed: Duration,
    pub is_finished: bool,
    pub current_value: f32,
    
    // Internal state for complex animations
    solver: Option<SleipnirSolver>,
    child_states: Vec<ActiveAnimation>,
    current_index: usize,
}

impl ActiveAnimation {
    pub fn new(animation: Animation) -> Self {
        Self {
            animation,
            elapsed: Duration::ZERO,
            is_finished: false,
            current_value: 0.0,
            solver: None,
            child_states: Vec::new(),
            current_index: 0,
        }
    }

    pub fn update(&mut self, dt: Duration, start_val: f32, end_val: f32) -> f32 {
        if self.is_finished { return end_val; }
        
        self.elapsed += dt;
        let t = self.elapsed.as_secs_f32();
        
        match &self.animation {
            Animation::Ginnungagap => {
                self.is_finished = true;
                self.current_value = end_val;
            }
            Animation::Linear { duration } => {
                let d = duration.as_secs_f32();
                if t >= d {
                    self.is_finished = true;
                    self.current_value = end_val;
                } else {
                    self.current_value = start_val + (end_val - start_val) * (t / d);
                }
            }
            Animation::Sleipnir(params) => {
                let solver = self.solver.get_or_insert_with(|| SleipnirSolver::new(*params, end_val, start_val));
                self.current_value = solver.tick(dt.as_secs_f32());
                if solver.is_settled() {
                    self.is_finished = true;
                }
            }
            Animation::Sequence(anims) => {
                if self.current_index >= anims.len() {
                    self.is_finished = true;
                    self.current_value = end_val;
                } else {
                    if self.child_states.is_empty() {
                        self.child_states = anims.iter().map(|a| ActiveAnimation::new(a.clone())).collect();
                    }
                    
                    let child = &mut self.child_states[self.current_index];
                    self.current_value = child.update(dt, start_val, end_val);
                    
                    if child.is_finished {
                        self.current_index += 1;
                        if self.current_index >= anims.len() {
                            self.is_finished = true;
                        }
                    }
                }
            }
            Animation::Parallel(anims) => {
                if self.child_states.is_empty() {
                    self.child_states = anims.iter().map(|a| ActiveAnimation::new(a.clone())).collect();
                }
                
                let mut all_finished = true;
                let mut sum_val = 0.0;
                for child in &mut self.child_states {
                    sum_val += child.update(dt, start_val, end_val);
                    if !child.is_finished {
                        all_finished = false;
                    }
                }
                
                self.current_value = if !anims.is_empty() { sum_val / anims.len() as f32 } else { end_val };
                if all_finished {
                    self.is_finished = true;
                }
            }
            _ => {
                // Fallback for complex variants (Stagger, Transitions)
                self.is_finished = true;
                self.current_value = end_val;
            }
        }
        self.current_value
    }
}

pub trait AnimationValue: Sized + Clone + PartialEq {
    fn lerp(&self, other: &Self, t: f32) -> Self;
    fn distance(&self, other: &Self) -> f32;
}

impl AnimationValue for f32 {
    fn lerp(&self, other: &Self, t: f32) -> Self { self + (other - self) * t }
    fn distance(&self, other: &Self) -> f32 { (self - other).abs() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rubber_band_solving() {
        let rb = RubberBand::new(0.0, 100.0);
        
        // Inside bounds
        assert_eq!(rb.solve(50.0), 50.0);
        
        // Above bounds
        let over = rb.solve(150.0);
        assert!(over > 100.0);
        assert!(over < 150.0); // Resistance applied
        
        // Below bounds
        let under = rb.solve(-50.0);
        assert!(under < 0.0);
        assert!(under > -50.0); // Resistance applied
    }

    #[test]
    fn test_sleipnir_solver_convergence() {
        let params = SleipnirParams::snappy();
        let mut solver = SleipnirSolver::new(params, 100.0, 0.0);
        
        // Initial state
        assert!(!solver.is_settled());
        
        // Simulate some ticks
        for _ in 0..100 {
            solver.tick(0.016);
        }
        
        // Should eventually settle near target
        assert!(solver.is_settled());
        assert!((solver.state.x - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_animation_sequence_execution() {
        let anims = vec![
            Animation::Linear { duration: Duration::from_millis(100) },
            Animation::Linear { duration: Duration::from_millis(100) },
        ];
        let mut active = ActiveAnimation::new(Animation::Sequence(anims));
        
        // Update first animation halfway
        active.update(Duration::from_millis(50), 0.0, 100.0);
        assert!(!active.is_finished);
        assert_eq!(active.current_index, 0);
        
        // Complete first animation
        active.update(Duration::from_millis(60), 0.0, 100.0);
        assert!(!active.is_finished);
        assert_eq!(active.current_index, 1);
        
        // Complete second animation
        active.update(Duration::from_millis(100), 0.0, 100.0);
        assert!(active.is_finished);
    }
}
