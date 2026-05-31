use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::Arc;

/// A radial (pie-segment) menu that displays items in a circle around a center point.
///
/// Each item occupies an angular sector. The menu expands from the center
/// when triggered and collapses when a selection is made or the user clicks away.
///
/// # Layout
///
/// Items are arranged clockwise starting from the 12 o'clock position.
/// Each item gets an equal angular slice (2π / item_count).
/// The inner radius creates a hollow center; the outer radius determines reach.
///
/// # Interaction
///
/// The application provides an `on_select` callback that receives the index
/// of the activated item. Hit-testing is performed via polar coordinates
/// (angle determines the sector, radius determines if inside the donut).
#[derive(Clone)]
pub struct RadialMenu {
    /// Menu items to display.
    pub items: Vec<RadialMenuItem>,
    /// Center position in screen coordinates.
    pub center: [f32; 2],
    /// Inner radius (hollow center) in pixels.
    pub inner_radius: f32,
    /// Outer radius (full reach) in pixels.
    pub outer_radius: f32,
    /// Index of the currently hovered item (None if none).
    pub hovered: Option<usize>,
    /// Whether the menu is currently visible.
    pub is_visible: bool,
    /// Callback fired when an item is selected.
    pub on_select: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    /// Callback fired when the menu is dismissed without selection.
    pub on_dismiss: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Angle offset in radians (default: 0 = first item at 12 o'clock).
    pub angle_offset: f32,
    /// Scale factor for the menu (useful for expand/collapse animation).
    pub scale: f32,
}

/// A single item in a radial menu.
#[derive(Clone)]
pub struct RadialMenuItem {
    /// Display label.
    pub label: String,
    /// Optional icon name (for rendering).
    pub icon: Option<String>,
    /// Whether this item is currently active/enabled.
    pub enabled: bool,
}

impl RadialMenuItem {
    /// Create a new radial menu item.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            enabled: true,
        }
    }

    /// Set the icon name.
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set whether the item is enabled.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Default for RadialMenu {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            center: [0.0, 0.0],
            inner_radius: 40.0,
            outer_radius: 120.0,
            hovered: None,
            is_visible: false,
            on_select: None,
            on_dismiss: None,
            angle_offset: -std::f32::consts::FRAC_PI_2, // Start from 12 o'clock
            scale: 1.0,
        }
    }
}

impl RadialMenu {
    /// Create a radial menu with the given items.
    pub fn new(items: Vec<RadialMenuItem>) -> Self {
        Self {
            items,
            ..Default::default()
        }
    }

    /// Set the center position.
    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.center = [x, y];
        self
    }

    /// Set the radius range.
    pub fn with_radius(mut self, inner: f32, outer: f32) -> Self {
        self.inner_radius = inner;
        self.outer_radius = outer;
        self
    }

    /// Set the selection callback.
    pub fn on_select(mut self, cb: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(cb));
        self
    }

    /// Set the dismiss callback.
    pub fn on_dismiss(mut self, cb: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_dismiss = Some(Arc::new(cb));
        self
    }

    /// Show the menu.
    pub fn show(&mut self) {
        self.is_visible = true;
    }

    /// Hide the menu.
    pub fn hide(&mut self) {
        self.is_visible = false;
        self.hovered = None;
    }

    /// Determine which item a point hits, if any.
    ///
    /// Returns `Some(index)` if the point is within a sector, `None` otherwise.
    pub fn hit_test(&self, x: f32, y: f32) -> Option<usize> {
        if self.items.is_empty() {
            return None;
        }

        let dx = x - self.center[0];
        let dy = y - self.center[1];
        let dist_sq = dx * dx + dy * dy;
        let inner_r = self.inner_radius * self.scale;
        let outer_r = self.outer_radius * self.scale;

        if dist_sq < inner_r * inner_r || dist_sq > outer_r * outer_r {
            return None;
        }

        // Calculate angle (atan2 returns -π..π, 0 = east)
        let mut angle = dy.atan2(dx);
        // Offset so 0 = 12 o'clock (north), clockwise positive
        angle -= std::f32::consts::FRAC_PI_2;
        // Add user offset
        angle += self.angle_offset;

        // Normalize to 0..2π
        while angle < 0.0 {
            angle += std::f32::consts::TAU;
        }
        while angle >= std::f32::consts::TAU {
            angle -= std::f32::consts::TAU;
        }

        let sector_size = std::f32::consts::TAU / self.items.len() as f32;
        let index = (angle / sector_size) as usize;
        Some(index.min(self.items.len() - 1))
    }

    /// Get the angular range (start, end) for a given item index.
    pub fn sector_angles(&self, index: usize) -> (f32, f32) {
        if self.items.is_empty() {
            return (0.0, 0.0);
        }
        let sector = std::f32::consts::TAU / self.items.len() as f32;
        let start = self.angle_offset + index as f32 * sector + std::f32::consts::FRAC_PI_2;
        let end = start + sector;
        (start, end)
    }

    /// Get the number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl View for RadialMenu {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        if !self.is_visible || self.items.is_empty() {
            return;
        }

        let cx = self.center[0];
        let cy = self.center[1];
        let inner_r = self.inner_radius * self.scale;
        let outer_r = self.outer_radius * self.scale;
        let n = self.items.len();
        let sector = std::f32::consts::TAU / n as f32;

        // Background circle (dimmed overlay area)
        let bg_rect = Rect {
            x: cx - outer_r,
            y: cy - outer_r,
            width: outer_r * 2.0,
            height: outer_r * 2.0,
        };
        renderer.fill_ellipse(bg_rect, [0.08, 0.08, 0.12, 0.85]);

        // Draw each sector
        for (i, item) in self.items.iter().enumerate() {
            let (start_angle, end_angle) = self.sector_angles(i);
            let is_hovered = self.hovered == Some(i);
            let angle_mid = (start_angle + end_angle) * 0.5;

            // Sector background
            let sector_color = if is_hovered {
                if item.enabled {
                    [0.0, 0.7, 1.0, 0.6]
                } else {
                    [0.5, 0.2, 0.2, 0.4]
                }
            } else {
                if item.enabled {
                    [0.15, 0.15, 0.22, 0.7]
                } else {
                    [0.1, 0.1, 0.12, 0.5]
                }
            };

            // Draw sector as an arc segment using polygon approximation
            let steps = 12;
            let angle_step = sector / steps as f32;
            let mut vertices: Vec<[f32; 2]> = Vec::with_capacity(steps + 2);

            // Center point
            vertices.push([cx, cy]);

            // Outer arc
            for s in 0..=steps {
                let a = start_angle + angle_step * s as f32;
                vertices.push([cx + a.cos() * outer_r, cy + a.sin() * outer_r]);
            }

            renderer.fill_polygon(&vertices, sector_color);

            // Sector border
            let border_color = if is_hovered {
                [0.0, 0.9, 1.0, 0.9]
            } else {
                [0.25, 0.25, 0.35, 0.6]
            };
            // Draw radial lines
            let start_outer = [
                cx + start_angle.cos() * outer_r,
                cy + start_angle.sin() * outer_r,
            ];
            let start_inner = [
                cx + start_angle.cos() * inner_r,
                cy + start_angle.sin() * inner_r,
            ];
            renderer.draw_line(
                start_outer[0],
                start_outer[1],
                start_inner[0],
                start_inner[1],
                border_color,
                1.0,
            );

            let end_outer = [
                cx + end_angle.cos() * outer_r,
                cy + end_angle.sin() * outer_r,
            ];
            let end_inner = [
                cx + end_angle.cos() * inner_r,
                cy + end_angle.sin() * inner_r,
            ];
            renderer.draw_line(
                end_outer[0],
                end_outer[1],
                end_inner[0],
                end_inner[1],
                border_color,
                1.0,
            );

            // Item label at midpoint of sector
            let label_r = (inner_r + outer_r) * 0.5;
            let label_x = cx + angle_mid.cos() * label_r;
            let label_y = cy + angle_mid.sin() * label_r;

            let label_color = if item.enabled {
                if is_hovered {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    [0.85, 0.85, 0.95, 1.0]
                }
            } else {
                [0.4, 0.4, 0.5, 0.6]
            };

            // Simple centering: estimate text width
            let char_w = 7.0;
            let text_w = item.label.len() as f32 * char_w;
            renderer.draw_text(
                &item.label,
                label_x - text_w * 0.5,
                label_y + 5.0,
                12.0,
                label_color,
            );
        }

        // Inner circle (hollow center)
        let center_circle = Rect {
            x: cx - inner_r,
            y: cy - inner_r,
            width: inner_r * 2.0,
            height: inner_r * 2.0,
        };
        renderer.fill_ellipse(center_circle, [0.12, 0.12, 0.16, 0.8]);
        renderer.stroke_ellipse(center_circle, [0.25, 0.25, 0.35, 0.5], 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radial_menu_hit_test() {
        let items = vec![
            RadialMenuItem::new("Cut"),
            RadialMenuItem::new("Copy"),
            RadialMenuItem::new("Paste"),
            RadialMenuItem::new("Delete"),
        ];
        let menu = RadialMenu::new(items)
            .at(200.0, 200.0)
            .with_radius(40.0, 120.0);

        // Center should miss (inside inner radius)
        assert_eq!(menu.hit_test(200.0, 200.0), None);

        // Far away should miss
        assert_eq!(menu.hit_test(500.0, 500.0), None);

        // Point in the first sector (roughly at 12 o'clock)
        let hit = menu.hit_test(200.0, 100.0);
        assert!(hit.is_some());
    }

    #[test]
    fn test_radial_menu_visibility() {
        let mut menu = RadialMenu::new(vec![RadialMenuItem::new("Test")]);
        assert!(!menu.is_visible);
        menu.show();
        assert!(menu.is_visible);
        menu.hide();
        assert!(!menu.is_visible);
    }

    #[test]
    fn test_sector_angles() {
        let items = vec![
            RadialMenuItem::new("A"),
            RadialMenuItem::new("B"),
            RadialMenuItem::new("C"),
            RadialMenuItem::new("D"),
        ];
        let menu = RadialMenu::new(items);

        let (start, end) = menu.sector_angles(0);
        let sector_size = end - start;
        assert!((sector_size - std::f32::consts::FRAC_PI_2).abs() < 0.001);
    }

    #[test]
    fn test_empty_menu() {
        let menu = RadialMenu::new(vec![]);
        assert_eq!(menu.hit_test(200.0, 200.0), None);
        assert!(menu.is_empty());
    }

    #[test]
    fn test_disabled_item_hit_test() {
        let items = vec![
            RadialMenuItem::new("Enabled"),
            RadialMenuItem::new("Disabled").with_enabled(false),
        ];
        let menu = RadialMenu::new(items)
            .at(100.0, 100.0)
            .with_radius(20.0, 60.0);

        // Hit test returns the index; rendering handles the disabled visual
        let hit = menu.hit_test(100.0, 50.0);
        assert!(hit.is_some());
    }
}
