//! FluxLayout -- Layout Animation for Sibling Repositioning.
//!
//! When an item in a list grows or shrinks, FluxLayout smoothly animates
//! the siblings below it to their new positions instead of snapping.

use std::cell::RefCell;
use std::collections::HashMap;

/// Tracks previous rects for child views and provides animated transitions.
///
/// On each render, FluxLayout compares current child rects to the previous
/// frame's rects. Children that moved are rendered at an interpolated position
/// between old and new, creating a smooth sliding effect.
pub struct FluxState {
    /// Maps child index to its previous rect.
    previous_rects: RefCell<HashMap<usize, cvkg_core::Rect>>,
    /// Animation progress 0.0 (old) to 1.0 (new).
    progress: RefCell<f32>,
    /// Animation duration in seconds.
    duration: f32,
}

impl FluxState {
    /// Create a new FluxState with the given animation duration.
    pub fn new(duration: f32) -> Self {
        Self {
            previous_rects: RefCell::new(HashMap::new()),
            progress: RefCell::new(1.0),
            duration: duration.max(0.05),
        }
    }

    /// Get the interpolated rect for a child.
    ///
    /// If the child moved since last frame, returns a rect interpolated
    /// between the old and new positions. If it hasn't moved, returns
    /// the new rect directly.
    pub fn interpolated_rect(
        &self,
        child_idx: usize,
        new_rect: cvkg_core::Rect,
    ) -> cvkg_core::Rect {
        let mut prev = self.previous_rects.borrow_mut();
        let mut prog = self.progress.borrow_mut();

        let result = if let Some(&old_rect) = prev.get(&child_idx) {
            if (old_rect.x - new_rect.x).abs() > 0.5 || (old_rect.y - new_rect.y).abs() > 0.5 {
                // Child moved - interpolate
                *prog = 0.0;
                lerp_rect(&old_rect, &new_rect, *prog)
            } else {
                // No movement
                new_rect
            }
        } else {
            // First appearance - no animation
            new_rect
        };

        prev.insert(child_idx, new_rect);
        result
    }

    /// Tick the animation forward by `delta_time` seconds.
    pub fn tick(&self, delta_time: f32) {
        let mut prog = self.progress.borrow_mut();
        if *prog < 1.0 {
            *prog = (*prog + delta_time / self.duration).min(1.0);
        }
    }

    /// Get the current animation progress (0.0 to 1.0).
    pub fn progress(&self) -> f32 {
        *self.progress.borrow()
    }

    /// Check if the animation is complete.
    pub fn is_complete(&self) -> bool {
        *self.progress.borrow() >= 1.0
    }
}

fn lerp_rect(a: &cvkg_core::Rect, b: &cvkg_core::Rect, t: f32) -> cvkg_core::Rect {
    let t = t.clamp(0.0, 1.0);
    cvkg_core::Rect {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
        width: a.width + (b.width - a.width) * t,
        height: a.height + (b.height - a.height) * t,
    }
}
