use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// SkollProgress indicator component.
#[derive(Clone)]
pub struct SkollProgress {
    pub(crate) value: f32,
    pub(crate) max: f32,
    pub(crate) variant: ProgressVariant,
}

impl SkollProgress {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            max: 100.0,
            variant: ProgressVariant::Linear,
        }
    }

    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        self
    }

    pub fn variant(mut self, variant: ProgressVariant) -> Self {
        self.variant = variant;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressVariant {
    Linear,
    Circular,
}

impl View for SkollProgress {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        let pct = if self.max > 0.0 {
            (self.value / self.max).clamp(0.0, 1.0)
        } else {
            0.0
        };

        match self.variant {
            ProgressVariant::Linear => {
                let track_h = rect.height.min(8.0);
                let track_y = rect.y + (rect.height - track_h) / 2.0;

                // Track
                renderer.fill_rounded_rect(
                    Rect {
                        x: rect.x,
                        y: track_y,
                        width: rect.width,
                        height: track_h,
                    },
                    track_h / 2.0,
                    theme::surface_elevated(),
                );

                // Fill
                renderer.fill_rounded_rect(
                    Rect {
                        x: rect.x,
                        y: track_y,
                        width: rect.width * pct,
                        height: track_h,
                    },
                    track_h / 2.0,
                    theme::accent(),
                );
            }
            ProgressVariant::Circular => {
                let dim = rect.width.min(rect.height);
                let circ_rect = Rect {
                    x: rect.x + (rect.width - dim) / 2.0,
                    y: rect.y + (rect.height - dim) / 2.0,
                    width: dim,
                    height: dim,
                };

                // Background ring
                renderer.stroke_ellipse(circ_rect, theme::surface(), 4.0);

                // SkollProgress ring (simulated with smaller ellipse for now)
                let inset = 4.0;
                let progress_rect = Rect {
                    x: circ_rect.x + inset,
                    y: circ_rect.y + inset,
                    width: (circ_rect.width - 2.0 * inset) * pct,
                    height: circ_rect.height - 2.0 * inset,
                };
                renderer.stroke_ellipse(progress_rect, theme::progress_fill(), 4.0);
            }
        }
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn cvkg_core::Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        match self.variant {
            ProgressVariant::Linear => cvkg_core::Size {
                width: proposal.width.unwrap_or(100.0),
                height: 12.0,
            },
            ProgressVariant::Circular => cvkg_core::Size {
                width: 40.0,
                height: 40.0,
            },
        }
    }
}

/// A horizontal status bar for system indicators
#[derive(Clone)]
pub struct StatusBar {
    pub text: String,
    pub color: [f32; 4],
}

impl StatusBar {
    pub fn new(text: impl Into<String>, color: [f32; 4]) -> Self {
        Self {
            text: text.into(),
            color,
        }
    }
}

impl View for StatusBar {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.fill_rect(rect, theme::surface());
        renderer.draw_line(
            rect.x,
            rect.y,
            rect.x + rect.width,
            rect.y,
            theme::border_strong(),
            1.0,
        );

        renderer.draw_text(
            &self.text,
            rect.x + 10.0,
            rect.y + (rect.height - 12.0) / 2.0,
            12.0,
            self.color,
        );
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn cvkg_core::Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let (tw, th) = renderer.measure_text(&self.text, 12.0);
        cvkg_core::Size {
            width: proposal.width.unwrap_or(tw + 40.0),
            height: th + 8.0,
        }
    }
}

/// Chart types for tactical visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartType {
    Line,
    Scatter,
    Bar,
    Radar,
}

/// ValkyrieAnalytics - A tactical chart for monitoring mission data.
/// Named after the Valkyries, who monitor and choose the course of battle.
#[derive(Clone)]
pub struct ValkyrieAnalytics {
    pub chart_type: ChartType,
    pub data: Vec<f32>,
    pub color: [f32; 4],
}

impl ValkyrieAnalytics {
    /// Creates a new ValkyrieAnalytics with the given type and data.
    pub fn new(chart_type: ChartType, data: Vec<f32>) -> Self {
        Self {
            chart_type,
            data,
            color: theme::progress_fill(),
        }
    }

    /// Sets the color of the chart elements.
    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

impl View for ValkyrieAnalytics {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ValkyrieAnalytics");

        renderer.set_aria_role("status");
        if self.data.is_empty() {
            renderer.pop_vnode();
            return;
        }

        match self.chart_type {
            ChartType::Line => {
                let dx = rect.width / (self.data.len() - 1) as f32;
                for i in 0..self.data.len() - 1 {
                    let x1 = rect.x + i as f32 * dx;
                    let x2 = rect.x + (i + 1) as f32 * dx;
                    let y1 = rect.y + rect.height * (1.0 - self.data[i]);
                    let y2 = rect.y + rect.height * (1.0 - self.data[i + 1]);
                    renderer.draw_line(x1, y1, x2, y2, self.color, 2.0);
                }
            }
            ChartType::Scatter => {
                for (i, &val) in self.data.iter().enumerate() {
                    let dx = rect.width / self.data.len() as f32;
                    let x = rect.x + i as f32 * dx;
                    let y = rect.y + rect.height * (1.0 - val);
                    renderer.fill_rounded_rect(
                        Rect {
                            x: x - 3.0,
                            y: y - 3.0,
                            width: 6.0,
                            height: 6.0,
                        },
                        3.0,
                        self.color,
                    );
                }
            }
            ChartType::Bar => {
                let dx = rect.width / self.data.len() as f32;
                let spacing = 2.0;
                for (i, &val) in self.data.iter().enumerate() {
                    let h = rect.height * val;
                    renderer.fill_rect(
                        Rect {
                            x: rect.x + i as f32 * dx + spacing,
                            y: rect.y + rect.height - h,
                            width: dx - 2.0 * spacing,
                            height: h,
                        },
                        self.color,
                    );
                }
            }
            ChartType::Radar => {
                if self.data.len() < 3 {
                    renderer.pop_vnode();
                    return;
                }

                let center_x = rect.x + rect.width / 2.0;
                let center_y = rect.y + rect.height / 2.0;
                let max_radius = rect.width.min(rect.height) / 2.0;

                let num_axes = self.data.len();
                for i in 0..num_axes {
                    let angle = (i as f32 / num_axes as f32) * 2.0 * std::f32::consts::PI
                        - std::f32::consts::FRAC_PI_2;
                    let x = center_x + angle.cos() * max_radius;
                    let y = center_y + angle.sin() * max_radius;
                    renderer.draw_line(center_x, center_y, x, y, theme::text_muted(), 1.0);
                }

                for i in 0..num_axes {
                    let next_i = (i + 1) % num_axes;
                    let angle1 = (i as f32 / num_axes as f32) * 2.0 * std::f32::consts::PI
                        - std::f32::consts::FRAC_PI_2;
                    let angle2 = (next_i as f32 / num_axes as f32) * 2.0 * std::f32::consts::PI
                        - std::f32::consts::FRAC_PI_2;

                    let r1 = self.data[i] * max_radius;
                    let r2 = self.data[next_i] * max_radius;

                    let x1 = center_x + angle1.cos() * r1;
                    let y1 = center_y + angle1.sin() * r1;
                    let x2 = center_x + angle2.cos() * r2;
                    let y2 = center_y + angle2.sin() * r2;

                    renderer.draw_line(x1, y1, x2, y2, self.color, 2.0);
                    renderer.fill_rounded_rect(
                        Rect {
                            x: x1 - 2.0,
                            y: y1 - 2.0,
                            width: 4.0,
                            height: 4.0,
                        },
                        2.0,
                        self.color,
                    );
                }
            }
        }
        renderer.pop_vnode();
    }
}

/// A real-time performance telemetry display with tactical aesthetics.
#[derive(Clone, Copy)]
pub struct TelemetryView;

impl View for TelemetryView {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if cvkg_core::load_system_state().realm == cvkg_core::Realm::Midgard {
            renderer.set_aria_role("status");
            renderer.fill_rounded_rect(rect, 4.0, theme::with_alpha(theme::bg(), 0.1));
            renderer.stroke_rect(rect, theme::surface(), 1.0);
            return;
        }

        let stats = renderer.get_telemetry();

        // Bifrost Glassmorphism
        renderer.bifrost(rect, 20.0, 1.2, 0.85);
        renderer.fill_rounded_rect(rect, 6.0, theme::with_alpha(theme::bg(), 0.6));

        let accent_cyan = theme::accent();
        let accent_gold = theme::viking_gold();
        let alert_red = theme::error_color();

        let border_color = if stats.hardware_stall_detected {
            alert_red
        } else {
            accent_cyan
        };
        renderer.stroke_rounded_rect(rect, 6.0, border_color, 1.5);

        // Tactical Header
        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: 20.0,
            },
            [border_color[0], border_color[1], border_color[2], 0.2],
        );
        renderer.draw_text(
            "KVASIR TELEMETRY",
            rect.x + 8.0,
            rect.y + 4.0,
            10.0,
            border_color,
        );

        let lines = [
            (
                "FPS",
                format!("{:.1}", 1000.0 / stats.frame_time_ms.max(0.1)),
            ),
            ("FRAME", format!("{:.2} ms", stats.frame_time_ms)),
            ("P99", format!("{:.2} ms", stats.p99_frame_time_ms)),
            ("JITTER", format!("{:.2} ms", stats.frame_jitter_ms)),
            ("DRAW", format!("{}", stats.draw_calls)),
            ("VERT", format!("{}", stats.vertices)),
        ];

        let start_y = rect.y + 28.0;
        for (i, (label, val)) in lines.iter().enumerate() {
            let y = start_y + i as f32 * 18.0;
            renderer.draw_text(label, rect.x + 8.0, y, 10.0, theme::text_muted());
            renderer.draw_text(val, rect.x + 60.0, y, 11.0, accent_gold);
        }

        if stats.hardware_stall_detected {
            renderer.fill_rounded_rect(
                Rect {
                    x: rect.x + 5.0,
                    y: rect.y + rect.height - 25.0,
                    width: rect.width - 10.0,
                    height: 20.0,
                },
                4.0,
                [alert_red[0], alert_red[1], alert_red[2], 0.2],
            );
            renderer.draw_text(
                "HARDWARE STALL DETECTED",
                rect.x + 12.0,
                rect.y + rect.height - 20.0,
                10.0,
                alert_red,
            );
        }

        // Dynamic Scanning Line (Simulated with elapsed time if available)
        // For now, just a static tactical divider
        renderer.draw_line(
            rect.x + 5.0,
            rect.y + 24.0,
            rect.x + rect.width - 5.0,
            rect.y + 24.0,
            theme::border(),
            1.0,
        );
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        _proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: 180.0,
            height: 160.0,
        }
    }
}

use cvkg_core::{MemoryLayer, TemporalEdge, TemporalNode};

/// MimirsWell - A dynamic, force-directed graph visualization for the Temporal Graph.
#[derive(Clone)]
pub struct MimirsWell {
    pub nodes: Vec<TemporalNode>,
    pub edges: Vec<TemporalEdge>,
}

impl MimirsWell {
    pub fn new(nodes: Vec<TemporalNode>, edges: Vec<TemporalEdge>) -> Self {
        Self { nodes, edges }
    }
}

impl View for MimirsWell {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();

        // 1. Draw Bifrost Paths (Edges)
        for edge in &self.edges {
            let (x1, y1) = self.get_node_pos(&edge.source, rect, t);
            let (x2, y2) = self.get_node_pos(&edge.target, rect, t);

            // Animated Glow-Path (Bifrost)
            let alpha = 0.2 + (t * 3.0).sin().abs() * 0.2;
            renderer.draw_line(x1, y1, x2, y2, theme::with_alpha(theme::magenta_liquid(), alpha), 1.0); // Magenta Liquid

            // Traveling Pulse
            let progress = (t * 0.5 + (edge.source.len() as f32)).fract();
            let px = x1 + (x2 - x1) * progress;
            let py = y1 + (y2 - y1) * progress;
            renderer.fill_ellipse(
                Rect {
                    x: px - 1.5,
                    y: py - 1.5,
                    width: 3.0,
                    height: 3.0,
                },
                theme::with_alpha(theme::text(), 0.6),
            );
        }

        // 2. Draw Nodes (Clipped-Corner / Tactical)
        for node in &self.nodes {
            let (nx, ny) = self.get_node_pos(&node.id, rect, t);

            let activity_pulse = (t * 4.0 + (node.weight * 10.0)).sin() * 0.15 + 0.85;
            let size = (10.0 + node.weight * 15.0) * activity_pulse;

            let mut color = match node.layer {
                MemoryLayer::Episodic => theme::accent(),  // Cyan
                MemoryLayer::Semantic => theme::viking_gold(), // Viking Gold
                MemoryLayer::Procedural => theme::magenta_liquid(), // Magenta Liquid
            };

            // Boost brightness based on pulse
            color[3] *= activity_pulse;

            // Draw Clipped-Corner Node
            self.draw_clipped_node(renderer, nx, ny, size, color);

            // Label for high-weight nodes
            if node.weight > 0.5 {
                renderer.draw_text(
                    &node.id,
                    nx + size / 2.0 + 4.0,
                    ny - 4.0,
                    9.0,
                    theme::with_alpha(theme::text(), 0.5 * activity_pulse),
                );
            }
        }
    }
}

impl MimirsWell {
    fn get_node_pos(&self, id: &str, rect: Rect, t: f32) -> (f32, f32) {
        let mut h = 0u32;
        for b in id.bytes() {
            h = h.wrapping_mul(31).wrapping_add(b as u32);
        }

        let fx = (h % 1000) as f32 / 1000.0;
        let fy = ((h / 1000) % 1000) as f32 / 1000.0;

        let dx = (t * 0.4 + fx * 20.0).sin() * 10.0;
        let dy = (t * 0.6 + fy * 20.0).cos() * 10.0;

        (
            rect.x + rect.width * 0.15 + rect.width * 0.7 * fx + dx,
            rect.y + rect.height * 0.15 + rect.height * 0.7 * fy + dy,
        )
    }

    fn draw_clipped_node(
        &self,
        renderer: &mut dyn Renderer,
        x: f32,
        y: f32,
        size: f32,
        color: [f32; 4],
    ) {
        let s = size / 2.0;
        let c = s * 0.4;

        let points = [
            (x - s + c, y - s),
            (x + s - c, y - s),
            (x + s, y - s + c),
            (x + s, y + s - c),
            (x + s - c, y + s),
            (x - s + c, y + s),
            (x - s, y + s - c),
            (x - s, y - s + c),
        ];

        let mut fill_color = color;
        fill_color[3] *= 0.15;
        renderer.fill_rect(
            Rect {
                x: x - s,
                y: y - s,
                width: size,
                height: size,
            },
            fill_color,
        );

        for i in 0..8 {
            let p1 = points[i];
            let p2 = points[(i + 1) % 8];
            renderer.draw_line(p1.0, p1.1, p2.0, p2.1, color, 1.2);
        }
    }
}

const RUNES: &[char] = &[
    'ᚠ', 'ᚢ', 'ᚦ', 'ᚨ', 'ᚱ', 'ᚲ', 'ᚷ', 'ᚹ', 'ᚺ', 'ᚾ', 'ᛁ', 'ᛃ', 'ᛇ', 'ᛈ', 'ᛉ', 'ᛊ', 'ᛏ', 'ᛒ', 'ᛖ',
    'ᛗ', 'ᛚ', 'ᛜ', 'ᛞ', 'ᛟ',
];

/// RuneScript - A text component that reveals itself with a runic "deciphering" animation.
/// Formerly ScanningText, renamed for Norse-themed tactical alignment.
#[derive(Clone)]
pub struct RuneScript {
    pub text: String,
    pub font_size: f32,
    pub color: [f32; 4],
    pub speed: f32, // Characters per second
}

impl RuneScript {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_size: 14.0,
            color: theme::progress_fill(), // Cyan
            speed: 20.0,
        }
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

impl View for RuneScript {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        let revealed_count = (t * self.speed) as usize;
        let mut display_text = String::new();

        let chars: Vec<char> = self.text.chars().collect();
        for i in 0..chars.len() {
            if i < revealed_count {
                display_text.push(chars[i]);
            } else if i < revealed_count + 4 {
                let rune_idx = ((t * 30.0 + i as f32) as usize) % RUNES.len();
                display_text.push(RUNES[rune_idx]);
            } else {
                break;
            }
        }

        if !display_text.is_empty() {
            renderer.draw_text(
                &display_text,
                rect.x,
                rect.y + self.font_size,
                self.font_size,
                self.color,
            );
        }
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        _proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let (w, h) = renderer.measure_text(&self.text, self.font_size);
        cvkg_core::Size {
            width: w,
            height: h,
        }
    }
}

/// SleipnirGait - A container that staggers the reveal of its children.
/// Named after Odin's 8-legged horse, known for its rapid and coordinated gait.
#[derive(Clone)]
pub struct SleipnirGait {
    pub children: Vec<cvkg_core::AnyView>,
    pub stagger_delay: f32, // Delay between child reveals in seconds
}

impl SleipnirGait {
    pub fn new(stagger_delay: f32) -> Self {
        Self {
            children: Vec::new(),
            stagger_delay,
        }
    }

    pub fn child<V: View + Clone + 'static>(mut self, view: V) -> Self {
        self.children.push(view.erase());
        self
    }
}

impl View for SleipnirGait {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        let child_height = rect.height / self.children.len().max(1) as f32;

        for (i, child) in self.children.iter().enumerate() {
            let start_time = i as f32 * self.stagger_delay;
            if t < start_time {
                continue;
            }

            // Apply reveal opacity based on how long since its start time
            let opacity = ((t - start_time) * 4.0).min(1.0);
            renderer.push_opacity(opacity);

            let child_rect = Rect {
                x: rect.x,
                y: rect.y + i as f32 * child_height,
                width: rect.width,
                height: child_height,
            };
            child.render(renderer, child_rect);

            renderer.pop_opacity();
        }
    }
}

/// VölvaScan - A container that renders "runic noise" before revealing its content.
/// Named after the Völva (seers) who saw through the veil of time.
#[derive(Clone)]
pub struct VölvaScan<V: View> {
    pub content: V,
    pub duration: f32,
}

impl<V: View> VölvaScan<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            duration: 1.5,
        }
    }
}

impl<V: View> View for VölvaScan<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();

        if t < self.duration {
            // Render Runic Noise
            let mut noise = String::new();
            let char_count = (rect.width * rect.height / 200.0) as usize;
            for i in 0..char_count {
                let rune_idx = ((t * 50.0 + i as f32) as usize) % RUNES.len();
                noise.push(RUNES[rune_idx]);
                if i % 10 == 0 {
                    noise.push('\n');
                }
            }
            renderer.draw_text(&noise, rect.x, rect.y + 10.0, 10.0, theme::with_alpha(theme::accent(), 0.4));
        } else {
            // Reveal Content
            let opacity = ((t - self.duration) * 2.0).min(1.0);
            renderer.push_opacity(opacity);
            self.content.render(renderer, rect);
            renderer.pop_opacity();
        }
    }
}
/// RunicTooltip - A contextual tooltip for providing hidden wisdom (information).
/// Named after the Runes, which encode secret knowledge.
#[derive(Clone)]
pub struct RunicTooltip<V: View> {
    pub content: V,
    pub text: String,
    pub is_visible: bool,
}

impl<V: View> RunicTooltip<V> {
    /// Creates a new RunicTooltip wrapping the given content.
    pub fn new(content: V, text: impl Into<String>) -> Self {
        Self {
            content,
            text: text.into(),
            is_visible: false,
        }
    }

    /// Sets whether the tooltip is visible.
    pub fn visible(mut self, visible: bool) -> Self {
        self.is_visible = visible;
        self
    }
}

impl<V: View> View for RunicTooltip<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "RunicTooltip");

        // 1. Render Base Content
        self.content.render(renderer, rect);

        // 2. Render Tooltip if visible
        if self.is_visible {
            let (tw, th) = renderer.measure_text(&self.text, 12.0);
            let tip_rect = Rect {
                x: rect.x + (rect.width - (tw + 16.0)) / 2.0,
                y: rect.y - th - 16.0,
                width: tw + 16.0,
                height: th + 8.0,
            };

            renderer.set_z_index(200.0);
            renderer.bifrost(tip_rect, 10.0, 1.2, 0.95);
            renderer.fill_rounded_rect(tip_rect, 4.0, theme::surface_overlay());
            renderer.stroke_rect(tip_rect, theme::with_alpha(theme::accent(), 0.6), 1.0);

            renderer.draw_text(
                &self.text,
                tip_rect.x + 8.0,
                tip_rect.y + 6.0,
                12.0,
                theme::text(),
            );
            renderer.set_z_index(0.0);
        }

        renderer.pop_vnode();
    }
}

/// MuninAvatar - A user representation component with status indicators.
/// Named after the hybrid concept of "form/image" (Eikona).
#[derive(Clone)]
pub struct MuninAvatar {
    pub src: Option<String>,
    pub fallback: String,
    pub status: Option<AvatarStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarStatus {
    Online,
    Offline,
    Busy,
    Away,
}

impl MuninAvatar {
    /// Creates a new MuninAvatar.
    pub fn new(fallback: impl Into<String>) -> Self {
        Self {
            src: None,
            fallback: fallback.into(),
            status: None,
        }
    }

    /// Sets the image source for the avatar.
    pub fn src(mut self, src: impl Into<String>) -> Self {
        self.src = Some(src.into());
        self
    }

    /// Sets the online status indicator.
    pub fn status(mut self, status: AvatarStatus) -> Self {
        self.status = Some(status);
        self
    }
}

impl View for MuninAvatar {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "MuninAvatar");

        // 1. Base Circle
        renderer.fill_ellipse(rect, theme::surface());
        renderer.stroke_ellipse(rect, theme::border(), 1.0);

        // 2. Content
        if let Some(src) = &self.src {
            renderer.draw_image(src, rect);
        } else {
            let (tw, _) = renderer.measure_text(&self.fallback, 14.0);
            renderer.draw_text(
                &self.fallback,
                rect.x + (rect.width - tw) / 2.0,
                rect.y + (rect.height - 14.0) / 2.0,
                14.0,
                theme::text(),
            );
        }

        // 3. Status Indicator
        if let Some(status) = &self.status {
            let status_size = rect.width * 0.25;
            let status_rect = Rect {
                x: rect.x + rect.width - status_size,
                y: rect.y + rect.height - status_size,
                width: status_size,
                height: status_size,
            };

            let color = match status {
                AvatarStatus::Online => theme::success(),
                AvatarStatus::Offline => theme::text_muted(),
                AvatarStatus::Busy => theme::error_color(),
                AvatarStatus::Away => theme::warning(),
            };

            renderer.fill_ellipse(status_rect, color);
            renderer.stroke_ellipse(status_rect, theme::bg(), 1.5);
        }

        renderer.pop_vnode();
    }
}

/// MerkiBadge - A status or count indicator component.
/// Named after Merki, the Norse word for mark or sign.
#[derive(Clone)]
pub struct MerkiBadge {
    pub text: String,
    pub color: [f32; 4],
}

impl MerkiBadge {
    /// Creates a new MerkiBadge.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: theme::accent(),
        }
    }

    /// Sets the color of the badge.
    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

impl View for MerkiBadge {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "MerkiBadge");

        let mut bg = self.color;
        bg[3] *= 0.2;

        renderer.fill_rounded_rect(rect, 4.0, bg);
        renderer.stroke_rounded_rect(rect, 4.0, self.color, 1.0);

        let (tw, _) = renderer.measure_text(&self.text, 10.0);
        renderer.draw_text(
            &self.text,
            rect.x + (rect.width - tw) / 2.0,
            rect.y + (rect.height - 10.0) / 2.0,
            10.0,
            theme::text(),
        );

        renderer.pop_vnode();
    }
}

/// UrdrTimeline - A chronological timeline of events (the past).
/// Named after Urdr, the Norn of the Past.
#[derive(Clone)]
pub struct UrdrTimeline {
    pub items: Vec<UrdrEvent>,
}

#[derive(Clone)]
pub struct UrdrEvent {
    pub title: String,
    pub timestamp: String,
    pub description: Option<String>,
}

impl UrdrTimeline {
    /// Creates a new UrdrTimeline.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Adds an event to the timeline.
    pub fn event(mut self, title: impl Into<String>, timestamp: impl Into<String>) -> Self {
        self.items.push(UrdrEvent {
            title: title.into(),
            timestamp: timestamp.into(),
            description: None,
        });
        self
    }
}

impl View for UrdrTimeline {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "UrdrTimeline");

        let t = renderer.elapsed_time();
        let line_x = rect.x + 20.0;
        renderer.draw_line(
            line_x,
            rect.y,
            line_x,
            rect.y + rect.height,
            theme::text_muted(),
            1.5,
        );

        let mut current_y = rect.y + 10.0;
        let item_spacing = 50.0;

        for (i, event) in self.items.iter().enumerate() {
            // 1. Bifrost Resonance (Glowing Temporal Nodes)
            let pulse = (t * 2.0 + i as f32).sin() * 0.2 + 0.8;
            renderer.fill_ellipse(
                Rect {
                    x: line_x - 5.0,
                    y: current_y - 1.0,
                    width: 10.0,
                    height: 10.0,
                },
                theme::with_alpha(theme::accent(), 0.3 * pulse),
            );
            renderer.fill_ellipse(
                Rect {
                    x: line_x - 3.0,
                    y: current_y + 1.0,
                    width: 6.0,
                    height: 6.0,
                },
                theme::progress_fill(),
            );

            // 2. Content
            renderer.draw_text(
                &event.timestamp,
                line_x + 20.0,
                current_y - 2.0,
                10.0,
                theme::text_muted(),
            );
            renderer.draw_text(
                &event.title,
                line_x + 20.0,
                current_y + 12.0,
                13.0,
                theme::text(),
            );

            current_y += item_spacing;
        }

        renderer.pop_vnode();
    }
}

/// DraumaSkeleton - A shimmering skeleton loader for async content.
/// Named after the dreams (Drauma) of content waiting to be born.
#[derive(Clone)]
pub struct DraumaSkeleton {
    pub border_radius: f32,
    pub shimmer: bool,
}

impl DraumaSkeleton {
    /// Creates a new DraumaSkeleton.
    pub fn new() -> Self {
        Self {
            border_radius: 4.0,
            shimmer: true,
        }
    }

    /// Sets the border radius of the skeleton.
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Enables or disables the shimmer effect.
    pub fn shimmer(mut self, enabled: bool) -> Self {
        self.shimmer = enabled;
        self
    }
}

impl View for DraumaSkeleton {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "DraumaSkeleton");

        renderer.set_aria_role("status");
        let t = renderer.elapsed_time();

        // 1. Mimir's Refraction (Skeletal Depth)
        // Drauma represents a "spectral" presence of content
        renderer.bifrost(rect, self.border_radius, 2.0, 0.8);
        renderer.fill_rounded_rect(rect, self.border_radius, theme::with_alpha(theme::surface(), 0.6));

        // 2. Kinetic Shimmer Effect
        if self.shimmer {
            let shimmer_pos = (t * 1.2).fract(); // Slower, more spectral shimmer
            let shimmer_w = rect.width * 0.5;
            let shimmer_rect = Rect {
                x: rect.x - shimmer_w + (rect.width + shimmer_w * 2.0) * shimmer_pos,
                y: rect.y,
                width: shimmer_w,
                height: rect.height,
            };

            let shimmer_alpha = 0.15 * (1.0 - (shimmer_pos - 0.5).abs() * 2.0);
            renderer.fill_rect(shimmer_rect, theme::with_alpha(theme::accent(), shimmer_alpha));
        }

        renderer.pop_vnode();
    }
}

// =============================================================================
// SPINNER — Animated loading indicator with multiple variants
// =============================================================================

/// HatiSpinner variant determining the visual style of the loading animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinnerVariant {
    /// A rotating ring with a gap (classic spinner).
    Circle,
    /// Three dots bouncing in sequence.
    Dots,
    /// A horizontal bar that pulses in width.
    PulseBar,
    /// Small 12px spinner for inline use (e.g., inside buttons).
    Inline,
}

/// An animated loading indicator component.
#[derive(Clone)]
pub struct HatiSpinner {
    pub(crate) variant: SpinnerVariant,
    pub(crate) size: f32,
    pub(crate) color: [f32; 4],
}

impl HatiSpinner {
    /// Creates a new HatiSpinner with the default Circle variant.
    pub fn new() -> Self {
        Self {
            variant: SpinnerVariant::Circle,
            size: 24.0,
            color: theme::accent(),
        }
    }

    /// Sets the spinner variant.
    pub fn variant(mut self, variant: SpinnerVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the spinner size.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Sets the spinner color.
    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

impl View for HatiSpinner {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();

        renderer.set_aria_role("progressbar");
        match self.variant {
            SpinnerVariant::Circle => {
                let dim = self.size;
                let circ_rect = Rect {
                    x: rect.x + (rect.width - dim) / 2.0,
                    y: rect.y + (rect.height - dim) / 2.0,
                    width: dim,
                    height: dim,
                };

                // Background ring (dim)
                let bg_color = [
                    self.color[0],
                    self.color[1],
                    self.color[2],
                    self.color[3] * 0.2,
                ];
                renderer.stroke_ellipse(circ_rect, bg_color, 2.5);

                // Rotating arc — simulated by drawing short line segments
                // along an arc with a trailing fade, creating the appearance
                // of a rotating spinner with a gap.
                let rotation = t * 2.5;
                let arc_frac = 0.65; // fraction of the circle that is visible
                let num_segments = 48;

                for i in 0..num_segments {
                    let frac = i as f32 / num_segments as f32;
                    if frac > arc_frac {
                        continue;
                    }
                    let angle = rotation + frac * 2.0 * std::f32::consts::PI;
                    let next_frac = (i + 1) as f32 / num_segments as f32;
                    let next_angle = rotation + next_frac * 2.0 * std::f32::consts::PI;

                    let cx = circ_rect.x + circ_rect.width / 2.0;
                    let cy = circ_rect.y + circ_rect.height / 2.0;
                    let r = circ_rect.width / 2.0;

                    let x1 = cx + angle.cos() * r;
                    let y1 = cy + angle.sin() * r;
                    let x2 = cx + next_angle.cos() * r;
                    let y2 = cy + next_angle.sin() * r;

                    // Fade the trailing edge
                    let alpha = self.color[3] * (1.0 - frac / arc_frac * 0.8).max(0.0);
                    let seg_color = [self.color[0], self.color[1], self.color[2], alpha];
                    renderer.draw_line(x1, y1, x2, y2, seg_color, 2.5);
                }
            }
            SpinnerVariant::Dots => {
                let dot_size = self.size * 0.25;
                let spacing = dot_size * 2.2;
                let total_width = spacing * 2.0;
                let start_x = rect.x + (rect.width - total_width) / 2.0;
                let base_y = rect.y + rect.height / 2.0;

                for i in 0..3 {
                    let offset = i as f32 * 0.6;
                    let bounce = (t * 4.0 + offset).sin();
                    let y = base_y + bounce * dot_size * 1.2;
                    let alpha = 0.4 + (bounce + 1.0) * 0.3; // 0.4–1.0

                    let dot_rect = Rect {
                        x: start_x + i as f32 * spacing - dot_size / 2.0,
                        y: y - dot_size / 2.0,
                        width: dot_size,
                        height: dot_size,
                    };

                    let dot_color = [
                        self.color[0],
                        self.color[1],
                        self.color[2],
                        self.color[3] * alpha,
                    ];
                    renderer.fill_ellipse(dot_rect, dot_color);
                }
            }
            SpinnerVariant::PulseBar => {
                let bar_h = self.size * 0.2;
                let max_w = self.size;
                let pulse = (t * 3.0).sin() * 0.5 + 0.5; // 0.0–1.0
                let bar_w = max_w * (0.3 + pulse * 0.7);
                let x = rect.x + (rect.width - bar_w) / 2.0;
                let y = rect.y + (rect.height - bar_h) / 2.0;

                let bar_rect = Rect {
                    x,
                    y,
                    width: bar_w,
                    height: bar_h,
                };

                renderer.fill_rounded_rect(bar_rect, bar_h / 2.0, self.color);
            }
            SpinnerVariant::Inline => {
                // Small 12px circle spinner
                let inline_size = 12.0;
                let dim = inline_size;
                let circ_rect = Rect {
                    x: rect.x + (rect.width - dim) / 2.0,
                    y: rect.y + (rect.height - dim) / 2.0,
                    width: dim,
                    height: dim,
                };

                let bg_color = [
                    self.color[0],
                    self.color[1],
                    self.color[2],
                    self.color[3] * 0.15,
                ];
                renderer.stroke_ellipse(circ_rect, bg_color, 1.5);

                let rotation = t * 3.0;
                let arc_frac = 0.5;
                let num_segments = 24;

                for i in 0..num_segments {
                    let frac = i as f32 / num_segments as f32;
                    if frac > arc_frac {
                        continue;
                    }
                    let angle = rotation + frac * 2.0 * std::f32::consts::PI;
                    let next_frac = (i + 1) as f32 / num_segments as f32;
                    let next_angle = rotation + next_frac * 2.0 * std::f32::consts::PI;

                    let cx = circ_rect.x + circ_rect.width / 2.0;
                    let cy = circ_rect.y + circ_rect.height / 2.0;
                    let r = circ_rect.width / 2.0;

                    let x1 = cx + angle.cos() * r;
                    let y1 = cy + angle.sin() * r;
                    let x2 = cx + next_angle.cos() * r;
                    let y2 = cy + next_angle.sin() * r;

                    let alpha = self.color[3] * (1.0 - frac / arc_frac * 0.8).max(0.0);
                    let seg_color = [self.color[0], self.color[1], self.color[2], alpha];
                    renderer.draw_line(x1, y1, x2, y2, seg_color, 1.5);
                }
            }
        }
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        match self.variant {
            SpinnerVariant::Inline => cvkg_core::Size {
                width: proposal.width.unwrap_or(12.0),
                height: 12.0,
            },
            _ => cvkg_core::Size {
                width: proposal.width.unwrap_or(self.size),
                height: self.size,
            },
        }
    }
}

// =============================================================================
// EMPTY STATE — Centered placeholder for empty content
// =============================================================================

/// A centered empty state component with icon, title, description, and optional CTA.
#[derive(Clone)]
pub struct EmptyState {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) icon: Option<String>,
    pub(crate) action_label: Option<String>,
    pub(crate) on_action: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
}

impl EmptyState {
    /// Creates a new EmptyState with the given title and description.
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            icon: None,
            action_label: None,
            on_action: None,
        }
    }

    /// Sets the icon (emoji or text) for the empty state.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets the CTA button label and action callback.
    pub fn action(
        mut self,
        label: impl Into<String>,
        on_action: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        self.action_label = Some(label.into());
        self.on_action = Some(std::sync::Arc::new(on_action));
        self
    }
}

impl View for EmptyState {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        use crate::{Button, ButtonVariant, FONT_BASE, FONT_LG, SPACE_LG, SPACE_SM};

        renderer.set_aria_role("status");
        let mut y = rect.y;
        let center_x = rect.x + rect.width / 2.0;

        // 1. Icon (48px text)
        if let Some(icon) = &self.icon {
            let (tw, th) = renderer.measure_text(icon, crate::FONT_3XL);
            renderer.draw_text(
                icon,
                center_x - tw / 2.0,
                y + (crate::FONT_3XL - th).max(0.0),
                crate::FONT_3XL,
                theme::text(),
            );
            y += crate::FONT_3XL + SPACE_SM;
        }

        // 2. Title (FONT_LG)
        let (tw, th) = renderer.measure_text(&self.title, FONT_LG);
        renderer.draw_text(
            &self.title,
            center_x - tw / 2.0,
            y + (FONT_LG - th).max(0.0),
            FONT_LG,
            theme::text(),
        );
        y += FONT_LG + SPACE_SM;

        // 3. Description (FONT_BASE, dimmed)
        let (tw, th) = renderer.measure_text(&self.description, FONT_BASE);
        renderer.draw_text(
            &self.description,
            center_x - tw / 2.0,
            y + (FONT_BASE - th).max(0.0),
            FONT_BASE,
            theme::text_muted(),
        );
        y += FONT_BASE + SPACE_LG;

        // 4. Optional CTA Button
        if let (Some(label), Some(on_action)) = (&self.action_label, &self.on_action) {
            let btn = Button::new(label.clone(), {
                let cb = on_action.clone();
                move || cb()
            })
            .variant(ButtonVariant::Default);

            let btn_rect = Rect {
                x: center_x - 80.0,
                y,
                width: 160.0,
                height: 36.0,
            };
            btn.render(renderer, btn_rect);
        }
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        use crate::{FONT_3XL, FONT_BASE, FONT_LG, SPACE_LG, SPACE_SM};

        let mut height = 0.0;

        if self.icon.is_some() {
            height += FONT_3XL + SPACE_SM;
        }

        let (_, title_h) = renderer.measure_text(&self.title, FONT_LG);
        height += title_h + SPACE_SM;

        let (_, desc_h) = renderer.measure_text(&self.description, FONT_BASE);
        height += desc_h + SPACE_LG;

        if self.action_label.is_some() {
            height += 36.0;
        }

        cvkg_core::Size {
            width: proposal.width.unwrap_or(300.0),
            height,
        }
    }
}

// --- HatiCarousel ---
use cvkg_core::Size;
use cvkg_core::layout::SizeProposal;
#[derive(Clone)]
pub struct HatiCarousel<V> {
    children: Vec<V>,
    current_index: usize,
}

impl<V: View> HatiCarousel<V> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            current_index: 0,
        }
    }

    pub fn child(mut self, child: V) -> Self {
        self.children.push(child);
        self
    }

    pub fn current_index(mut self, idx: usize) -> Self {
        self.current_index = idx;
        self
    }
}

impl<V: View> Default for HatiCarousel<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: View> View for HatiCarousel<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "HatiCarousel");
        // Clip children to bounds
        renderer.push_clip_rect(rect);

        if let Some(active_child) = self.children.get(self.current_index) {
            active_child.render(renderer, rect);
        }

        renderer.pop_clip_rect();
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let mut max_w = 0.0_f32;
        let mut max_h = 0.0_f32;
        for c in &self.children {
            let size = c.intrinsic_size(renderer, proposal);
            max_w = max_w.max(size.width);
            max_h = max_h.max(size.height);
        }
        Size {
            width: max_w,
            height: max_h,
        }
    }
}
