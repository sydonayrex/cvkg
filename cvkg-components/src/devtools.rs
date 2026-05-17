use cvkg_core::{Never, Rect, Renderer, View};
use cvkg_vdom::VDom;
// No imports needed from crate for now

/// A HUD-style inspector for the Virtual DOM.
pub struct VdomInspector {
    vdom: std::sync::Arc<VDom>,
}

impl VdomInspector {
    /// Create a new VdomInspector.
    pub fn new(vdom: std::sync::Arc<VDom>) -> Self {
        Self { vdom }
    }
}

impl View for VdomInspector {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Render a semi-transparent panel on the right
        let panel_width = 300.0;
        let panel_rect = Rect {
            x: rect.x + rect.width - panel_width,
            y: rect.y,
            width: panel_width,
            height: rect.height,
        };

        // Bifrost background
        renderer.bifrost(panel_rect, 20.0, 0.5, 0.8);
        renderer.fill_rect(panel_rect, [0.05, 0.05, 0.05, 0.8]);
        renderer.stroke_rect(panel_rect, [0.0, 1.0, 1.0, 1.0], 1.0);

        // Title
        let title_rect = Rect {
            x: panel_rect.x + 10.0,
            y: panel_rect.y + 10.0,
            width: panel_width - 20.0,
            height: 30.0,
        };
        renderer.draw_text(
            "VDOM INSPECTOR",
            title_rect.x,
            title_rect.y,
            16.0,
            [0.0, 1.0, 1.0, 1.0],
        );

        // Hierarchy
        if let Some(root_id) = self.vdom.root {
            self.render_node(
                renderer,
                root_id,
                panel_rect.x + 10.0,
                panel_rect.y + 50.0,
                0,
            );
        }
    }
}

impl VdomInspector {
    fn render_node(
        &self,
        renderer: &mut dyn Renderer,
        id: cvkg_vdom::NodeId,
        x: f32,
        y: f32,
        depth: usize,
    ) -> f32 {
        let mut current_y = y;
        if let Some(node) = self.vdom.nodes.get(&id) {
            let indent = depth as f32 * 15.0;
            let node_label = format!("{} (ID: {})", node.component_type, id.0);

            renderer.draw_text(
                &node_label,
                x + indent,
                current_y,
                12.0,
                [1.0, 1.0, 1.0, 1.0],
            );
            current_y += 20.0;

            for child_id in &node.children {
                current_y = self.render_node(renderer, *child_id, x, current_y, depth + 1);
            }
        }
        current_y
    }
}

/// A HUD-style overlay for performance telemetry.
pub struct TelemetryOverlay;

impl View for TelemetryOverlay {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let telemetry = renderer.get_telemetry();

        let panel_width = 240.0;
        let panel_height = 280.0;
        let panel_rect = Rect {
            x: rect.x + 10.0,
            y: rect.y + 10.0,
            width: panel_width,
            height: panel_height,
        };

        // Bifrost background (frosted glass)
        renderer.bifrost(panel_rect, 15.0, 0.3, 0.7);
        renderer.fill_rect(panel_rect, [0.02, 0.02, 0.03, 0.8]);
        renderer.stroke_rect(panel_rect, [0.0, 0.8, 1.0, 0.5], 1.0);

        let mut y = panel_rect.y + 20.0;
        let x = panel_rect.x + 15.0;
        let line_h = 18.0;

        // Title
        renderer.draw_text("SYSTEM_TELEMETRY", x, y, 14.0, [0.0, 1.0, 0.9, 1.0]);
        y += 25.0;

        // Frame timing
        renderer.draw_text(
            &format!("FRAME: {:.2} ms", telemetry.frame_time_ms),
            x,
            y,
            12.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        y += line_h;

        // Pass timing breakdown
        renderer.draw_text(
            &format!("  INPUT:  {:.2} ms", telemetry.input_time_ms),
            x,
            y,
            10.0,
            [0.7, 0.7, 0.8, 1.0],
        );
        y += line_h;
        renderer.draw_text(
            &format!("  LAYOUT: {:.2} ms", telemetry.layout_time_ms),
            x,
            y,
            10.0,
            [0.7, 0.7, 0.8, 1.0],
        );
        y += line_h;
        renderer.draw_text(
            &format!("  STATE:  {:.2} ms", telemetry.state_flush_time_ms),
            x,
            y,
            10.0,
            [0.7, 0.7, 0.8, 1.0],
        );
        y += line_h;
        renderer.draw_text(
            &format!("  DRAW:   {:.2} ms", telemetry.draw_time_ms),
            x,
            y,
            10.0,
            [0.7, 0.7, 0.8, 1.0],
        );
        y += line_h;
        renderer.draw_text(
            &format!("  SUBMIT: {:.2} ms", telemetry.gpu_submit_time_ms),
            x,
            y,
            10.0,
            [0.7, 0.7, 0.8, 1.0],
        );
        y += 25.0;

        // GPU Stats
        renderer.draw_text("GPU_RESOURCES", x, y, 12.0, [1.0, 0.6, 0.0, 1.0]);
        y += 20.0;
        renderer.draw_text(
            &format!("VRAM_TOTAL: {:.2} MB", telemetry.vram_usage_mb),
            x,
            y,
            11.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        y += line_h;
        renderer.draw_text(
            &format!("  TEX: {:.2} MB", telemetry.vram_textures_mb),
            x,
            y,
            10.0,
            [0.6, 0.6, 0.7, 1.0],
        );
        y += line_h;
        renderer.draw_text(
            &format!("  BUF: {:.2} MB", telemetry.vram_buffers_mb),
            x,
            y,
            10.0,
            [0.6, 0.6, 0.7, 1.0],
        );
        y += 25.0;

        // Draw calls & Vertices
        renderer.draw_text(
            &format!("DRAW_CALLS: {}", telemetry.draw_calls),
            x,
            y,
            11.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        y += line_h;
        renderer.draw_text(
            &format!("VERTICES:   {}", telemetry.vertices),
            x,
            y,
            11.0,
            [1.0, 1.0, 1.0, 1.0],
        );
    }
}

/// A visual overlay showing layout constraints and boundaries.
pub struct ConstraintOverlay {
    /// Whether to show the overlay
    pub enabled: bool,
    /// Color for constraint lines (RGBA)
    pub constraint_color: [f32; 4],
    /// Color for padding areas (RGBA)
    pub padding_color: [f32; 4],
    /// Show margin visualization
    pub show_margins: bool,
    /// Show padding visualization
    pub show_padding: bool,
}

impl Default for ConstraintOverlay {
    fn default() -> Self {
        Self {
            enabled: true,
            constraint_color: [0.0, 1.0, 1.0, 0.8], // Cyan
            padding_color: [1.0, 0.0, 1.0, 0.3],    // Magenta (semi-transparent)
            show_margins: true,
            show_padding: true,
        }
    }
}

impl View for ConstraintOverlay {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.enabled {
            return;
        }

        // Draw main constraint boundary
        renderer.stroke_rect(rect, self.constraint_color, 1.0);

        // Draw corner markers for precise alignment
        let marker_size = 8.0;

        // Top-left
        renderer.fill_rect(
            Rect {
                x: rect.x - marker_size / 2.0,
                y: rect.y - marker_size / 2.0,
                width: marker_size,
                height: marker_size,
            },
            self.constraint_color,
        );

        // Top-right
        renderer.fill_rect(
            Rect {
                x: rect.x + rect.width - marker_size / 2.0,
                y: rect.y - marker_size / 2.0,
                width: marker_size,
                height: marker_size,
            },
            self.constraint_color,
        );

        // Bottom-left
        renderer.fill_rect(
            Rect {
                x: rect.x - marker_size / 2.0,
                y: rect.y + rect.height - marker_size / 2.0,
                width: marker_size,
                height: marker_size,
            },
            self.constraint_color,
        );

        // Bottom-right
        renderer.fill_rect(
            Rect {
                x: rect.x + rect.width - marker_size / 2.0,
                y: rect.y + rect.height - marker_size / 2.0,
                width: marker_size,
                height: marker_size,
            },
            self.constraint_color,
        );

        // Center crosshairs
        renderer.fill_rect(
            Rect {
                x: rect.x + rect.width / 2.0 - 0.5,
                y: rect.y,
                width: 1.0,
                height: rect.height,
            },
            [
                self.constraint_color[0],
                self.constraint_color[1],
                self.constraint_color[2],
                0.4,
            ],
        );
        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: rect.y + rect.height / 2.0 - 0.5,
                width: rect.width,
                height: 1.0,
            },
            [
                self.constraint_color[0],
                self.constraint_color[1],
                self.constraint_color[2],
                0.4,
            ],
        );
    }
}

impl ConstraintOverlay {
    /// Create a new constraint overlay with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable the overlay.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the constraint color.
    pub fn constraint_color(mut self, color: [f32; 4]) -> Self {
        self.constraint_color = color;
        self
    }

    /// Set the padding visualization color.
    pub fn padding_color(mut self, color: [f32; 4]) -> Self {
        self.padding_color = color;
        self
    }
}
