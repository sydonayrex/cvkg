use cvkg_core::{Never, Rect, Renderer, View};

/// Progress indicator component.
pub struct Progress {
    pub(crate) value: f32,
    pub(crate) max: f32,
    pub(crate) variant: ProgressVariant,
}

impl Progress {
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

pub enum ProgressVariant {
    Linear,
    Circular,
}

impl View for Progress {
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
                    Rect { x: rect.x, y: track_y, width: rect.width, height: track_h },
                    track_h / 2.0,
                    [0.15, 0.15, 0.2, 1.0],
                );
                
                // Fill
                renderer.fill_rounded_rect(
                    Rect { x: rect.x, y: track_y, width: rect.width * pct, height: track_h },
                    track_h / 2.0,
                    [0.0, 0.85, 1.0, 1.0],
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
                renderer.stroke_ellipse(circ_rect, [0.1, 0.1, 0.15, 1.0], 4.0);
                
                // Progress ring (simulated with smaller ellipse for now)
                let inset = 4.0;
                let progress_rect = Rect {
                    x: circ_rect.x + inset,
                    y: circ_rect.y + inset,
                    width: (circ_rect.width - 2.0 * inset) * pct,
                    height: circ_rect.height - 2.0 * inset,
                };
                renderer.stroke_ellipse(progress_rect, [0.0, 1.0, 1.0, 1.0], 4.0);
            }
        }
    }

    fn intrinsic_size(&self, _renderer: &mut dyn cvkg_core::Renderer, proposal: cvkg_core::SizeProposal) -> cvkg_core::Size {
        match self.variant {
            ProgressVariant::Linear => cvkg_core::Size {
                width: proposal.width.unwrap_or(100.0),
                height: 12.0,
            },
            ProgressVariant::Circular => cvkg_core::Size { width: 40.0, height: 40.0 },
        }
    }
}

/// Radial or linear gauge display
#[allow(dead_code)]
pub struct Gauge {
    pub(crate) value: f32,
    pub(crate) range: std::ops::RangeInclusive<f32>,
}

impl Gauge {
    pub fn new(value: f32, range: std::ops::RangeInclusive<f32>) -> Self {
        Self { value, range }
    }
}

impl View for Gauge {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.stroke_ellipse(rect, [0.15, 0.15, 0.2, 1.0], 6.0);
        let start = *self.range.start();
        let end = *self.range.end();
        let pct = if (end - start).abs() > f32::EPSILON {
            ((self.value - start) / (end - start)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let inset = 6.0;
        let inner = Rect {
            x: rect.x + inset,
            y: rect.y + inset,
            width: (rect.width - 2.0 * inset) * pct,
            height: rect.height - 2.0 * inset,
        };
        renderer.fill_ellipse(inner, [0.0, 0.85, 1.0, 1.0]);
    }

    fn intrinsic_size(&self, _renderer: &mut dyn cvkg_core::Renderer, proposal: cvkg_core::SizeProposal) -> cvkg_core::Size {
        let size = proposal.width.unwrap_or(100.0).min(proposal.height.unwrap_or(100.0));
        cvkg_core::Size { width: size, height: size }
    }
}

/// A horizontal status bar for system indicators
pub struct StatusBar {
    pub text: String,
    pub color: [f32; 4],
}

impl StatusBar {
    pub fn new(text: impl Into<String>, color: [f32; 4]) -> Self {
        Self { text: text.into(), color }
    }
}

impl View for StatusBar {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.fill_rect(rect, [0.05, 0.05, 0.08, 0.9]);
        renderer.draw_line(rect.x, rect.y, rect.x + rect.width, rect.y, [0.2, 0.2, 0.3, 1.0], 1.0);
        
        renderer.draw_text(&self.text, rect.x + 10.0, rect.y + (rect.height - 12.0) / 2.0, 12.0, self.color);
    }

    fn intrinsic_size(&self, renderer: &mut dyn cvkg_core::Renderer, proposal: cvkg_core::SizeProposal) -> cvkg_core::Size {
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

/// A tactical chart for monitoring mission data
pub struct ChartView {
    chart_type: ChartType,
    data: Vec<f32>,
    color: [f32; 4],
}

impl ChartView {
    pub fn new(chart_type: ChartType, data: Vec<f32>) -> Self {
        Self {
            chart_type,
            data,
            color: [0.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

impl View for ChartView {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.data.is_empty() {
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
                        Rect { x: x - 3.0, y: y - 3.0, width: 6.0, height: 6.0 },
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
                    renderer.draw_line(center_x, center_y, x, y, [0.3, 0.3, 0.4, 0.5], 1.0);
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
                        Rect { x: x1 - 2.0, y: y1 - 2.0, width: 4.0, height: 4.0 },
                        2.0,
                        self.color,
                    );
                }
            }
        }
    }
}

/// A real-time performance telemetry display with tactical aesthetics.
pub struct TelemetryView;

impl View for TelemetryView {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if cvkg_core::load_system_state().realm == cvkg_core::Realm::Midgard {
            renderer.fill_rounded_rect(rect, 4.0, [0.1, 0.12, 0.15, 0.1]);
            renderer.stroke_rect(rect, [0.2, 0.2, 0.2, 1.0], 1.0);
            return;
        }

        let stats = renderer.get_telemetry();
        
        // Bifrost Glassmorphism
        renderer.bifrost(rect, 20.0, 1.2, 0.85);
        renderer.fill_rounded_rect(rect, 6.0, [0.02, 0.03, 0.05, 0.6]);
        
        let accent_cyan = [0.0, 1.0, 1.0, 0.9];
        let accent_gold = [1.0, 0.8, 0.0, 0.9];
        let alert_red = [1.0, 0.2, 0.2, 1.0];
        
        let border_color = if stats.hardware_stall_detected { alert_red } else { accent_cyan };
        renderer.stroke_rounded_rect(rect, 6.0, border_color, 1.5);
        
        // Tactical Header
        renderer.fill_rect(
            Rect { x: rect.x, y: rect.y, width: rect.width, height: 20.0 },
            [border_color[0], border_color[1], border_color[2], 0.2]
        );
        renderer.draw_text("KVASIR TELEMETRY", rect.x + 8.0, rect.y + 4.0, 10.0, border_color);

let lines = vec![
            ("FPS", format!("{:.1}", 1000.0 / stats.frame_time_ms.max(0.1))),
            ("FRAME", format!("{:.2} ms", stats.frame_time_ms)),
            ("P99", format!("{:.2} ms", stats.p99_frame_time_ms)),
            ("JITTER", format!("{:.2} ms", stats.frame_jitter_ms)),
            ("DRAW", format!("{}", stats.draw_calls)),
            ("VERT", format!("{}", stats.vertices)),
        ];

        let start_y = rect.y + 28.0;
        for (i, (label, val)) in lines.iter().enumerate() {
            let y = start_y + i as f32 * 18.0;
            renderer.draw_text(label, rect.x + 8.0, y, 10.0, [0.7, 0.7, 0.8, 0.8]);
            renderer.draw_text(val, rect.x + 60.0, y, 11.0, accent_gold);
        }

        if stats.hardware_stall_detected {
            renderer.fill_rounded_rect(
                Rect { x: rect.x + 5.0, y: rect.y + rect.height - 25.0, width: rect.width - 10.0, height: 20.0 },
                4.0,
                [alert_red[0], alert_red[1], alert_red[2], 0.2]
            );
            renderer.draw_text("HARDWARE STALL DETECTED", rect.x + 12.0, rect.y + rect.height - 20.0, 10.0, alert_red);
        }
        
        // Dynamic Scanning Line (Simulated with elapsed time if available)
        // For now, just a static tactical divider
        renderer.draw_line(rect.x + 5.0, rect.y + 24.0, rect.x + rect.width - 5.0, rect.y + 24.0, [1.0, 1.0, 1.0, 0.1], 1.0);
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: cvkg_core::SizeProposal) -> cvkg_core::Size {
        cvkg_core::Size { width: 180.0, height: 160.0 }
    }
}

use cvkg_core::{TemporalNode, TemporalEdge, MemoryLayer};

/// MimirsWell - A dynamic, force-directed graph visualization for the Temporal Graph.
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
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        
        // 1. Draw Bifrost Paths (Edges)
        for edge in &self.edges {
            let (x1, y1) = self.get_node_pos(&edge.source, rect, t);
            let (x2, y2) = self.get_node_pos(&edge.target, rect, t);
            
            // Animated Glow-Path (Bifrost)
            let alpha = 0.2 + (t * 3.0).sin().abs() * 0.2;
            renderer.draw_line(x1, y1, x2, y2, [1.0, 0.0, 1.0, alpha], 1.0); // Magenta Liquid
            
            // Traveling Pulse
            let progress = (t * 0.5 + (edge.source.len() as f32)).fract();
            let px = x1 + (x2 - x1) * progress;
            let py = y1 + (y2 - y1) * progress;
            renderer.fill_ellipse(Rect { x: px - 1.5, y: py - 1.5, width: 3.0, height: 3.0 }, [1.0, 1.0, 1.0, 0.6]);
        }
        
        // 2. Draw Nodes (Clipped-Corner / Tactical)
        for node in &self.nodes {
            let (nx, ny) = self.get_node_pos(&node.id, rect, t);
            
            // Neural Activity Pulsing (Inspired by VYOM)
            let activity_pulse = (t * 4.0 + (node.weight * 10.0)).sin() * 0.15 + 0.85;
            let size = (10.0 + node.weight * 15.0) * activity_pulse;
            
            let mut color = match node.layer {
                MemoryLayer::Episodic => [0.0, 0.8, 1.0, 0.9], // Cyan
                MemoryLayer::Semantic => [1.0, 0.84, 0.0, 0.9], // Viking Gold
                MemoryLayer::Procedural => [1.0, 0.0, 1.0, 0.9], // Magenta Liquid
            };
            
            // Boost brightness based on pulse
            color[3] *= activity_pulse;
            
            // Draw Clipped-Corner Node
            self.draw_clipped_node(renderer, nx, ny, size, color);
            
            // Label for high-weight nodes
            if node.weight > 0.5 {
                renderer.draw_text(&node.id, nx + size / 2.0 + 4.0, ny - 4.0, 9.0, [1.0, 1.0, 1.0, 0.5 * activity_pulse]);
            }
        }
    }
}

impl MimirsWell {
    fn get_node_pos(&self, id: &str, rect: Rect, t: f32) -> (f32, f32) {
        let mut h = 0u32;
        for b in id.bytes() { h = h.wrapping_mul(31).wrapping_add(b as u32); }
        
        let fx = (h % 1000) as f32 / 1000.0;
        let fy = ((h / 1000) % 1000) as f32 / 1000.0;
        
        let dx = (t * 0.4 + fx * 20.0).sin() * 10.0;
        let dy = (t * 0.6 + fy * 20.0).cos() * 10.0;
        
        (
            rect.x + rect.width * 0.15 + rect.width * 0.7 * fx + dx,
            rect.y + rect.height * 0.15 + rect.height * 0.7 * fy + dy
        )
    }

    fn draw_clipped_node(&self, renderer: &mut dyn Renderer, x: f32, y: f32, size: f32, color: [f32; 4]) {
        let s = size / 2.0;
        let c = s * 0.4;
        
        let points = [
            (x - s + c, y - s), (x + s - c, y - s),
            (x + s, y - s + c), (x + s, y + s - c),
            (x + s - c, y + s), (x - s + c, y + s),
            (x - s, y + s - c), (x - s, y - s + c),
        ];
        
        let mut fill_color = color;
        fill_color[3] *= 0.15;
        renderer.fill_rect(Rect { x: x - s, y: y - s, width: size, height: size }, fill_color);
        
        for i in 0..8 {
            let p1 = points[i];
            let p2 = points[(i + 1) % 8];
            renderer.draw_line(p1.0, p1.1, p2.0, p2.1, color, 1.2);
        }
    }
}

const RUNES: &[char] = &[
    'ᚠ', 'ᚢ', 'ᚦ', 'ᚨ', 'ᚱ', 'ᚲ', 'ᚷ', 'ᚹ', 'ᚺ', 'ᚾ', 'ᛁ', 'ᛃ', 'ᛇ', 'ᛈ', 'ᛉ', 'ᛊ',
    'ᛏ', 'ᛒ', 'ᛖ', 'ᛗ', 'ᛚ', 'ᛜ', 'ᛞ', 'ᛟ',
];

/// RuneScript - A text component that reveals itself with a runic "deciphering" animation.
/// Formerly ScanningText, renamed for Norse-themed tactical alignment.
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
            color: [0.0, 1.0, 1.0, 1.0], // Cyan
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
    fn body(self) -> Self::Body { unreachable!() }

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
            renderer.draw_text(&display_text, rect.x, rect.y + self.font_size, self.font_size, self.color);
        }
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, _proposal: cvkg_core::SizeProposal) -> cvkg_core::Size {
        let (w, h) = renderer.measure_text(&self.text, self.font_size);
        cvkg_core::Size { width: w, height: h }
    }
}

/// SleipnirGait - A container that staggers the reveal of its children.
/// Named after Odin's 8-legged horse, known for its rapid and coordinated gait.
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
    fn body(self) -> Self::Body { unreachable!() }

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
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        
        if t < self.duration {
            // Render Runic Noise
            let mut noise = String::new();
            let char_count = (rect.width * rect.height / 200.0) as usize;
            for i in 0..char_count {
                let rune_idx = ((t * 50.0 + i as f32) as usize) % RUNES.len();
                noise.push(RUNES[rune_idx]);
                if i % 10 == 0 { noise.push('\n'); }
            }
            renderer.draw_text(&noise, rect.x, rect.y + 10.0, 10.0, [0.0, 1.0, 1.0, 0.4]);
        } else {
            // Reveal Content
            let opacity = ((t - self.duration) * 2.0).min(1.0);
            renderer.push_opacity(opacity);
            self.content.render(renderer, rect);
            renderer.pop_opacity();
        }
    }
}
