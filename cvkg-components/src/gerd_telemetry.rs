//! Gerd Telemetry - Reactive telemetry dashboard
//!
//! Gerd the Vanir giantess represents fierce protection and boundary defense -
//! this telemetry system monitors and protects system boundaries with reactive insights.

use cvkg_core::{layout::{LayoutCache, LayoutView, SizeProposal}, Rect, Renderer, Size, View, Never};

/// Telemetry data point
#[derive(Debug, Clone)]
pub struct TelemetryPoint {
    pub timestamp: f64,
    pub metric: String,
    pub value: f64,
    pub unit: String,
}

/// Telemetry series for trending
#[derive(Debug, Clone)]
pub struct TelemetrySeries {
    pub name: String,
    pub points: Vec<TelemetryPoint>,
    pub color: [f32; 4],
}

/// Gerd Telemetry Dashboard for reactive monitoring
pub struct GerdTelemetry {
    pub(crate) series: Vec<TelemetrySeries>,
    pub(crate) alerts: Vec<Alert>,
}

/// Alert for threshold violations
#[derive(Debug, Clone)]
pub struct Alert {
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl Default for GerdTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

impl GerdTelemetry {
    pub fn new() -> Self {
        Self {
            series: Vec::new(),
            alerts: Vec::new(),
        }
    }

    pub fn series(mut self, name: &str, color: [f32; 4]) -> Self {
        self.series.push(TelemetrySeries {
            name: name.to_string(),
            points: Vec::new(),
            color,
        });
        self
    }

    pub fn point(mut self, series_name: &str, value: f64, unit: &str) -> Self {
        if let Some(s) = self.series.iter_mut().find(|s| s.name == series_name) {
            s.points.push(TelemetryPoint {
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64(),
                metric: series_name.to_string(),
                value,
                unit: unit.to_string(),
            });
        }
        self
    }

    pub fn alert(mut self, message: &str, severity: AlertSeverity) -> Self {
        self.alerts.push(Alert {
            message: message.to_string(),
            severity,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64(),
        });
        self
    }
}

impl View for GerdTelemetry {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Header
        renderer.fill_rect(
            Rect { x: rect.x, y: rect.y, width: rect.width, height: 28.0 },
            [0.08, 0.04, 0.08, 1.0]
        );
        renderer.draw_text("Gerd Telemetry Dashboard", rect.x + 10.0, rect.y + 9.0, 13.0, [0.9, 0.6, 0.9, 1.0]);

        // Alerts section
        let alert_y = rect.y + 35.0;
        for alert in &self.alerts {
            let color = match alert.severity {
                AlertSeverity::Info => [0.4, 0.7, 1.0, 1.0],
                AlertSeverity::Warning => [0.9, 0.7, 0.2, 1.0],
                AlertSeverity::Critical => [0.9, 0.3, 0.3, 1.0],
            };
            renderer.fill_rect(
                Rect { x: rect.x + 10.0, y: alert_y, width: rect.width - 20.0, height: 20.0 },
                [0.1, 0.1, 0.12, 1.0]
            );
            renderer.draw_text(&format!("⚠ {}", alert.message), rect.x + 15.0, alert_y + 5.0, 10.0, color);
        }

        // Series visualization
        let viz_y = rect.y + 65.0 + (self.alerts.len() as f32 * 25.0);
        let viz_h = rect.height - viz_y - 10.0;

        for (i, series) in self.series.iter().enumerate() {
            let series_y = viz_y + i as f32 * (viz_h / self.series.len() as f32);
            let series_h = viz_h / self.series.len() as f32 - 5.0;

            // Draw trend line
            if series.points.len() > 1 {
                let max_val = series.points.iter().map(|p| p.value).fold(0.0f64, f64::max).max(1.0);
                let min_val = series.points.iter().map(|p| p.value).fold(0.0, f64::min);
                let val_range = (max_val - min_val).max(1.0);

                for j in 0..series.points.len() - 1 {
                    let x1 = rect.x + 20.0 + (j as f32 / series.points.len() as f32) * (rect.width - 40.0);
                    let y1 = series_y + series_h - ((series.points[j].value - min_val) / val_range) as f32 * series_h;
                    let x2 = rect.x + 20.0 + ((j + 1) as f32 / series.points.len() as f32) * (rect.width - 40.0);
                    let y2 = series_y + series_h - ((series.points[j + 1].value - min_val) / val_range) as f32 * series_h;
                    renderer.draw_line(x1, y1, x2, y2, series.color, 2.0);
                }
            }

            renderer.draw_text(&series.name, rect.x + 10.0, series_y, 10.0, series.color);
            if let Some(last) = series.points.last() {
                renderer.draw_text(&format!("{:.1}{}", last.value, last.unit), rect.x + rect.width - 60.0, series_y, 10.0, [0.7, 0.8, 0.9, 1.0]);
            }
        }
    }
}

impl LayoutView for GerdTelemetry {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn LayoutView], _cache: &mut LayoutCache) -> Size {
        Size { width: 320.0, height: 200.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn LayoutView], _cache: &mut LayoutCache) {}
}