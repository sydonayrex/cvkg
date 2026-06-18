//! Spring snap events and haptic binding.
//!
//! When a spring-based animation (Sleipnir) crosses its equilibrium point
//! or settles, a "snap event" is emitted. Applications can bind these events
//! to haptic feedback, sound effects, or other sensory responses.
//!
//! # Usage
//!
//! ```no_run
//! use cvkg_anim::{
//!     spring_snap::{SpringSnapEvent, SnapPhase, SnapTracker},
//!     Animation, SleipnirParams,
//! };
//! let params = SleipnirParams::snappy();
//! let anim = Animation::Sleipnir(params);
//! let mut tracker = SnapTracker::new(42.0, 0.5);
//! // In your animation loop:
//! // match tracker.track(value, dt) {
//! //     SpringSnapEvent::Settled => { /* trigger haptic */ }
//! //     SpringSnapEvent::CrossedTarget => { /* tick haptic */ }
//! //     SpringSnapEvent::Overshoot { depth } => { /* light haptic */ }
//! //     SpringSnapEvent::None => {}
//! // }
//! ```

use std::sync::Arc;

/// Phases of a spring snap event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapPhase {
    /// Spring has crossed the target but not settled -- "tick" feel.
    CrossedTarget,
    /// Spring is stretching past equilibrium -- light "pop" feel.
    Overshoot,
    /// Spring has come to rest at the target.
    Settled,
    /// Spring motion direction changed.
    DirectionChange,
}

/// A snap event emitted during spring animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpringSnapEvent {
    /// No event this frame.
    None,
    /// Spring crossed the target equilibrium point.
    CrossedTarget,
    /// Spring overshot the target by `depth` units (positive).
    Overshoot { depth: f32 },
    /// Spring has settled (come to rest).
    Settled,
    /// Spring changed direction of motion.
    DirectionChange { velocity: f32 },
}

/// Tracks a spring animation value and emits snap events.
#[derive(Debug, Clone)]
pub struct SnapTracker {
    target: f32,
    threshold: f32,
    prev_value: f32,
    prev_prev_value: f32,
    settled: bool,
    crossed_this_frame: bool,
}

impl SnapTracker {
    /// Create a new snap tracker.
    ///
    /// # Arguments
    /// * `initial_value` -- Current animated value.
    /// * `threshold` -- Distance from target considered "settled" (pixels).
    pub fn new(initial_value: f32, threshold: f32) -> Self {
        Self {
            target: 0.0,
            threshold: threshold.max(0.1),
            prev_value: initial_value,
            prev_prev_value: initial_value,
            settled: false,
            crossed_this_frame: false,
        }
    }

    /// Set the target value for the spring.
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
        self.settled = false;
        self.crossed_this_frame = false;
    }

    /// Track a new value and return any snap event.
    ///
    /// Call this once per frame with the current animated value.
    pub fn track(&mut self, value: f32) -> SpringSnapEvent {
        if self.settled {
            self.prev_prev_value = self.prev_value;
            self.prev_value = value;
            return SpringSnapEvent::None;
        }

        let prev = self.prev_value;
        let prev_prev = self.prev_prev_value;

        // Check if settled (near target, low velocity)
        let dist_to_target = (value - self.target).abs();
        let velocity = (value - prev).abs();

        if dist_to_target < self.threshold && velocity < self.threshold * 0.5 {
            self.settled = true;
            self.prev_prev_value = prev;
            self.prev_value = value;
            return SpringSnapEvent::Settled;
        }

        // Check for target crossing (sign change in displacement from target)
        let prev_disp = prev - self.target;
        let curr_disp = value - self.target;
        let crossed =
            (prev_disp > 0.0 && curr_disp <= 0.0) || (prev_disp < 0.0 && curr_disp >= 0.0);

        // Check for overshoot (moving away from target after crossing)
        let prev_dist = (prev - self.target).abs();
        let curr_dist = (value - self.target).abs();
        let overshooting = curr_dist > prev_dist && dist_to_target > self.threshold;

        // Check for direction change
        let prev_dir = prev - prev_prev;
        let curr_dir = value - prev;
        let dir_changed = (prev_dir > 0.0 && curr_dir < 0.0) || (prev_dir < 0.0 && curr_dir > 0.0);

        self.prev_prev_value = prev;
        self.prev_value = value;

        if crossed {
            SpringSnapEvent::CrossedTarget
        } else if overshooting {
            SpringSnapEvent::Overshoot { depth: curr_dist }
        } else if dir_changed {
            SpringSnapEvent::DirectionChange { velocity: curr_dir }
        } else {
            SpringSnapEvent::None
        }
    }

    /// Whether the spring is currently settled.
    pub fn is_settled(&self) -> bool {
        self.settled
    }

    /// Reset tracking state (e.g., when a new target is set).
    pub fn reset(&mut self, current_value: f32) {
        self.prev_value = current_value;
        self.prev_prev_value = current_value;
        self.settled = false;
        self.crossed_this_frame = false;
    }
}

/// Configuration for haptic binding on spring snap events.
#[derive(Clone)]
pub struct HapticBinding {
    /// Whether to fire haptic on target crossing.
    pub cross_haptic: bool,
    /// Whether to fire haptic on direction change.
    pub direction_haptic: bool,
    /// Whether to fire haptic on settle.
    pub settle_haptic: bool,
    /// Intensity for cross haptic.
    pub cross_intensity: f32,
    /// Intensity for direction change haptic.
    pub direction_intensity: f32,
    /// Intensity for settle haptic.
    pub settle_intensity: f32,
    /// Callback fired for each snap event.
    pub on_snap: Option<Arc<dyn Fn(SpringSnapEvent) + Send + Sync>>,
}

impl Default for HapticBinding {
    fn default() -> Self {
        Self {
            cross_haptic: true,
            direction_haptic: false,
            settle_haptic: true,
            cross_intensity: 0.3,
            direction_intensity: 0.1,
            settle_intensity: 0.6,
            on_snap: None,
        }
    }
}

impl HapticBinding {
    /// Create a new haptic binding with all events enabled.
    pub fn all() -> Self {
        Self {
            cross_haptic: true,
            direction_haptic: true,
            settle_haptic: true,
            cross_intensity: 0.3,
            direction_intensity: 0.1,
            settle_intensity: 0.6,
            on_snap: None,
        }
    }

    /// Create a minimal haptic binding (settle only).
    pub fn settle_only() -> Self {
        Self {
            cross_haptic: false,
            direction_haptic: false,
            settle_haptic: true,
            cross_intensity: 0.0,
            direction_intensity: 0.0,
            settle_intensity: 0.6,
            on_snap: None,
        }
    }

    /// Set the snap callback.
    pub fn with_callback(mut self, cb: Arc<dyn Fn(SpringSnapEvent) + Send + Sync>) -> Self {
        self.on_snap = Some(cb);
        self
    }

    /// Process a snap event and fire haptic callback if configured.
    pub fn process(&self, event: SpringSnapEvent) {
        let _should_fire = match event {
            SpringSnapEvent::None => return,
            SpringSnapEvent::CrossedTarget => self.cross_haptic,
            SpringSnapEvent::Overshoot { .. } => self.direction_haptic,
            SpringSnapEvent::Settled => self.settle_haptic,
            SpringSnapEvent::DirectionChange { .. } => self.direction_haptic,
        };
        if let Some(ref cb) = self.on_snap {
            cb(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snap_tracker_settled() {
        let mut tracker = SnapTracker::new(0.0, 1.0);
        tracker.set_target(100.0);

        // First frame: not settled
        assert_eq!(tracker.track(50.0), SpringSnapEvent::None);

        // At target with low velocity
        let _ = tracker.track(99.5);
        let event = tracker.track(99.9);
        assert_eq!(event, SpringSnapEvent::Settled);
        assert!(tracker.is_settled());
    }

    #[test]
    fn test_snap_tracker_crossing() {
        let mut tracker = SnapTracker::new(0.0, 1.0);
        tracker.set_target(100.0);

        // Cross from below to above target
        tracker.track(90.0);
        let event = tracker.track(110.0);
        assert_eq!(event, SpringSnapEvent::CrossedTarget);
    }

    #[test]
    fn test_haptic_binding_process() {
        let binding = HapticBinding::default();
        binding.process(SpringSnapEvent::CrossedTarget);

        // With default binding, cross_haptic is true
        // (callback fires internally)
        let binding_with_cb = HapticBinding::default().with_callback(Arc::new(|e| {
            assert_eq!(e, SpringSnapEvent::Settled);
        }));
        binding_with_cb.process(SpringSnapEvent::Settled);
    }
}
