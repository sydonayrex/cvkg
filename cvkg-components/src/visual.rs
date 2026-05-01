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

/// A real-time performance telemetry display
pub struct TelemetryView;

impl View for TelemetryView {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let stats = renderer.get_telemetry();
        
        renderer.fill_rounded_rect(rect, 4.0, [0.0, 0.0, 0.0, 0.8]);
        renderer.stroke_rect(rect, [0.0, 1.0, 0.5, 0.5], 1.0);
        
        let lines = [
            format!("FPS: {:.1}", 1000.0 / stats.frame_time_ms.max(0.1)),
            format!("Frame: {:.2} ms", stats.frame_time_ms),
            format!("Draw Calls: {}", stats.draw_calls),
            format!("Vertices: {}", stats.vertices),
        ];
        
        for (i, line) in lines.iter().enumerate() {
            renderer.draw_text(line, rect.x + 8.0, rect.y + 8.0 + i as f32 * 18.0, 12.0, [0.0, 1.0, 0.5, 1.0]);
        }
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: cvkg_core::SizeProposal) -> cvkg_core::Size {
        cvkg_core::Size { width: 140.0, height: 80.0 }
    }
}
