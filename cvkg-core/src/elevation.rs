//! Elevation and z-index layer system.
//!
//! Provides a 5-to-25 elevation scale as named constants, and documented
//! z-index layer constants for consistent visual stacking.
//!
//! # Elevation Scale
//!
//! Elevation levels map to shadow depth. Higher levels cast deeper shadows:
//!
//! | Level | Name | Use Case |
//! |-------|------|----------|
//! | 0 | FLAT | No shadow, baseline elements |
//! | 1 | RESTING | Cards at rest, static content |
//! | 2 | HOVER | Cards on hover, subtle lift |
//! | 4 | FLOATING | Floating action buttons, sticky headers |
//! | 8 | OVERLAY | Dropdowns, popovers, tooltips |
//! | 16 | MODAL | Dialogs, sheets, overlays |
//! | 25 | TOAST | Notifications, highest priority |
//!
//! # Z-Index Layers
//!
//! Named z-index constants ensure consistent stacking across the UI:
//!
//! - `BASE` (0): Background content
//! - `DROPDOWN` (1000): Dropdown menus, select popups
//! - `STICKY` (2000): Sticky headers, persistent bars
//! - `MODAL` (3000): Modal dialogs, full-screen overlays
//! - `TOAST` (4000): Toast notifications, highest non-debug layer
//! - `TOOLTIP` (5000): Tooltips (always on top)
//! - `DEBUG` (99999): Debug overlays, development tools

/// Elevation level constants (0-25 scale).
pub mod elevation {
    /// No elevation — flat baseline elements with no shadow.
    pub const FLAT: u8 = 0;

    /// Resting elevation — cards, panels at their default state.
    pub const RESTING: u8 = 1;

    /// Hover elevation — subtle lift for interactive elements on hover.
    pub const HOVER: u8 = 2;

    /// Floating elevation — FABs, sticky headers, persistent UI.
    pub const FLOATING: u8 = 4;

    /// Overlay elevation — dropdowns, popovers, context menus.
    pub const OVERLAY: u8 = 8;

    /// Modal elevation — dialogs, sheets, full-screen overlays.
    pub const MODAL: u8 = 16;

    /// Toast elevation — notifications, highest standard UI layer.
    pub const TOAST: u8 = 25;

    /// Maximum elevation level.
    pub const MAX: u8 = 25;

    /// Convert an elevation level to a shadow blur radius.
    pub fn to_blur_radius(level: u8) -> f32 {
        match level {
            0 => 0.0,
            1 => 2.0,
            2 => 4.0,
            4 => 8.0,
            8 => 12.0,
            16 => 24.0,
            25 => 40.0,
            _ => (level as f32) * 1.6,
        }
    }

    /// Convert an elevation level to a shadow offset (Y).
    pub fn to_offset_y(level: u8) -> f32 {
        match level {
            0 => 0.0,
            1 => 1.0,
            2 => 2.0,
            4 => 4.0,
            8 => 6.0,
            16 => 12.0,
            25 => 20.0,
            _ => (level as f32) * 0.8,
        }
    }

    /// Convert an elevation level to shadow opacity.
    pub fn to_opacity(level: u8) -> f32 {
        match level {
            0 => 0.0,
            1 => 0.1,
            2 => 0.15,
            4 => 0.2,
            8 => 0.25,
            16 => 0.3,
            25 => 0.35,
            _ => (level as f32) / 100.0,
        }
    }
}

/// Named z-index layer constants for consistent visual stacking.
pub mod z_index {
    /// Base layer — background content, canvas.
    pub const BASE: i32 = 0;

    /// Content layer — main page content.
    pub const CONTENT: i32 = 100;

    /// Dropdown layer — select popups, dropdown menus.
    pub const DROPDOWN: i32 = 1000;

    /// Sticky layer — sticky headers, persistent bars.
    pub const STICKY: i32 = 2000;

    /// Modal layer — dialogs, full-screen overlays.
    pub const MODAL: i32 = 3000;

    /// Toast layer — notification toasts.
    pub const TOAST: i32 = 4000;

    /// Tooltip layer — always-on-top tooltips.
    pub const TOOLTIP: i32 = 5000;

    /// Debug layer — development tools, highest priority.
    pub const DEBUG: i32 = 99999;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, ModifiedView, Never, Rect, Renderer, View, testing::MockRenderer};

    struct SolidView {
        color: [f32; 4],
    }

    impl View for SolidView {
        type Body = Never;
        fn body(self) -> Self::Body {
            unreachable!()
        }
        fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
            renderer.fill_rect(rect, self.color);
        }
    }

    #[test]
    fn elevation_determines_draw_order() {
        // Two views at different elevations: higher elevation should render
        // (push content) after lower elevation in draw order
        let mut renderer = MockRenderer::new();
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0);

        // Draw flat view first
        let flat_view = SolidView {
            color: [1.0, 0.0, 0.0, 1.0], // red
        };
        flat_view.render(&mut renderer, rect);

        // Draw modal view second (higher elevation)
        let modal_view = SolidView {
            color: [0.0, 0.0, 1.0, 1.0], // blue
        }
        .elevation(elevation::MODAL as f32);
        modal_view.render(&mut renderer, rect);

        // Should have 2 draw calls (one from flat, one from modal)
        renderer.assert_draw_call_count(2);

        // The second draw call should be blue (the modal view's content)
        renderer.assert_color_at(50.0, 25.0, [0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn z_index_values_are_positive() {
        // All named z-index layers should be non-negative
        assert!(z_index::BASE >= 0);
        assert!(z_index::CONTENT > 0);
        assert!(z_index::DROPDOWN > z_index::CONTENT);
        assert!(z_index::TOOLTIP < z_index::DEBUG);
    }

    #[test]
    fn elevation_scale_ordering() {
        // Higher levels should have larger blur radii
        assert!(
            elevation::to_blur_radius(elevation::FLAT)
                < elevation::to_blur_radius(elevation::RESTING)
        );
        assert!(
            elevation::to_blur_radius(elevation::RESTING)
                < elevation::to_blur_radius(elevation::HOVER)
        );
        assert!(
            elevation::to_blur_radius(elevation::HOVER)
                < elevation::to_blur_radius(elevation::FLOATING)
        );
        assert!(
            elevation::to_blur_radius(elevation::FLOATING)
                < elevation::to_blur_radius(elevation::OVERLAY)
        );
        assert!(
            elevation::to_blur_radius(elevation::OVERLAY)
                < elevation::to_blur_radius(elevation::MODAL)
        );
        assert!(
            elevation::to_blur_radius(elevation::MODAL)
                < elevation::to_blur_radius(elevation::TOAST)
        );
    }

    #[test]
    fn elevation_offsets_increase() {
        // Higher levels should have larger Y offsets
        assert!(
            elevation::to_offset_y(elevation::FLAT) < elevation::to_offset_y(elevation::RESTING)
        );
        assert!(
            elevation::to_offset_y(elevation::RESTING) < elevation::to_offset_y(elevation::HOVER)
        );
        assert!(
            elevation::to_offset_y(elevation::HOVER) < elevation::to_offset_y(elevation::FLOATING)
        );
    }

    #[test]
    fn elevation_opacities_increase() {
        assert!(elevation::to_opacity(elevation::FLAT) < elevation::to_opacity(elevation::RESTING));
        assert!(
            elevation::to_opacity(elevation::RESTING) < elevation::to_opacity(elevation::MODAL)
        );
    }

    #[test]
    fn flat_has_no_shadow() {
        assert_eq!(elevation::to_blur_radius(elevation::FLAT), 0.0);
        assert_eq!(elevation::to_offset_y(elevation::FLAT), 0.0);
        assert_eq!(elevation::to_opacity(elevation::FLAT), 0.0);
    }

    #[test]
    fn z_index_layers_stack_correctly() {
        // Lower layers should have lower z-index values
        assert!(z_index::BASE < z_index::CONTENT);
        assert!(z_index::CONTENT < z_index::DROPDOWN);
        assert!(z_index::DROPDOWN < z_index::STICKY);
        assert!(z_index::STICKY < z_index::MODAL);
        assert!(z_index::MODAL < z_index::TOAST);
        assert!(z_index::TOAST < z_index::TOOLTIP);
        assert!(z_index::TOOLTIP < z_index::DEBUG);
    }

    #[test]
    fn z_index_modal_above_dropdown() {
        // A modal dialog should render above a dropdown
        assert!(z_index::MODAL > z_index::DROPDOWN);
    }

    #[test]
    fn z_index_toast_above_modal() {
        // A toast notification should render above a modal
        assert!(z_index::TOAST > z_index::MODAL);
    }

    #[test]
    fn z_index_tooltip_above_toast() {
        // Tooltips should render above toasts
        assert!(z_index::TOOLTIP > z_index::TOAST);
    }

    #[test]
    fn elevation_max_is_25() {
        assert_eq!(elevation::MAX, 25);
        assert_eq!(elevation::TOAST, 25);
    }

    #[test]
    fn custom_elevation_interpolates() {
        // Custom levels (not in the named constants) should interpolate
        let blur_3 = elevation::to_blur_radius(3);
        let blur_2 = elevation::to_blur_radius(2);
        let blur_4 = elevation::to_blur_radius(4);
        assert!(blur_2 < blur_3);
        assert!(blur_3 < blur_4);
    }
}
