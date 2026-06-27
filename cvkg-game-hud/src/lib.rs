//! Game HUD component pack for CVKG.
//!
//! Provides game-specific UI components: HealthBar, ManaBar, CooldownIndicator,
//! DamageNumber, and Minimap. Each uses RK4 spring physics for smooth value
//! transitions rather than snapping instantly.

use cvkg_components::Progress;
use cvkg_core::{Color, Never, Rect, Renderer, View};

/// Internal spring animator using manual RK4 integration.
#[derive(Clone)]
struct SpringAnimator {
    display: f32,
    target: f32,
    velocity: f32,
    stiffness: f32,
    damping: f32,
    finished: bool,
}

impl SpringAnimator {
    fn new(initial: f32) -> Self {
        Self {
            display: initial,
            target: initial,
            velocity: 0.0,
            stiffness: 170.0,
            damping: 12.0,
            finished: true,
        }
    }

    fn set_target(&mut self, target: f32) {
        if (target - self.target).abs() > 0.001 {
            self.target = target;
            self.finished = false;
        }
    }

    /// Step the spring animation using RK4. Returns true if still animating.
    fn update(&mut self, dt: f32) -> bool {
        if self.finished {
            return false;
        }
        let dt = dt.min(0.033);
        let mut x = self.display;
        let mut v = self.velocity;
        let k = self.stiffness;
        let d = self.damping;

        let f = |px: f32, pv: f32| -> (f32, f32) { (pv, -k * (px - self.target) - d * pv) };

        let (k1_dx, k1_dv) = f(x, v);
        let (k2_dx, k2_dv) = f(x + 0.5 * dt * k1_dx, v + 0.5 * dt * k1_dv);
        let (k3_dx, k3_dv) = f(x + 0.5 * dt * k2_dx, v + 0.5 * dt * k2_dv);
        let (k4_dx, k4_dv) = f(x + dt * k3_dx, v + dt * k3_dv);

        x += (dt / 6.0) * (k1_dx + 2.0 * k2_dx + 2.0 * k3_dx + k4_dx);
        v += (dt / 6.0) * (k1_dv + 2.0 * k2_dv + 2.0 * k3_dv + k4_dv);

        self.display = x;
        self.velocity = v;

        if (x - self.target).abs() < 0.01 && v.abs() < 0.1 {
            self.finished = true;
            self.display = self.target;
            false
        } else {
            true
        }
    }

    fn current(&self) -> f32 {
        self.display
    }
}

/// Health bar with segmented display and smooth spring animation.
#[derive(Clone)]
pub struct HealthBar {
    current: f32,
    max: f32,
    color_full: Color,
    color_mid: Color,
    color_low: Color,
    height: f32,
    animator: SpringAnimator,
}

impl HealthBar {
    pub fn new(current: f32, max: f32) -> Self {
        Self {
            current,
            max,
            color_full: Color {
                r: 0.0,
                g: 0.8,
                b: 0.2,
                a: 1.0,
            },
            color_mid: Color {
                r: 1.0,
                g: 0.84,
                b: 0.0,
                a: 1.0,
            },
            color_low: Color {
                r: 1.0,
                g: 0.2,
                b: 0.1,
                a: 1.0,
            },
            height: 12.0,
            animator: SpringAnimator::new(current),
        }
    }

    /// Set the target health value. The bar animates toward this value.
    pub fn set_value(&mut self, value: f32) {
        self.current = value.clamp(0.0, self.max);
        self.animator.set_target(self.current);
    }

    /// Update animation. Call once per frame with delta time in seconds.
    pub fn update(&mut self, dt: f32) -> bool {
        self.animator.update(dt)
    }

    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    pub fn colors(mut self, full: Color, mid: Color, low: Color) -> Self {
        self.color_full = full;
        self.color_mid = mid;
        self.color_low = low;
        self
    }

    /// Returns true if the animation is still running.
    pub fn is_animating(&self) -> bool {
        !self.animator.finished
    }
}

impl View for HealthBar {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let val = self.animator.current();
        let pct = (val / self.max).clamp(0.0, 1.0);
        let color = if pct > 0.5 {
            self.color_full
        } else if pct > 0.25 {
            self.color_mid
        } else {
            self.color_low
        };
        let bar_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width * pct,
            height: self.height,
        };
        renderer.fill_rounded_rect(bar_rect, 4.0, [color.r, color.g, color.b, color.a]);
        let bg_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: self.height,
        };
        renderer.stroke_rounded_rect(bg_rect, 4.0, [0.3, 0.3, 0.3, 0.5], 1.0);
    }
}

/// Mana bar (blue, segmented) with spring animation.
#[derive(Clone)]
pub struct ManaBar {
    current: f32,
    max: f32,
    height: f32,
    animator: SpringAnimator,
}

impl ManaBar {
    pub fn new(current: f32, max: f32) -> Self {
        Self {
            current,
            max,
            height: 8.0,
            animator: SpringAnimator::new(current),
        }
    }

    pub fn set_value(&mut self, value: f32) {
        self.current = value.clamp(0.0, self.max);
        self.animator.set_target(self.current);
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.animator.update(dt)
    }

    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    pub fn is_animating(&self) -> bool {
        !self.animator.finished
    }
}

impl View for ManaBar {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let val = self.animator.current();
        let pct = (val / self.max).clamp(0.0, 1.0);
        let color = [0.2, 0.5, 1.0, 1.0];
        let bar_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width * pct,
            height: self.height,
        };
        renderer.fill_rounded_rect(bar_rect, 3.0, color);
    }
}

/// Cooldown indicator (circular sweep) with spring animation.
#[derive(Clone)]
pub struct CooldownIndicator {
    remaining: f32,
    total: f32,
    size: f32,
    animator: SpringAnimator,
}

impl CooldownIndicator {
    pub fn new(remaining: f32, total: f32) -> Self {
        Self {
            remaining,
            total,
            size: 40.0,
            animator: SpringAnimator::new(remaining),
        }
    }

    pub fn set_remaining(&mut self, value: f32) {
        self.remaining = value.clamp(0.0, self.total);
        self.animator.set_target(self.remaining);
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.animator.update(dt)
    }

    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }

    pub fn is_animating(&self) -> bool {
        !self.animator.finished
    }
}

impl View for CooldownIndicator {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let val = self.animator.current();
        let pct = (val / self.total).clamp(0.0, 1.0);
        let color = [0.3, 0.3, 0.4, 0.8 * pct];
        let bg_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: self.size,
            height: self.size,
        };
        renderer.fill_rounded_rect(bg_rect, self.size / 2.0, color);
    }
}

/// Floating damage number with spring-based float-up animation.
#[derive(Clone)]
pub struct DamageNumber {
    value: u32,
    color: Color,
    animator: SpringAnimator,
}

impl DamageNumber {
    pub fn new(value: u32) -> Self {
        Self {
            value,
            color: Color {
                r: 1.0,
                g: 0.3,
                b: 0.1,
                a: 1.0,
            },
            animator: SpringAnimator::new(0.0),
        }
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    pub fn heal(mut self) -> Self {
        self.color = Color {
            r: 0.0,
            g: 0.9,
            b: 0.3,
            a: 1.0,
        };
        self
    }

    /// Trigger the float-up animation.
    pub fn trigger(&mut self) {
        self.animator = SpringAnimator::new(0.0);
        self.animator.set_target(-30.0);
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.animator.update(dt)
    }

    pub fn is_animating(&self) -> bool {
        !self.animator.finished
    }
}

impl View for DamageNumber {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let offset = self.animator.current();
        let text = format!("{}", self.value);
        renderer.draw_text(
            &text,
            rect.x,
            rect.y + offset,
            16.0,
            [self.color.r, self.color.g, self.color.b, self.color.a],
        );
    }
}

/// Minimap placeholder.
#[derive(Clone)]
pub struct Minimap {
    size: f32,
}

impl Minimap {
    pub fn new() -> Self {
        Self { size: 120.0 }
    }

    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
}

impl View for Minimap {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let bg = Rect {
            x: rect.x,
            y: rect.y,
            width: self.size,
            height: self.size,
        };
        renderer.fill_rounded_rect(bg, 8.0, [0.1, 0.1, 0.15, 0.8]);
        renderer.stroke_rounded_rect(bg, 8.0, [0.3, 0.3, 0.4, 0.6], 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_bar_animates_toward_target() {
        let mut bar = HealthBar::new(100.0, 100.0);
        bar.set_value(50.0);

        assert_eq!(bar.animator.current(), 100.0);

        for _ in 0..120 {
            bar.update(1.0 / 60.0);
        }

        let current = bar.animator.current();
        assert!(
            (current - 50.0).abs() < 1.0,
            "Expected near 50.0, got {}",
            current
        );
    }

    #[test]
    fn health_bar_does_not_snap() {
        let mut bar = HealthBar::new(100.0, 100.0);
        bar.set_value(0.0);

        bar.update(1.0 / 60.0);
        let current = bar.animator.current();
        assert!(
            current > 0.0,
            "Bar should animate gradually, not snap. Got {}",
            current
        );
    }

    #[test]
    fn damage_number_floats_up() {
        let mut dmg = DamageNumber::new(42);
        dmg.trigger();

        // Spring overshoots initially, then settles toward target
        for _ in 0..60 {
            dmg.update(1.0 / 60.0);
        }

        let current = dmg.animator.current();
        assert!(
            current < -5.0,
            "Damage number should have floated up. Got {}",
            current
        );
    }

    #[test]
    fn spring_animator_reaches_target() {
        let mut anim = SpringAnimator::new(0.0);
        anim.set_target(100.0);

        for _ in 0..240 {
            anim.update(1.0 / 60.0);
        }

        assert!(
            (anim.current() - 100.0).abs() < 0.1,
            "Spring should reach target. Got {}",
            anim.current()
        );
    }

    #[test]
    fn spring_animator_is_finished_at_target() {
        let mut anim = SpringAnimator::new(50.0);
        anim.set_target(50.0);

        assert!(
            anim.finished,
            "Spring should be finished when target == current"
        );
    }

    #[test]
    fn spring_animator_overshoots_and_settles() {
        let mut anim = SpringAnimator::new(0.0);
        anim.set_target(50.0);

        let mut max_val: f32 = 0.0;
        for _ in 0..240 {
            anim.update(1.0 / 60.0);
            max_val = max_val.max(anim.current());
        }

        assert!(
            max_val > 50.0,
            "Spring should overshoot target. Max was {}",
            max_val
        );
        assert!(
            (anim.current() - 50.0).abs() < 0.1,
            "Spring should settle at target. Got {}",
            anim.current()
        );
    }
}
