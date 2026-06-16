//! HeimdallDock — macOS-style dock with magnification and auto-hide.
//! Named after Heimdall, guardian of the Bifrost bridge.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::Arc;

/// Compute magnified size for a dock item based on pointer proximity.
/// Uses a Gaussian envelope centered on the pointer with σ = 80px.
/// Maximum magnification: 2.0× at zero distance.
pub fn dock_item_magnification(
    item_center: f32,
    pointer_x: f32,
    _base_size: f32,
    max_scale: f32,
) -> f32 {
    let sigma = 80.0_f32;
    let dist = (item_center - pointer_x).abs();
    let gaussian = (-dist * dist / (2.0 * sigma * sigma)).exp();
    1.0 + (max_scale - 1.0) * gaussian
}

/// A single item in the dock.
#[derive(Clone)]
pub struct DockItem {
    pub id: String,
    pub label: String,
    pub badge: Option<u32>,
    pub is_running: bool,
    pub on_click: Arc<dyn Fn() + Send + Sync>,
}

impl DockItem {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        on_click: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            badge: None,
            is_running: false,
            on_click: Arc::new(on_click),
        }
    }

    pub fn badge(mut self, count: u32) -> Self {
        self.badge = Some(count);
        self
    }

    pub fn running(mut self, running: bool) -> Self {
        self.is_running = running;
        self
    }
}

/// macOS-style dock with magnification, auto-hide, and bounce animations.
pub struct HeimdallDock {
    pub items: Vec<DockItem>,
    pub position: DockPosition,
    pub auto_hide: bool,
    pub magnification: f32,
    pointer_x: f32,
    pointer_in_dock: bool,
}

pub enum DockPosition {
    Bottom,
    Left,
    Right,
}

impl HeimdallDock {
    pub fn new(items: Vec<DockItem>) -> Self {
        Self {
            items,
            position: DockPosition::Bottom,
            auto_hide: false,
            magnification: 2.0,
            pointer_x: 0.0,
            pointer_in_dock: false,
        }
    }

    pub fn position(mut self, pos: DockPosition) -> Self {
        self.position = pos;
        self
    }

    pub fn auto_hide(mut self, auto_hide: bool) -> Self {
        self.auto_hide = auto_hide;
        self
    }

    pub fn handle_pointer_move(&mut self, x: f32, y: f32) {
        self.pointer_x = x;
        // Track whether pointer is in dock area for auto-hide
        self.pointer_in_dock = y > 0.0; // Simplified
    }

    /// Get the platter rect for the current position.
    fn platter_rect(&self, rect: Rect) -> Rect {
        match self.position {
            DockPosition::Bottom => Rect {
                x: rect.x + 20.0,
                y: rect.y + rect.height - 68.0,
                width: rect.width - 40.0,
                height: 56.0,
            },
            DockPosition::Left => Rect {
                x: rect.x + 8.0,
                y: rect.y + 40.0,
                width: 56.0,
                height: rect.height - 80.0,
            },
            DockPosition::Right => Rect {
                x: rect.x + rect.width - 64.0,
                y: rect.y + 40.0,
                width: 56.0,
                height: rect.height - 80.0,
            },
        }
    }
}

impl View for HeimdallDock {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let platter_rect = self.platter_rect(rect);

        // Glass platter background
        renderer.bifrost(platter_rect, 25.0, 1.2, 0.7);
        renderer.fill_rounded_rect(platter_rect, 16.0, theme::with_alpha(theme::surface_elevated(), 0.85));

        // Render each item with magnification
        let base_size = 48.0;
        let center_y = platter_rect.y + platter_rect.height / 2.0;
        let total_width = self.items.len() as f32 * (base_size + 8.0);
        let start_x = platter_rect.x + (platter_rect.width - total_width) / 2.0 + base_size / 2.0;

        for (i, item) in self.items.iter().enumerate() {
            let item_center = start_x + i as f32 * (base_size + 8.0);
            let scale =
                dock_item_magnification(item_center, self.pointer_x, base_size, self.magnification);
            let scaled_size = base_size * scale;

            let item_rect = Rect {
                x: item_center - scaled_size / 2.0,
                y: center_y - scaled_size / 2.0,
                width: scaled_size,
                height: scaled_size,
            };

            // Icon background (rounded rect)
            renderer.fill_rounded_rect(item_rect, 12.0, theme::with_alpha(theme::surface_elevated(), 0.9));

            // Running indicator dot
            if item.is_running {
                let dot_rect = Rect {
                    x: item_rect.x + item_rect.width / 2.0 - 2.0,
                    y: item_rect.y + item_rect.height + 4.0,
                    width: 4.0,
                    height: 4.0,
                };
                let accent = theme::accent();
                renderer.fill_ellipse(dot_rect, theme::accent());
            }

            // Badge
            if let Some(count) = item.badge {
                let badge_rect = Rect {
                    x: item_rect.x + item_rect.width - 10.0,
                    y: item_rect.y - 2.0,
                    width: 14.0,
                    height: 14.0,
                };
                renderer.fill_ellipse(badge_rect, theme::with_alpha(theme::error_color(), 0.9));
                let text = if count > 99 {
                    "99+".to_string()
                } else {
                    count.to_string()
                };
                renderer.draw_text(
                    &text,
                    badge_rect.x + 2.0,
                    badge_rect.y + 3.0,
                    9.0,
                    theme::text(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dock_magnification_at_zero_distance() {
        let scale = dock_item_magnification(100.0, 100.0, 48.0, 2.0);
        assert!(
            (scale - 2.0).abs() < 0.01,
            "At zero distance, scale should be 2.0, got {}",
            scale
        );
    }

    #[test]
    fn test_dock_magnification_at_far_distance() {
        let scale = dock_item_magnification(100.0, 500.0, 48.0, 2.0);
        assert!(
            (scale - 1.0).abs() < 0.05,
            "At far distance, scale should approach 1.0, got {}",
            scale
        );
    }

    #[test]
    fn test_dock_magnification_symmetry() {
        let left = dock_item_magnification(100.0, 50.0, 48.0, 2.0);
        let right = dock_item_magnification(100.0, 150.0, 48.0, 2.0);
        assert!(
            (left - right).abs() < 0.001,
            "Magnification should be symmetric: left={}, right={}",
            left,
            right
        );
    }

    #[test]
    fn test_dock_magnification_mid_range() {
        // At sigma distance (80px), scale should be about 1.0 + (2.0-1.0)*e^(-0.5) ≈ 1.606
        let scale = dock_item_magnification(100.0, 180.0, 48.0, 2.0);
        let expected = 1.0 + (-0.5f32).exp();
        assert!(
            (scale - expected).abs() < 0.01,
            "At sigma distance, scale should be ~1.606, got {}",
            scale
        );
    }

    #[test]
    fn test_dock_item_new() {
        let item = DockItem::new("test", "Test", || {});
        assert_eq!(item.id, "test");
        assert_eq!(item.label, "Test");
        assert_eq!(item.badge, None);
        assert!(!item.is_running);
    }

    #[test]
    fn test_dock_item_badge() {
        let item = DockItem::new("test", "Test", || {}).badge(5);
        assert_eq!(item.badge, Some(5));
    }

    #[test]
    fn test_dock_item_running() {
        let item = DockItem::new("test", "Test", || {}).running(true);
        assert!(item.is_running);
    }

    #[test]
    fn test_dock_default_position() {
        let dock = HeimdallDock::new(vec![]);
        assert!(matches!(dock.position, DockPosition::Bottom));
    }

    #[test]
    fn test_dock_custom_position() {
        let dock = HeimdallDock::new(vec![]).position(DockPosition::Left);
        assert!(matches!(dock.position, DockPosition::Left));
    }

    #[test]
    fn test_dock_auto_hide() {
        let dock = HeimdallDock::new(vec![]).auto_hide(true);
        assert!(dock.auto_hide);
    }

    #[test]
    fn test_dock_pointer_move() {
        let mut dock = HeimdallDock::new(vec![]);
        dock.handle_pointer_move(150.0, 200.0);
        assert_eq!(dock.pointer_x, 150.0);
        assert!(dock.pointer_in_dock);
    }
}
