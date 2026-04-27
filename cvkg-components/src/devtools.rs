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
        renderer.draw_text(&format!("FRAME: {:.2} ms", telemetry.frame_time_ms), x, y, 12.0, [1.0, 1.0, 1.0, 1.0]);
        y += line_h;
        
        // Pass timing breakdown
        renderer.draw_text(&format!("  INPUT:  {:.2} ms", telemetry.input_time_ms), x, y, 10.0, [0.7, 0.7, 0.8, 1.0]);
        y += line_h;
        renderer.draw_text(&format!("  LAYOUT: {:.2} ms", telemetry.layout_time_ms), x, y, 10.0, [0.7, 0.7, 0.8, 1.0]);
        y += line_h;
        renderer.draw_text(&format!("  STATE:  {:.2} ms", telemetry.state_flush_time_ms), x, y, 10.0, [0.7, 0.7, 0.8, 1.0]);
        y += line_h;
        renderer.draw_text(&format!("  DRAW:   {:.2} ms", telemetry.draw_time_ms), x, y, 10.0, [0.7, 0.7, 0.8, 1.0]);
        y += line_h;
        renderer.draw_text(&format!("  SUBMIT: {:.2} ms", telemetry.gpu_submit_time_ms), x, y, 10.0, [0.7, 0.7, 0.8, 1.0]);
        y += 25.0;

        // GPU Stats
        renderer.draw_text("GPU_RESOURCES", x, y, 12.0, [1.0, 0.6, 0.0, 1.0]);
        y += 20.0;
        renderer.draw_text(&format!("VRAM_TOTAL: {:.2} MB", telemetry.vram_usage_mb), x, y, 11.0, [1.0, 1.0, 1.0, 1.0]);
        y += line_h;
        renderer.draw_text(&format!("  TEX: {:.2} MB", telemetry.vram_textures_mb), x, y, 10.0, [0.6, 0.6, 0.7, 1.0]);
        y += line_h;
        renderer.draw_text(&format!("  BUF: {:.2} MB", telemetry.vram_buffers_mb), x, y, 10.0, [0.6, 0.6, 0.7, 1.0]);
        y += 25.0;

        // Draw calls & Vertices
        renderer.draw_text(&format!("DRAW_CALLS: {}", telemetry.draw_calls), x, y, 11.0, [1.0, 1.0, 1.0, 1.0]);
        y += line_h;
        renderer.draw_text(&format!("VERTICES:   {}", telemetry.vertices), x, y, 11.0, [1.0, 1.0, 1.0, 1.0]);
    }
}
