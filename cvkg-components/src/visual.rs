use cvkg_core::{Never, View};

/// Linear or circular progress indicator
#[allow(dead_code)]
pub struct ProgressView {
    pub(crate) value: f32,
    pub(crate) total: f32,
}

impl ProgressView {
    /// Create a new ProgressView.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::ProgressView;
    /// let progress = ProgressView::new(0.5, 1.0);
    /// ```
    pub fn new(value: f32, total: f32) -> Self {
        Self { value, total }
    }
}

impl View for ProgressView {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        let track_h = 8.0;
        let track_y = rect.y + (rect.height - track_h) / 2.0;
        // Background track
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x,
                y: track_y,
                width: rect.width,
                height: track_h,
            },
            track_h / 2.0,
            [0.15, 0.15, 0.2, 1.0],
        );
        // Fill
        let pct = if self.total > f32::EPSILON {
            (self.value / self.total).clamp(0.0, 1.0)
        } else {
            0.0
        };
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x,
                y: track_y,
                width: rect.width * pct,
                height: track_h,
            },
            track_h / 2.0,
            [0.0, 0.85, 1.0, 1.0],
        );
    }
}

/// Radial or linear gauge display
#[allow(dead_code)]
pub struct Gauge {
    pub(crate) value: f32,
    pub(crate) range: std::ops::RangeInclusive<f32>,
}

impl Gauge {
    /// Create a new Gauge.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::Gauge;
    /// let gauge = Gauge::new(50.0, 0.0..=100.0);
    /// ```
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
        // Outer ring
        renderer.stroke_ellipse(rect, [0.15, 0.15, 0.2, 1.0], 6.0);
        // Arc fill — approximate with a coloured inner ellipse scaled by value
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
}
use cvkg_core::{Rect, Renderer};

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
    /// Create a new ChartView.
    pub fn new(chart_type: ChartType, data: Vec<f32>) -> Self {
        Self {
            chart_type,
            data,
            color: [0.0, 1.0, 1.0, 1.0], // Default NiflCyan
        }
    }

    /// Set the chart color.
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
                    return;
                }

                let center_x = rect.x + rect.width / 2.0;
                let center_y = rect.y + rect.height / 2.0;
                let max_radius = rect.width.min(rect.height) / 2.0;

                // Draw background axes
                let num_axes = self.data.len();
                for i in 0..num_axes {
                    let angle = (i as f32 / num_axes as f32) * 2.0 * std::f32::consts::PI
                        - std::f32::consts::FRAC_PI_2;
                    let x = center_x + angle.cos() * max_radius;
                    let y = center_y + angle.sin() * max_radius;
                    renderer.draw_line(center_x, center_y, x, y, [0.3, 0.3, 0.4, 0.5], 1.0);
                }

                // Draw data polygon
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

                    // Draw the edge of the polygon
                    renderer.draw_line(x1, y1, x2, y2, self.color, 2.0);

                    // Optional: draw small points at vertices
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
    }
}
