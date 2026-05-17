use crate::types::NodeId;
use serde::{Deserialize, Serialize};

/// Cubic Bezier easing parameters for edge animation.
///
/// Maps a normalized life value (1.0 → 0.0) to visual properties
/// like color, width, and opacity using a cubic bezier curve.
/// This enables elastic, spring-like edge transitions instead of
/// linear interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SplineEasing {
    /// Control point X1 (time axis). Range: 0.0–1.0.
    pub x1: f32,
    /// Control point Y1 (value axis). Range: 0.0–1.0.
    pub y1: f32,
    /// Control point X2 (time axis). Range: 0.0–1.0.
    pub x2: f32,
    /// Control point Y2 (value axis). Range: 0.0–1.0.
    pub y2: f32,
}

impl SplineEasing {
    /// Creates a new cubic bezier easing with the given control points.
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self {
            x1: x1.clamp(0.0, 1.0),
            y1: y1.clamp(0.0, 1.0),
            x2: x2.clamp(0.0, 1.0),
            y2: y2.clamp(0.0, 1.0),
        }
    }

    /// Linear easing (no curve).
    pub fn linear() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0)
    }

    /// Ease-in-out curve for smooth edge transitions.
    pub fn ease_in_out() -> Self {
        Self::new(0.25, 0.1, 0.25, 1.0)
    }

    /// Elastic burst curve for edge spawn animation.
    pub fn elastic() -> Self {
        Self::new(0.68, -0.55, 0.27, 1.55)
    }

    /// Evaluates the bezier easing at the given parameter t.
    ///
    /// Uses Newton-Raphson iteration to solve for the X component,
    /// then evaluates the Y component of the cubic bezier.
    pub fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        // Solve for t given x using Newton-Raphson
        let mut guess = t;
        for _ in 0..8 {
            let x = self.bezier_x(guess) - t;
            let dx = self.bezier_x_deriv(guess);
            if dx.abs() < 1e-6 {
                break;
            }
            guess -= x / dx;
            guess = guess.clamp(0.0, 1.0);
        }

        self.bezier_y(guess)
    }

    fn bezier_x(&self, t: f32) -> f32 {
        let mt = 1.0 - t;
        3.0 * mt * mt * t * self.x1 + 3.0 * mt * t * t * self.x2 + t * t * t
    }

    fn bezier_y(&self, t: f32) -> f32 {
        let mt = 1.0 - t;
        3.0 * mt * mt * t * self.y1 + 3.0 * mt * t * t * self.y2 + t * t * t
    }

    fn bezier_x_deriv(&self, t: f32) -> f32 {
        let mt = 1.0 - t;
        3.0 * mt * mt * self.x1 + 6.0 * mt * t * (self.x2 - self.x1) + 3.0 * t * t * (1.0 - self.x2)
    }
}

impl Default for SplineEasing {
    fn default() -> Self {
        Self::ease_in_out()
    }
}

/// Interaction state for a flow edge.
///
/// Tracks hover, selection, and animated visual properties
/// for GPU-instanced ribbon rendering.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EdgeInteraction {
    /// Edge is in its default state.
    Default,
    /// Pointer is hovering over the edge.
    Hovered,
    /// Edge is selected (e.g., for inspection or deletion).
    Selected,
    /// Edge is being dragged (e.g., reconnecting).
    Dragging,
}

/// A connection between two flow nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEdge {
    pub id: u64,
    pub source_node: NodeId,
    pub source_port_idx: usize,
    pub target_node: NodeId,
    pub target_port_idx: usize,
    /// RGBA color of the edge ribbon.
    pub color: [f32; 4],
    /// Base width of the edge ribbon in pixels.
    pub width: f32,
    /// Current interaction state.
    pub interaction: EdgeInteraction,
    /// Hover highlight color (OKLCH-derived RGBA).
    pub hover_color: [f32; 4],
    /// Width multiplier applied on hover.
    pub hover_width_multiplier: f32,
    /// Spline easing for edge spawn animation.
    pub spawn_easing: SplineEasing,
    /// Spline easing for edge color transitions.
    pub color_easing: SplineEasing,
    /// Current animation progress (0.0 = just spawned, 1.0 = fully settled).
    pub animation_progress: f32,
    /// Whether the edge is currently visible.
    pub visible: bool,
}

impl FlowEdge {
    /// Creates a new flow edge between two nodes.
    pub fn new(
        id: u64,
        source_node: NodeId,
        source_port_idx: usize,
        target_node: NodeId,
        target_port_idx: usize,
    ) -> Self {
        Self {
            id,
            source_node,
            source_port_idx,
            target_node,
            target_port_idx,
            color: [0.4, 0.45, 0.55, 0.7],
            width: 2.0,
            interaction: EdgeInteraction::Default,
            hover_color: [0.3, 0.7, 1.0, 0.9],
            hover_width_multiplier: 1.5,
            spawn_easing: SplineEasing::elastic(),
            color_easing: SplineEasing::ease_in_out(),
            animation_progress: 1.0,
            visible: true,
        }
    }

    /// Sets the edge color.
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Sets the edge width.
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width.max(0.5);
        self
    }

    /// Sets the hover highlight color.
    pub fn with_hover_color(mut self, color: [f32; 4]) -> Self {
        self.hover_color = color;
        self
    }

    /// Sets the hover width multiplier.
    pub fn with_hover_width(mut self, multiplier: f32) -> Self {
        self.hover_width_multiplier = multiplier.max(1.0);
        self
    }

    /// Sets the spawn animation easing.
    pub fn with_spawn_easing(mut self, easing: SplineEasing) -> Self {
        self.spawn_easing = easing;
        self
    }

    /// Sets the color transition easing.
    pub fn with_color_easing(mut self, easing: SplineEasing) -> Self {
        self.color_easing = easing;
        self
    }

    /// Returns true if the edge is currently hovered.
    pub fn is_hovered(&self) -> bool {
        self.interaction == EdgeInteraction::Hovered
    }

    /// Returns true if the edge is selected.
    pub fn is_selected(&self) -> bool {
        self.interaction == EdgeInteraction::Selected
    }

    /// Sets the interaction state to hovered.
    pub fn set_hovered(&mut self) {
        self.interaction = EdgeInteraction::Hovered;
    }

    /// Sets the interaction state to selected.
    pub fn set_selected(&mut self) {
        self.interaction = EdgeInteraction::Selected;
    }

    /// Resets the interaction state to default.
    pub fn set_default(&mut self) {
        self.interaction = EdgeInteraction::Default;
    }

    /// Returns the effective color based on interaction state and animation.
    ///
    /// Uses the color easing spline to smoothly transition between
    /// default and hover/selected colors.
    pub fn effective_color(&self) -> [f32; 4] {
        let target = match self.interaction {
            EdgeInteraction::Default => self.color,
            EdgeInteraction::Hovered => self.hover_color,
            EdgeInteraction::Selected => [0.2, 0.8, 1.0, 1.0],
            EdgeInteraction::Dragging => [1.0, 0.6, 0.2, 0.8],
        };

        let t = self.color_easing.evaluate(self.animation_progress);
        [
            self.color[0] + (target[0] - self.color[0]) * t,
            self.color[1] + (target[1] - self.color[1]) * t,
            self.color[2] + (target[2] - self.color[2]) * t,
            self.color[3] + (target[3] - self.color[3]) * t,
        ]
    }

    /// Returns the effective width based on interaction state and animation.
    ///
    /// Uses the spawn easing spline for elastic width animation on spawn,
    /// and applies the hover width multiplier when hovered.
    pub fn effective_width(&self) -> f32 {
        let spawn_t = self.spawn_easing.evaluate(self.animation_progress);
        let base = self.width * spawn_t;
        if self.is_hovered() || self.is_selected() {
            base * self.hover_width_multiplier
        } else {
            base
        }
    }

    /// Advances the animation by the given delta time.
    ///
    /// Returns true if the animation is still in progress.
    pub fn tick_animation(&mut self, dt: f32) -> bool {
        if self.animation_progress < 1.0 {
            self.animation_progress = (self.animation_progress + dt * 2.0).min(1.0);
            true
        } else {
            false
        }
    }

    /// Restarts the spawn animation (e.g., when the edge is recreated).
    pub fn restart_animation(&mut self) {
        self.animation_progress = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_creation() {
        let edge = FlowEdge::new(1, NodeId(10), 0, NodeId(20), 0);
        assert_eq!(edge.id, 1);
        assert_eq!(edge.source_node, NodeId(10));
        assert_eq!(edge.target_node, NodeId(20));
        assert_eq!(edge.width, 2.0);
        assert!(edge.visible);
        assert_eq!(edge.interaction, EdgeInteraction::Default);
    }

    #[test]
    fn test_edge_hover() {
        let mut edge = FlowEdge::new(1, NodeId(10), 0, NodeId(20), 0);
        assert!(!edge.is_hovered());
        edge.set_hovered();
        assert!(edge.is_hovered());
        assert_eq!(edge.interaction, EdgeInteraction::Hovered);
    }

    #[test]
    fn test_edge_selected() {
        let mut edge = FlowEdge::new(1, NodeId(10), 0, NodeId(20), 0);
        assert!(!edge.is_selected());
        edge.set_selected();
        assert!(edge.is_selected());
    }

    #[test]
    fn test_edge_default() {
        let mut edge = FlowEdge::new(1, NodeId(10), 0, NodeId(20), 0);
        edge.set_hovered();
        edge.set_default();
        assert_eq!(edge.interaction, EdgeInteraction::Default);
    }

    #[test]
    fn test_edge_effective_width() {
        let mut edge = FlowEdge::new(1, NodeId(10), 0, NodeId(20), 0);
        edge.animation_progress = 1.0;
        assert_eq!(edge.effective_width(), 2.0);

        edge.set_hovered();
        assert_eq!(edge.effective_width(), 3.0); // 2.0 * 1.5
    }

    #[test]
    fn test_edge_animation_tick() {
        let mut edge = FlowEdge::new(1, NodeId(10), 0, NodeId(20), 0);
        edge.restart_animation();
        assert_eq!(edge.animation_progress, 0.0);

        let still_animating = edge.tick_animation(0.1);
        assert!(still_animating);
        assert!(edge.animation_progress > 0.0);

        // Tick to completion
        for _ in 0..20 {
            edge.tick_animation(0.1);
        }
        assert_eq!(edge.animation_progress, 1.0);
        assert!(!edge.tick_animation(0.1));
    }

    #[test]
    fn test_edge_with_builder() {
        let edge = FlowEdge::new(1, NodeId(10), 0, NodeId(20), 0)
            .with_color([1.0, 0.0, 0.0, 1.0])
            .with_width(4.0)
            .with_hover_color([0.0, 1.0, 0.0, 1.0])
            .with_hover_width(2.0)
            .with_spawn_easing(SplineEasing::elastic());

        assert_eq!(edge.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(edge.width, 4.0);
        assert_eq!(edge.hover_color, [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(edge.hover_width_multiplier, 2.0);
    }

    #[test]
    fn test_spline_easing_linear() {
        let e = SplineEasing::linear();
        assert!((e.evaluate(0.0) - 0.0).abs() < 0.01);
        assert!((e.evaluate(0.5) - 0.5).abs() < 0.05);
        assert!((e.evaluate(1.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_spline_easing_evaluate() {
        let e = SplineEasing::ease_in_out();
        let mid = e.evaluate(0.5);
        assert!(mid > 0.0 && mid < 1.0);
    }

    #[test]
    fn test_edge_effective_color() {
        let mut edge = FlowEdge::new(1, NodeId(10), 0, NodeId(20), 0);
        edge.animation_progress = 1.0;
        let default_color = edge.effective_color();
        assert_eq!(default_color, edge.color);

        edge.set_hovered();
        let hover_color = edge.effective_color();
        assert_ne!(hover_color, default_color);
    }
}
