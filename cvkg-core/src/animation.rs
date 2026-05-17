//! Spring-physics animated value wrapper.
//!
//! `Animated<T>` wraps any value that can be linearly interpolated and provides
//! a spring-damper integration that converges toward a target without overshoot
//! artifacts. Used by the layout engine and view modifiers to drive smooth
//! transitions (e.g. magnetic pull, focus changes, scroll animations).

use std::time::Duration;

/// Configuration for a critically-damped spring.
///
/// The spring follows the equation: `a = -2 * zeta * omega * v - omega^2 * (x - target)`
/// where `zeta` is the damping ratio and `omega = 2 * PI / period`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringConfig {
    /// Oscillation period in seconds. Smaller = snappier.
    pub period: f32,
    /// Damping ratio. 1.0 = critically damped (no overshoot).
    /// < 1.0 = underdamped (bouncy), > 1.0 = overdamped (sluggish).
    pub damping_ratio: f32,
    /// Convergence threshold. When both position delta and velocity fall below
    /// this, the spring is considered settled.
    pub epsilon: f32,
}

impl SpringConfig {
    /// A snappy spring (150ms period, critically damped).
    pub fn snappy() -> Self {
        Self {
            period: 0.15,
            damping_ratio: 1.0,
            epsilon: 0.001,
        }
    }

    /// A smooth spring (300ms period, critically damped).
    pub fn smooth() -> Self {
        Self {
            period: 0.30,
            damping_ratio: 1.0,
            epsilon: 0.001,
        }
    }

    /// A bouncy spring (400ms period, slightly underdamped).
    pub fn bouncy() -> Self {
        Self {
            period: 0.40,
            damping_ratio: 0.7,
            epsilon: 0.005,
        }
    }
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self::snappy()
    }
}

/// A value animated by spring physics toward a target.
///
/// Each call to `update(dt)` advances the simulation by `dt` seconds.
/// Use `set_target()` to change the destination; the spring will
/// automatically converge.
///
/// Type requirements on `T`:
/// - `Copy + Default` for value semantics
/// - `std::ops::Add<Output = T>` for position + velocity integration
/// - `std::ops::Sub<Output = T>` for displacement computation
/// - `std::ops::Mul<f32, Output = T>` for scalar multiplication (velocity, damping)
/// - `f32::from(&T)` via a custom trait for convergence checking
///
/// For `f32`, `glam::Vec2`, `glam::Vec4`, and `Color`, blanket impls are provided.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Animated<T: SpringValue> {
    pub value: T,
    target: T,
    velocity: T,
    config: SpringConfig,
    settled: bool,
}

/// Trait for types that can be used inside `Animated<T>`.
///
/// Provides linear interpolation, scalar multiplication, and convergence detection.
pub trait SpringValue:
    Copy + Default + std::ops::Add<Output = Self> + std::ops::Sub<Output = Self>
{
    /// Multiply by a scalar.
    fn mul_scalar(self, s: f32) -> Self;
    /// Element-wise absolute value.
    fn abs(self) -> Self;
    /// Element-wise maximum with a scalar threshold.
    fn max_scalar(self, s: f32) -> Self;
    /// Returns true if all components are approximately zero (within epsilon).
    fn is_near_zero(self, epsilon: f32) -> bool;
    /// Linearly interpolate between self and other by t in [0, 1].
    fn lerp(self, other: Self, t: f32) -> Self;
}

impl SpringValue for f32 {
    fn mul_scalar(self, s: f32) -> Self {
        self * s
    }
    fn abs(self) -> Self {
        self.abs()
    }
    fn max_scalar(self, s: f32) -> Self {
        self.max(s)
    }
    fn is_near_zero(self, epsilon: f32) -> bool {
        self.abs() < epsilon
    }
    fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl SpringValue for glam::Vec2 {
    fn mul_scalar(self, s: f32) -> Self {
        self * s
    }
    fn abs(self) -> Self {
        self.abs()
    }
    fn max_scalar(self, s: f32) -> Self {
        Self::new(self.x.max(s), self.y.max(s))
    }
    fn is_near_zero(self, epsilon: f32) -> bool {
        self.x.abs() < epsilon && self.y.abs() < epsilon
    }
    fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl SpringValue for glam::Vec4 {
    fn mul_scalar(self, s: f32) -> Self {
        self * s
    }
    fn abs(self) -> Self {
        self.abs()
    }
    fn max_scalar(self, s: f32) -> Self {
        Self::new(self.x.max(s), self.y.max(s), self.z.max(s), self.w.max(s))
    }
    fn is_near_zero(self, epsilon: f32) -> bool {
        self.x.abs() < epsilon
            && self.y.abs() < epsilon
            && self.z.abs() < epsilon
            && self.w.abs() < epsilon
    }
    fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl<T: SpringValue + std::cmp::PartialEq> Animated<T> {
    /// Creates a new animated value starting at `initial` with the given spring config.
    pub fn new(initial: T, config: SpringConfig) -> Self {
        Self {
            value: initial,
            target: initial,
            velocity: T::default(),
            config,
            settled: true,
        }
    }

    /// Creates a new animated value starting at `initial` with default (snappy) config.
    pub fn default(initial: T) -> Self {
        Self::new(initial, SpringConfig::default())
    }

    /// Sets the target value the spring should converge toward.
    pub fn set_target(&mut self, target: T) {
        if self.target != target {
            self.target = target;
            self.settled = false;
        }
    }

    /// Returns the current target value.
    pub fn target(&self) -> T {
        self.target
    }

    /// Returns true if the spring has converged and no further updates are needed.
    pub fn is_settled(&self) -> bool {
        self.settled
    }

    /// Advances the spring simulation by `dt` seconds.
    ///
    /// Uses a semi-implicit Euler integration of the spring-damper equation:
    /// ```text
    ///   omega = 2 * PI / period
    ///   displacement = value - target
    ///   acceleration = -2 * zeta * omega * velocity - omega^2 * displacement
    ///   velocity += acceleration * dt
    ///   value += velocity * dt
    /// ```
    pub fn update(&mut self, dt: f32) {
        if self.settled {
            return;
        }

        let omega = 2.0 * std::f32::consts::PI / self.config.period;
        let zeta = self.config.damping_ratio;
        let displacement = self.value - self.target;
        let acceleration =
            displacement.mul_scalar(-omega * omega) + self.velocity.mul_scalar(-2.0 * zeta * omega);
        self.velocity = self.velocity + acceleration.mul_scalar(dt);
        self.value = self.value + self.velocity.mul_scalar(dt);

        // Check convergence
        if displacement.is_near_zero(self.config.epsilon)
            && self.velocity.is_near_zero(self.config.epsilon)
        {
            self.value = self.target;
            self.velocity = T::default();
            self.settled = true;
        }
    }

    /// Advances the spring by a `Duration`.
    pub fn update_duration(&mut self, dt: Duration) {
        self.update(dt.as_secs_f32());
    }

    /// Immediately snaps to the target without animation.
    pub fn snap_to_target(&mut self) {
        self.value = self.target;
        self.velocity = T::default();
        self.settled = true;
    }

    /// Resets the value and target to `value` without animation.
    pub fn reset(&mut self, value: T) {
        self.value = value;
        self.target = value;
        self.velocity = T::default();
        self.settled = true;
    }

    /// Returns the current spring configuration.
    pub fn config(&self) -> SpringConfig {
        self.config
    }

    /// Replaces the spring configuration.
    pub fn set_config(&mut self, config: SpringConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_converges_to_target() {
        let mut anim = Animated::new(0.0f32, SpringConfig::snappy());
        anim.set_target(1.0);
        for _ in 0..100 {
            anim.update(0.016);
        }
        assert!(anim.is_settled());
        assert!((anim.value - 1.0).abs() < 0.01);
    }

    #[test]
    fn spring_is_initially_settled() {
        let anim = Animated::new(5.0f32, SpringConfig::default());
        assert!(anim.is_settled());
    }

    #[test]
    fn spring_snap_to_target() {
        let mut anim = Animated::new(0.0f32, SpringConfig::snappy());
        anim.set_target(10.0);
        anim.snap_to_target();
        assert!(anim.is_settled());
        assert_eq!(anim.value, 10.0);
    }

    #[test]
    fn spring_reset() {
        let mut anim = Animated::new(0.0f32, SpringConfig::snappy());
        anim.set_target(10.0);
        anim.update(0.016);
        anim.reset(5.0);
        assert!(anim.is_settled());
        assert_eq!(anim.value, 5.0);
        assert_eq!(anim.target(), 5.0);
    }

    #[test]
    fn spring_vec2_converges() {
        let mut anim = Animated::new(glam::Vec2::ZERO, SpringConfig::smooth());
        anim.set_target(glam::Vec2::new(100.0, 200.0));
        for _ in 0..200 {
            anim.update(0.016);
        }
        assert!(anim.is_settled());
        assert!((anim.value.x - 100.0).abs() < 0.01);
        assert!((anim.value.y - 200.0).abs() < 0.01);
    }

    #[test]
    fn spring_does_not_overshoot_critically_damped() {
        let mut anim = Animated::new(0.0f32, SpringConfig::snappy());
        anim.set_target(1.0);
        let mut max_val = 0.0f32;
        for _ in 0..200 {
            anim.update(0.016);
            if anim.value > max_val {
                max_val = anim.value;
            }
        }
        // Critically damped should not overshoot by more than ~5%
        assert!(max_val < 1.05, "overshoot: {}", max_val);
    }
}
