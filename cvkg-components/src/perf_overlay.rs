use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};
use std::time::Instant;

/// Performance profiling overlay showing real-time frame statistics.
///
/// Displays FPS, frame time (ms), draw call count, and a rolling
/// frame-time graph. Toggle with Cmd/Ctrl+Shift+P (OS-agnostic).
pub struct PerfOverlay {
    /// Whether the overlay is currently visible.
    pub visible: bool,
    /// Number of frames to average for FPS display.
    pub sample_window: usize,
    history: Vec<f32>,
    last_frame: Option<Instant>,
    current_fps: f32,
    current_frame_ms: f32,
    /// Number of draw calls in the last frame.
    pub draw_calls: u32,
    /// Number of triangles rendered in the last frame.
    pub triangles: u32,
    /// Number of vertices rendered in the last frame.
    pub vertices: u32,
    /// GPU time estimate in ms (if available).
    pub gpu_time_ms: f32,
    peak_frame_ms: f32,
}

impl PerfOverlay {
    /// Create a new performance overlay with default settings.
    pub fn new() -> Self {
        Self {
            visible: false,
            sample_window: 120,
            history: Vec::with_capacity(120),
            last_frame: None,
            current_fps: 0.0,
            current_frame_ms: 0.0,
            draw_calls: 0,
            triangles: 0,
            vertices: 0,
            gpu_time_ms: 0.0,
            peak_frame_ms: 0.0,
        }
    }

    /// Show the overlay.
    pub fn show(mut self) -> Self {
        self.visible = true;
        self
    }

    /// Hide the overlay.
    pub fn hide(mut self) -> Self {
        self.visible = false;
        self
    }

    /// Record a frame completion with the given active frame duration and draw statistics.
    ///
    /// # Contract
    /// `frame_time_ms` is the active duration (build + layout + paint + submit) of the frame.
    /// Internal statistics use this active duration for graphing/averages to avoid sleep pollution.
    pub fn record_frame(
        &mut self,
        frame_time_ms: f32,
        draw_calls: u32,
        triangles: u32,
        vertices: u32,
    ) {
        let now = Instant::now();
        if let Some(last) = self.last_frame {
            let dt = now.duration_since(last).as_secs_f32();
            if dt > 0.0 {
                self.current_fps = 1.0 / dt;
            }
            self.current_frame_ms = frame_time_ms;
            self.peak_frame_ms = self.peak_frame_ms.max(frame_time_ms);
            self.history.push(frame_time_ms);
            if self.history.len() > self.sample_window {
                self.history.remove(0);
            }
        }
        self.last_frame = Some(now);
        self.draw_calls = draw_calls;
        self.triangles = triangles;
        self.vertices = vertices;
    }

    /// Get the average frame time over the sample window.
    pub fn avg_frame_ms(&self) -> f32 {
        if self.history.is_empty() {
            return 0.0;
        }
        self.history.iter().sum::<f32>() / self.history.len() as f32
    }

    /// Get the 99th percentile frame time.
    pub fn p99_frame_ms(&self) -> f32 {
        if self.history.is_empty() {
            return 0.0;
        }
        let mut sorted = self.history.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((sorted.len() as f32) * 0.99) as usize;
        sorted.get(idx).copied().unwrap_or(0.0)
    }
}

impl Default for PerfOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl View for PerfOverlay {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.visible {
            return;
        }

        let pad = 12.0;
        let overlay_w: f32 = 280.0;
        let overlay_h: f32 = 220.0;
        let ox = rect.x + rect.width - overlay_w - 16.0;
        let oy = rect.y + 170.0;

        let panel_rect = Rect {
            x: ox,
            y: oy,
            width: overlay_w,
            height: overlay_h,
        };

        // Push vnode with the actual panel rectangle so it only captures pointer events
        // within the overlay itself, leaving the rest of the window interactive.
        renderer.push_vnode(panel_rect, "PerfOverlay");

        // Background
        renderer.fill_rounded_rect(
            Rect {
                x: ox,
                y: oy,
                width: overlay_w,
                height: overlay_h,
            },
            10.0,
            [0.02, 0.02, 0.04, 0.96],
        );
        renderer.stroke_rounded_rect(
            Rect {
                x: ox,
                y: oy,
                width: overlay_w,
                height: overlay_h,
            },
            10.0,
            theme::border(),
            1.0,
        );

        let text_x = ox + pad;
        let mut y = oy + pad;

        // Title
        let title_baseline = renderer.measure_text_baseline("Performance", 14.0);
        renderer.draw_text("Performance", text_x + 1.0, y + 1.0 - title_baseline, 14.0, [0.0, 0.0, 0.0, 0.75]);
        renderer.draw_text("Performance", text_x, y - title_baseline, 14.0, theme::text());
        y += 22.0;

        // Divider
        renderer.draw_line(text_x, y, ox + overlay_w - pad, y, theme::border(), 1.0);
        y += 8.0;

        // FPS
        let fps_color = if self.current_fps >= 55.0 {
            theme::toast_success()
        } else if self.current_fps >= 30.0 {
            theme::warning()
        } else {
            theme::error_color()
        };
        let fps_text = format!("{:.0} FPS", self.current_fps);
        let frame_text = format!("{:.1}ms", self.current_frame_ms);
        let fps_baseline = renderer.measure_text_baseline(&fps_text, 16.0);
        let frame_baseline = renderer.measure_text_baseline(&frame_text, 13.0);
        renderer.draw_text(&fps_text, text_x + 1.0, y + 1.0 - fps_baseline, 16.0, [0.0, 0.0, 0.0, 0.75]);
        renderer.draw_text(&fps_text, text_x, y - fps_baseline, 16.0, fps_color);
        renderer.draw_text(&frame_text, text_x + 111.0, y + 1.0 - frame_baseline, 13.0, [0.0, 0.0, 0.0, 0.75]);
        renderer.draw_text(&frame_text, text_x + 110.0, y - frame_baseline, 13.0, theme::text());
        y += 22.0;

        // Stats
        let draw_calls_text = format!("Draw Calls: {}", self.draw_calls);
        let draw_calls_baseline = renderer.measure_text_baseline(&draw_calls_text, 11.0);
        renderer.draw_text(&draw_calls_text, text_x + 1.0, y + 1.0 - draw_calls_baseline, 11.0, [0.0, 0.0, 0.0, 0.7]);
        renderer.draw_text(&draw_calls_text, text_x, y - draw_calls_baseline, 11.0, theme::text());
        y += 18.0;
        let geom_text = format!("Tris: {}  Verts: {}", self.triangles, self.vertices);
        let geom_baseline = renderer.measure_text_baseline(&geom_text, 11.0);
        renderer.draw_text(&geom_text, text_x + 1.0, y + 1.0 - geom_baseline, 11.0, [0.0, 0.0, 0.0, 0.7]);
        renderer.draw_text(&geom_text, text_x, y - geom_baseline, 11.0, theme::text());
        y += 18.0;
        let stats_text = format!("Avg: {:.1}ms  P99: {:.1}ms", self.avg_frame_ms(), self.p99_frame_ms());
        let stats_baseline = renderer.measure_text_baseline(&stats_text, 11.0);
        renderer.draw_text(&stats_text, text_x + 1.0, y + 1.0 - stats_baseline, 11.0, [0.0, 0.0, 0.0, 0.7]);
        renderer.draw_text(&stats_text, text_x, y - stats_baseline, 11.0, theme::text());
        // Rolling frame time graph
        let graph_x = text_x;
        let graph_w = overlay_w - pad * 2.0;
        let graph_h: f32 = 50.0;
        let graph_y = oy + overlay_h - pad - graph_h;

        // Graph background
        renderer.fill_rounded_rect(
            Rect {
                x: graph_x,
                y: graph_y,
                width: graph_w,
                height: graph_h,
            },
            4.0,
            theme::surface_elevated(),
        );

        // Grid lines
        for i in 0..=4u32 {
            let gy = graph_y + graph_h * i as f32 / 4.0;
            renderer.draw_line(graph_x, gy, graph_x + graph_w, gy, theme::border(), 0.5);
        }

        // Frame time bars
        if !self.history.is_empty() {
            let max_ms = self.peak_frame_ms.max(33.0);
            let bar_w = (graph_w / self.history.len() as f32).max(1.0);
            for (i, &ms) in self.history.iter().enumerate() {
                let bar_h = (ms / max_ms * graph_h).min(graph_h);
                let bar_x = graph_x + i as f32 * bar_w;
                let bar_y = graph_y + graph_h - bar_h;
                let bar_color = if ms <= 16.67 {
                    theme::toast_success()
                } else if ms <= 33.33 {
                    theme::warning()
                } else {
                    theme::error_color()
                };
                renderer.fill_rect(
                    Rect {
                        x: bar_x,
                        y: bar_y,
                        width: bar_w - 0.5,
                        height: bar_h,
                    },
                    bar_color,
                );
            }
        }

        // 16ms target line (60fps)
        let target_y = graph_y + graph_h - (16.67 / self.peak_frame_ms.max(33.0) * graph_h);
        if target_y >= graph_y {
            renderer.draw_line(
                graph_x,
                target_y,
                graph_x + graph_w,
                target_y,
                theme::accent(),
                1.0,
            );
        }

        renderer.pop_vnode();
    }
}
