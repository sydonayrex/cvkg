//! Easing curve library for animation timing functions.
//!
//! Provides standard CSS easing curves (ease-in, ease-out, ease-in-out, linear)
//! and a generic cubic bezier curve type. All functions map a normalized time
//! `t` in [0, 1] to an eased value in [0, 1].
//!
//! This module exists alongside the existing RK4 spring system (`spring_snap`)
//! for spring-physics-based animations. Easing curves are used for:
//! - Value transitions that don't need physics (fade in/out, slide)
//! - Timing functions for programmatic animations
//! - CSS-equivalent easing for consistent feel across UI
//!
//! # Examples
//!
//! ```
//! use cvkg_anim::easing::{Easing, ease_in_out};
//!
//! let t = 0.5;
//! let eased = Easing::EaseInOut.evaluate(t);
//! assert!(eased > 0.4 && eased < 0.6); // S-curve passes through ~0.5 at t=0.5
//!
//! // Or use the convenience function
//! let val = ease_in_out(0.25);
//! ```

/// Standard easing curves (CSS-compatible).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    /// Linear: no easing, constant velocity.
    Linear,
    /// Ease-in: starts slow, accelerates toward end.
    EaseIn,
    /// Ease-out: starts fast, decelerates toward end.
    EaseOut,
    /// Ease-in-out: slow start and end, fast middle.
    EaseInOut,
    /// Cubic bezier with custom control points.
    CubicBezier { x1: f32, y1: f32, x2: f32, y2: f32 },
    /// Step function (discrete jumps).
    Steps { count: u32, end_on_final: bool },
}

impl Easing {
    /// Evaluate the easing function at time `t` (clamped to [0, 1]).
    pub fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t * t,
            Easing::EaseOut => {
                let inv = 1.0 - t;
                1.0 - inv * inv * inv
            }
            Easing::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let inv = 2.0 * t - 2.0;
                    0.5 * inv * inv * inv + 1.0
                }
            }
            Easing::CubicBezier { x1, y1, x2, y2 } => cubic_bezier(t, *x1, *y1, *x2, *y2),
            Easing::Steps {
                count,
                end_on_final,
            } => {
                let count = *count;
                let end_on_final = *end_on_final;
                if count == 0 {
                    return t;
                }
                let steps = count as f32;
                let current_step = (t * steps).floor();
                if end_on_final {
                    if t >= 1.0 {
                        1.0
                    } else {
                        let s = current_step.min(steps - 1.0);
                        s / (steps - 1.0).max(1.0)
                    }
                } else {
                    let s = current_step.min(steps - 1.0);
                    s / steps
                }
            }
        }
    }

    /// Create an ease-in-out cubic bezier (CSS default).
    pub fn ease_in_out() -> Self {
        Easing::EaseInOut
    }

    /// Create an ease-in cubic bezier (CSS default).
    pub fn ease_in() -> Self {
        Easing::EaseIn
    }

    /// Create an ease-out cubic bezier (CSS default).
    pub fn ease_out() -> Self {
        Easing::EaseOut
    }

    /// Create a cubic bezier with CSS-style control points (0.0 to 1.0 range for y).
    pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Easing::CubicBezier { x1, y1, x2, y2 }
    }

    /// Create a steps easing function.
    pub fn steps(count: u32, end_on_final: bool) -> Self {
        Easing::Steps {
            count,
            end_on_final,
        }
    }

    /// Reverse the easing (mirror image: 1 - f(1-t)).
    pub fn reverse(&self) -> impl Fn(f32) -> f32 + Copy {
        move |t: f32| 1.0 - self.evaluate(1.0 - t)
    }
}

impl Default for Easing {
    fn default() -> Self {
        Easing::EaseInOut
    }
}

/// Convenience: ease-in-out evaluation.
pub fn ease_in_out(t: f32) -> f32 {
    Easing::EaseInOut.evaluate(t)
}

/// Convenience: ease-in evaluation.
pub fn ease_in(t: f32) -> f32 {
    Easing::EaseIn.evaluate(t)
}

/// Convenience: ease-out evaluation.
pub fn ease_out(t: f32) -> f32 {
    Easing::EaseOut.evaluate(t)
}

/// Convenience: linear evaluation.
pub fn linear(t: f32) -> f32 {
    Easing::Linear.evaluate(t)
}

/// Cubic bezier evaluation using Newton's method for x-then-y.
fn cubic_bezier(t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    // First, find the parameter `t_param` where x(t_param) == t
    let t_param = solve_cubic_bezier_x(t, x1, x2);
    // Then evaluate y at that parameter
    cubic_bezier_y(t_param, y1, y2)
}

/// Solve for parameter t where x(t) = target, using Newton's method.
fn solve_cubic_bezier_x(target: f32, x1: f32, x2: f32) -> f32 {
    let x0 = 0.0_f32;
    let x3 = 1.0_f32;

    // Initial guess
    let mut t = target;
    for _ in 0..8 {
        let x = cubic_bezier_point(t, x0, x1, x2, x3);
        let dx = cubic_bezier_dx(t, x0, x1, x2, x3);
        if dx.abs() < 1e-7 {
            break;
        }
        let diff = x - target;
        if diff.abs() < 1e-6 {
            break;
        }
        t -= diff / dx;
        t = t.clamp(0.0, 1.0);
    }
    t
}

/// Evaluate x component of cubic bezier at parameter t.
fn cubic_bezier_point(t: f32, p0: f32, p1: f32, p2: f32, p3: f32) -> f32 {
    let inv = 1.0 - t;
    inv * inv * inv * p0 + 3.0 * inv * inv * t * p1 + 3.0 * inv * t * t * p2 + t * t * t * p3
}

/// Derivative of cubic bezier x at parameter t.
fn cubic_bezier_dx(t: f32, p0: f32, p1: f32, p2: f32, p3: f32) -> f32 {
    let inv = 1.0 - t;
    3.0 * inv * inv * (p1 - p0) + 6.0 * inv * t * (p2 - p1) + 3.0 * t * t * (p3 - p2)
}

/// Evaluate y component of cubic bezier at parameter t.
fn cubic_bezier_y(t: f32, y1: f32, y2: f32) -> f32 {
    // y0 = 0, y3 = 1 for normalized easing
    let inv = 1.0 - t;
    3.0 * inv * inv * t * y1 + 3.0 * inv * t * t * y2 + t * t * t
}

/// Animate a value from `start` to `end` over time [0, 1] using an easing curve.
pub fn animate_value(start: f32, end: f32, t: f32, easing: Easing) -> f32 {
    let eased_t = easing.evaluate(t);
    start + (end - start) * eased_t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_is_identity() {
        assert!((linear(0.0) - 0.0).abs() < 1e-6);
        assert!((linear(0.25) - 0.25).abs() < 1e-6);
        assert!((linear(0.5) - 0.5).abs() < 1e-6);
        assert!((linear(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ease_in_is_slow_at_start() {
        // ease-in: cubic t^3. At t=0.25, value = 0.015625
        let val = ease_in(0.25);
        assert!(val < 0.05, "ease-in should be near start at t=0.25");
        assert!((ease_in(0.0) - 0.0).abs() < 1e-6);
        assert!((ease_in(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ease_out_is_fast_at_start() {
        // ease-out: 1 - (1-t)^3. At t=0.25, value = 0.578
        let val = ease_out(0.25);
        assert!(val > 0.5, "ease-out should be past halfway at t=0.25");
        assert!((ease_out(0.0) - 0.0).abs() < 1e-6);
        assert!((ease_out(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ease_in_out_is_s_shaped() {
        // ease-in-out should pass through 0.5 at t=0.5
        let mid = ease_in_out(0.5);
        assert!(
            (mid - 0.5).abs() < 1e-6,
            "ease-in-out midpoint should be 0.5"
        );
        assert!((ease_in_out(0.0) - 0.0).abs() < 1e-6);
        assert!((ease_in_out(1.0) - 1.0).abs() < 1e-6);
        // Should be < linear in early phase and > linear in late phase
        assert!(ease_in_out(0.25) < 0.25);
        assert!(ease_in_out(0.75) > 0.75);
    }

    #[test]
    fn ease_in_out_curve_shape() {
        // Verify intermediate values follow the expected ease-in-out cubic shape
        let v25 = ease_in_out(0.25);
        let v75 = ease_in_out(0.75);
        // At t=0.25: 0.5 * (2*0.25)^3 = 0.5 * 0.125 = 0.125 (but formula is different)
        // Actually for t < 0.5: 4*t^3 => 4 * 0.015625 = 0.0625
        assert!(
            v25 > 0.0 && v25 < 0.15,
            "ease-in-out at t=0.25 should be ~0.0625, got {}",
            v25
        );
        // At t=0.75: 0.5 * (2*0.75-2)^3 + 1 = 0.5 * (-0.5)^3 + 1 = 0.5 * (-0.125) + 1 = 0.9375
        assert!(
            v75 > 0.85 && v75 < 1.0,
            "ease-in-out at t=0.75 should be ~0.9375, got {}",
            v75
        );
    }

    #[test]
    fn cubic_bezier_matches_ease_in_out() {
        // CSS ease-in-out = cubic-bezier(0.42, 0, 0.58, 1)
        let bezier = Easing::cubic_bezier(0.42, 0.0, 0.58, 1.0);
        let builtin = Easing::EaseInOut;
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            let diff = (bezier.evaluate(t) - builtin.evaluate(t)).abs();
            assert!(
                diff < 0.1,
                "cubic-bezier(0.42,0,0.58,1) should match ease-in-out at t={}, diff={}",
                t,
                diff
            );
        }
    }

    #[test]
    fn steps_easing() {
        let steps = Easing::steps(4, true);
        assert!((steps.evaluate(0.0) - 0.0).abs() < 1e-6);
        // At t=0.1 (within first step): value = 0.0
        assert!((steps.evaluate(0.1) - 0.0).abs() < 1e-6);
        // At t=0.3 (passed first step boundary at 0.25): value = 1/3
        assert!((steps.evaluate(0.3) - 1.0 / 3.0).abs() < 1e-6);
        // At t=0.6 (passed second boundary at 0.5): value = 2/3
        assert!((steps.evaluate(0.6) - 2.0 / 3.0).abs() < 1e-6);
        // At t=1.0: final value = 1.0
        assert!((steps.evaluate(1.0) - 1.0).abs() < 1e-6);

        let steps_end = Easing::steps(4, false);
        // Without end_on_final, t=1.0 gives the penultimate step
        assert!((steps_end.evaluate(1.0) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn animate_value_uses_easing() {
        let start = 0.0;
        let end = 100.0;

        // Linear: midpoint at t=0.5
        assert!((animate_value(start, end, 0.5, Easing::Linear) - 50.0).abs() < 1e-6);

        // Ease-in: at t=0.5, value should be less than 50
        let val = animate_value(start, end, 0.5, Easing::EaseIn);
        assert!(val < 50.0, "ease-in at t=0.5 should be < 50, got {}", val);

        // Ease-out: at t=0.5, value should be more than 50
        let val = animate_value(start, end, 0.5, Easing::EaseOut);
        assert!(val > 50.0, "ease-out at t=0.5 should be > 50, got {}", val);
    }

    #[test]
    fn reverse_easing() {
        let easing = Easing::EaseIn;
        let reversed = easing.reverse();
        // reversed(t) = 1 - easing(1-t) = easing_out behavior
        let val = reversed(0.25);
        assert!(
            val > 0.5,
            "reversed ease-in at t=0.25 should be > 0.5, got {}",
            val
        );
    }

    #[test]
    fn clamping() {
        assert!((ease_in_out(-1.0) - 0.0).abs() < 1e-6);
        assert!((ease_in_out(2.0) - 1.0).abs() < 1e-6);
    }
}
