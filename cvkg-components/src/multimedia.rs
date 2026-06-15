use crate::theme;
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

/// Audio waveform visualizer component.
pub struct Audio {
    pub(crate) is_playing: bool,
    pub(crate) progress: f32,
    pub(crate) waveform: Vec<f32>,
}

impl Default for Audio {
    fn default() -> Self {
        Self::new()
    }
}

impl Audio {
    /// Create a new empty Audio component.
    pub fn new() -> Self {
        Self {
            is_playing: false,
            progress: 0.0,
            waveform: Vec::new(),
        }
    }

    /// Set current playback progress (0.0 to 1.0).
    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self
    }

    /// Set simulated waveform amplitude data.
    pub fn waveform(mut self, values: Vec<f32>) -> Self {
        self.waveform = values;
        self
    }

    /// Set playback state.
    pub fn playing(mut self, playing: bool) -> Self {
        self.is_playing = playing;
        self
    }
}

impl View for Audio {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Draw container box
        renderer.fill_rounded_rect(rect, 4.0, theme::surface());
        renderer.stroke_rounded_rect(rect, 4.0, theme::border(), 1.0);

        let icon = if self.is_playing { "⏸" } else { "▶" };
        renderer.draw_text(
            icon,
            rect.x + 12.0,
            rect.y + rect.height / 2.0 - 6.0,
            14.0,
            theme::accent(),
        );

        let wave_x = rect.x + 40.0;
        let wave_w = rect.width - 60.0;
        let wave_h = rect.height - 20.0;
        let wave_y = rect.y + 10.0;

        // Draw waveform bars
        if !self.waveform.is_empty() {
            let bar_count = self.waveform.len();
            let bar_w = wave_w / bar_count as f32;

            for i in 0..bar_count {
                let bar_x = wave_x + i as f32 * bar_w;
                let amp = self.waveform[i].clamp(0.0, 1.0);
                let current_bar_h = wave_h * amp;
                let current_bar_y = wave_y + (wave_h - current_bar_h) / 2.0;

                let bar_progress_ratio = i as f32 / bar_count as f32;
                let color = if bar_progress_ratio <= self.progress {
                    theme::accent()
                } else {
                    theme::text_dim()
                };

                renderer.fill_rect(
                    Rect {
                        x: bar_x + 1.0,
                        y: current_bar_y,
                        width: (bar_w - 2.0).max(1.0),
                        height: current_bar_h.max(2.0),
                    },
                    color,
                );
            }
        } else {
            // Draw dummy flat waveform
            renderer.draw_line(
                wave_x,
                wave_y + wave_h / 2.0,
                wave_x + wave_w,
                wave_y + wave_h / 2.0,
                theme::text_dim(),
                2.0,
            );
        }
    }
}

impl LayoutView for Audio {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 300.0,
            height: 60.0,
        }
    }
    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}

/// Video player component with scanning lines.
pub struct Video {
    pub(crate) title: String,
    pub(crate) is_playing: bool,
    pub(crate) playback_progress: f32,
}

impl Default for Video {
    fn default() -> Self {
        Self::new()
    }
}

impl Video {
    /// Create a new Video component.
    pub fn new() -> Self {
        Self {
            title: "Feed-01".to_string(),
            is_playing: false,
            playback_progress: 0.0,
        }
    }

    /// Set video title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set playback progress (0.0 to 1.0).
    pub fn progress(mut self, progress: f32) -> Self {
        self.playback_progress = progress.clamp(0.0, 1.0);
        self
    }

    /// Set video playback state.
    pub fn playing(mut self, playing: bool) -> Self {
        self.is_playing = playing;
        self
    }
}

impl View for Video {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Draw border frame with styled corner highlights
        renderer.fill_rect(rect, theme::surface());
        renderer.stroke_rect(rect, theme::border(), 1.0);

        // Highlight corners
        let cs = 12.0;
        renderer.draw_line(rect.x, rect.y, rect.x + cs, rect.y, theme::accent(), 2.0);
        renderer.draw_line(rect.x, rect.y, rect.x, rect.y + cs, theme::accent(), 2.0);
        renderer.draw_line(
            rect.x + rect.width - cs,
            rect.y,
            rect.x + rect.width,
            rect.y,
            theme::accent(),
            2.0,
        );
        renderer.draw_line(
            rect.x + rect.width,
            rect.y,
            rect.x + rect.width,
            rect.y + cs,
            theme::accent(),
            2.0,
        );

        renderer.draw_line(
            rect.x,
            rect.y + rect.height,
            rect.x + cs,
            rect.y + rect.height,
            theme::accent(),
            2.0,
        );
        renderer.draw_line(
            rect.x,
            rect.y + rect.height - cs,
            rect.x,
            rect.y + rect.height,
            theme::accent(),
            2.0,
        );
        renderer.draw_line(
            rect.x + rect.width - cs,
            rect.y + rect.height,
            rect.x + rect.width,
            rect.y + rect.height,
            theme::accent(),
            2.0,
        );
        renderer.draw_line(
            rect.x + rect.width,
            rect.y + rect.height - cs,
            rect.x + rect.width,
            rect.y + rect.height,
            theme::accent(),
            2.0,
        );

        // Draw scanlines (transparent horizontal grid)
        let mut scan_y = rect.y + 4.0;
        while scan_y < rect.y + rect.height {
            renderer.draw_line(
                rect.x,
                scan_y,
                rect.x + rect.width,
                scan_y,
                [
                    theme::accent()[0],
                    theme::accent()[1],
                    theme::accent()[2],
                    0.05,
                ],
                0.5,
            );
            scan_y += 6.0;
        }

        // Overlay status text
        renderer.draw_text(
            &self.title,
            rect.x + 10.0,
            rect.y + 20.0,
            10.0,
            theme::text_muted(),
        );
        let status = if self.is_playing {
            "PLAYING // FEED_ACTIVE"
        } else {
            "PAUSED // STANDBY"
        };
        renderer.draw_text(
            status,
            rect.x + 10.0,
            rect.y + rect.height - 24.0,
            9.0,
            theme::accent(),
        );

        // Playback progress bar
        let bar_h = 2.0;
        let progress_w = rect.width * self.playback_progress;
        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: rect.y + rect.height - bar_h,
                width: progress_w,
                height: bar_h,
            },
            theme::accent(),
        );
    }
}

impl LayoutView for Video {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 400.0,
            height: 250.0,
        }
    }
    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}

/// Coordinate grid Map component with tactical sonar radar sweeps.
pub struct Map {
    pub(crate) sonar_sweep_angle: f32,
    pub(crate) markers: Vec<(f32, f32, String)>,
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Map {
    /// Create a new tactical Map component.
    pub fn new() -> Self {
        Self {
            sonar_sweep_angle: 0.0,
            markers: Vec::new(),
        }
    }

    /// Set radar sweep line angle.
    pub fn sweep_angle(mut self, angle: f32) -> Self {
        self.sonar_sweep_angle = angle;
        self
    }

    /// Add a tactical marker point (normalized 0.0 to 1.0 position coordinates).
    pub fn marker(mut self, x: f32, y: f32, label: &str) -> Self {
        self.markers
            .push((x.clamp(0.0, 1.0), y.clamp(0.0, 1.0), label.to_string()));
        self
    }
}

impl View for Map {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Container background grid
        renderer.fill_rect(rect, theme::surface());
        renderer.stroke_rect(rect, theme::border(), 1.0);

        // Coordinate lines
        let steps = 4;
        for i in 1..steps {
            let x = rect.x + (i as f32 / steps as f32) * rect.width;
            renderer.draw_line(
                x,
                rect.y,
                x,
                rect.y + rect.height,
                [
                    theme::accent()[0],
                    theme::accent()[1],
                    theme::accent()[2],
                    0.1,
                ],
                0.5,
            );

            let y = rect.y + (i as f32 / steps as f32) * rect.height;
            renderer.draw_line(
                rect.x,
                y,
                rect.x + rect.width,
                y,
                [
                    theme::accent()[0],
                    theme::accent()[1],
                    theme::accent()[2],
                    0.1,
                ],
                0.5,
            );
        }

        // Radar center sweep concentric rings
        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        let max_r = (rect.width.min(rect.height) / 2.0 - 10.0).max(10.0);

        for i in 1..=3 {
            let r = max_r * (i as f32 / 3.0);
            renderer.stroke_ellipse(
                Rect {
                    x: center_x - r,
                    y: center_y - r,
                    width: r * 2.0,
                    height: r * 2.0,
                },
                [
                    theme::accent()[0],
                    theme::accent()[1],
                    theme::accent()[2],
                    0.15,
                ],
                0.5,
            );
        }

        // Sweep vector line
        let sweep_x = center_x + max_r * self.sonar_sweep_angle.cos();
        let sweep_y = center_y + max_r * self.sonar_sweep_angle.sin();
        renderer.draw_line(
            center_x,
            center_y,
            sweep_x,
            sweep_y,
            [
                theme::accent()[0],
                theme::accent()[1],
                theme::accent()[2],
                0.6,
            ],
            1.5,
        );

        // Render tactical point markers
        for (mx, my, label) in &self.markers {
            let px = rect.x + mx * rect.width;
            let py = rect.y + my * rect.height;

            // Dot pulse ring
            renderer.stroke_ellipse(
                Rect {
                    x: px - 6.0,
                    y: py - 6.0,
                    width: 12.0,
                    height: 12.0,
                },
                theme::success(),
                1.0,
            );
            renderer.fill_ellipse(
                Rect {
                    x: px - 2.0,
                    y: py - 2.0,
                    width: 4.0,
                    height: 4.0,
                },
                theme::success(),
            );

            renderer.draw_text(label, px + 8.0, py - 5.0, 8.0, theme::text_muted());
        }
    }
}

impl LayoutView for Map {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 300.0,
            height: 300.0,
        }
    }
    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}
