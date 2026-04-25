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
//!
//! Sources:
//! Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//! CVKG Extended: Section 2 of the CVKG Design Specification

//! # Sleipnir Animation Engine
//!
//! Provides high-fidelity physics-based animation and transition systems for CVKG.
//!
//! - **Sleipnir**: RK4 Spring Physics solver for organic, interruptible motion.
//! - **Bifrost Transitions**: Smooth glass-aware fades and blurs.
//! - **Mjolnir Transitions**: Hard geometric slicing and shattering effects.

use std::time::Duration;

/// Sleipnir spring parameters for the physics solver
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SleipnirParams {
    /// Spring stiffness (tension)
    pub stiffness: f32,
    /// Damping ratio (friction)
    pub damping: f32,
    /// Mass of the object
    pub mass: f32,
}

impl Default for SleipnirParams {
    fn default() -> Self {
        Self {
            stiffness: 170.0,
            damping: 26.0,
            mass: 1.0,
        }
    }
}

/// Animation describes how a view should animate over time
#[derive(Debug, Clone, PartialEq)]
pub enum Animation {
    /// No animation (instant)
    Ginnungagap,
    /// Linear animation
    Linear { duration: Duration },
    /// Organic spring animation (The 8-legged horse)
    Sleipnir(SleipnirParams),
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

impl Animation {
    /// Create a default Sleipnir spring animation
    pub fn sleipnir() -> Self {
        Animation::Sleipnir(SleipnirParams::default())
    }

    /// Create a Bifrost fade transition
    pub fn bifrost_fade(duration: Duration) -> Self {
        Animation::BifrostFade { duration }
    }
}

/// A state in the Sleipnir physics solver
#[derive(Debug, Clone, Copy, PartialEq)]
struct SolverState {
    x: f32,
    v: f32,
}

/// SleipnirSolver implements a 4th-order Runge-Kutta (RK4) integration for springs.
/// This provides superior stability and precision compared to Euler integration,
/// especially for high-stiffness springs.
pub struct SleipnirSolver {
    params: SleipnirParams,
    target: f32,
    state: SolverState,
}

impl SleipnirSolver {
    /// Create a new solver with a target value and starting state
    pub fn new(params: SleipnirParams, target: f32, current: f32) -> Self {
        Self {
            params,
            target,
            state: SolverState { x: current, v: 0.0 },
        }
    }

    /// Advance the simulation by dt seconds using RK4 integration
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

    /// Evaluate acceleration at a specific sub-step
    fn evaluate(&self, initial: SolverState, dt: f32, d: SolverState) -> SolverState {
        let state = SolverState {
            x: initial.x + d.x * dt,
            v: initial.v + d.v * dt,
        };

        let force =
            -self.params.stiffness * (state.x - self.target) - self.params.damping * state.v;
        let acceleration = force / self.params.mass;

        SolverState {
            x: state.v,
            v: acceleration,
        }
    }

    /// Check if the spring has effectively settled at the target
    pub fn is_settled(&self) -> bool {
        (self.state.x - self.target).abs() < 0.001 && self.state.v.abs() < 0.001
    }
}

/// AnimationValue represents a value that can be interpolated or animated via physics
pub trait AnimationValue: Sized + Clone + PartialEq {
    /// Linear interpolation between two values
    fn lerp(&self, other: &Self, t: f32) -> Self;

    /// Distance between two values (used for convergence checks)
    fn distance(&self, other: &Self) -> f32;
}

impl AnimationValue for f32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }

    fn distance(&self, other: &Self) -> f32 {
        (self - other).abs()
    }
}

/// Color represented in the Nifl/Muspel colorway
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NiflColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl AnimationValue for NiflColor {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    fn distance(&self, other: &Self) -> f32 {
        ((self.r - other.r).powi(2) + (self.g - other.g).powi(2) + (self.b - other.b).powi(2))
            .sqrt()
    }
}
