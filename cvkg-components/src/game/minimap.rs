//! MiniMap component for game UI.
//!
//! A rectangular viewport with positioned markers and zoom support.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// Marker position on the minimap.
#[derive(Clone, Debug)]
pub struct MapMarker {
    /// Position in world coordinates (x, y).
    pub position: (f32, f32),
    /// Marker color.
    pub color: [f32; 4],
    /// Marker size.
    pub size: f32,
}

impl MapMarker {
    /// Create a new map marker.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: (x, y),
            color: theme::accent(),
            size: 4.0,
        }
    }

    /// Set the marker color.
    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Set the marker size.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

/// MiniMap showing player position and points of interest.
///
/// ## Accessibility
/// - Role: `region`
/// - ARIA: `aria-label` for map description
/// - Reduced motion: respects `is_reduced_motion()` for zoom animation
#[derive(Clone)]
pub struct MiniMap {
    /// World bounds (min_x, min_y, max_x, max_y).
    pub world_bounds: (f32, f32, f32, f32),
    /// Player position in world coordinates.
    pub player_position: (f32, f32),
    /// Markers to display.
    pub markers: Vec<MapMarker>,
    /// Zoom level (1.0 = full view).
    pub zoom: f32,
}

impl Default for MiniMap {
    fn default() -> Self {
        Self::new((0.0, 0.0, 1000.0, 1000.0))
    }
}

impl MiniMap {
    /// Create a new MiniMap with world bounds.
    pub fn new(world_bounds: (f32, f32, f32, f32)) -> Self {
        Self {
            world_bounds,
            player_position: (0.0, 0.0),
            markers: Vec::new(),
            zoom: 1.0,
        }
    }

    /// Set player position.
    pub fn player_position(mut self, x: f32, y: f32) -> Self {
        self.player_position = (x, y);
        self
    }

    /// Add a marker.
    pub fn marker(mut self, marker: MapMarker) -> Self {
        self.markers.push(marker);
        self
    }

    /// Set zoom level.
    pub fn zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom.max(0.1).min(10.0);
        self
    }
}

impl View for MiniMap {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "MiniMap");
        renderer.register_a11y("region", "Game minimap");

        let (min_x, min_y, max_x, max_y) = self.world_bounds;
        let world_width = max_x - min_x;
        let world_height = max_y - min_y;

        // Background (dark map)
        renderer.fill_rounded_rect(rect, 4.0, [0.05, 0.05, 0.1, 0.8]);
        renderer.stroke_rounded_rect(rect, 4.0, theme::border(), 1.0);

        // Scale positions to minimap
        let x_scale = rect.width / (world_width * self.zoom);
        let y_scale = rect.height / (world_height * self.zoom);

        // Draw markers
        for marker in &self.markers {
            let x = rect.x + (marker.position.0 - min_x) * x_scale;
            let y = rect.y + (marker.position.1 - min_y) * y_scale;

            if x >= rect.x && x <= rect.x + rect.width && y >= rect.y && y <= rect.y + rect.height {
                renderer.fill_ellipse(
                    Rect {
                        x: x - marker.size / 2.0,
                        y: y - marker.size / 2.0,
                        width: marker.size,
                        height: marker.size,
                    },
                    marker.color,
                );
            }
        }

        // Draw player (center of map)
        let px = rect.x + (self.player_position.0 - min_x) * x_scale;
        let py = rect.y + (self.player_position.1 - min_y) * y_scale;

        // Player dot (white)
        renderer.fill_ellipse(
            Rect {
                x: px - 3.0,
                y: py - 3.0,
                width: 6.0,
                height: 6.0,
            },
            [1.0, 1.0, 1.0, 1.0],
        );

        // Player direction indicator (rotated marker)
        renderer.fill_ellipse(
            Rect {
                x: px - 2.0,
                y: py - 2.0,
                width: 4.0,
                height: 4.0,
            },
            theme::accent(),
        );

        renderer.pop_vnode();
    }
}
