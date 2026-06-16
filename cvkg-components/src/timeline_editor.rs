use crate::theme;
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

/// Animation timeline component
pub struct TimelineEditor {
    pub(crate) tracks: Vec<TimelineTrack>,
    pub(crate) current_time: f32,
    pub(crate) duration: f32,
}

pub struct TimelineTrack {
    pub name: String,
    pub keyframes: Vec<TimelineKeyframe>,
}

pub struct TimelineKeyframe {
    pub time: f32,
    pub value: f32,
}

impl Default for TimelineEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl TimelineEditor {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            current_time: 0.0,
            duration: 100.0,
        }
    }

    pub fn duration(mut self, d: f32) -> Self {
        self.duration = d;
        self
    }

    pub fn track(mut self, name: &str, keyframes: Vec<(f32, f32)>) -> Self {
        self.tracks.push(TimelineTrack {
            name: name.to_string(),
            keyframes: keyframes
                .into_iter()
                .map(|(t, v)| TimelineKeyframe { time: t, value: v })
                .collect(),
        });
        self
    }
}

impl View for TimelineEditor {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let header_h = 32.0;
        let track_h = 40.0;
        let track_count = self.tracks.len();
        let total_h = header_h + track_count as f32 * track_h + 40.0;

        if rect.height < total_h {
            return;
        }

        // Time ruler
        let ruler_y = rect.y + header_h;
        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: ruler_y,
                width: rect.width,
                height: 24.0,
            },
            theme::surface_elevated(),
        );

        // Time markers
        let marker_count = 10;
        for i in 0..=marker_count {
            let t = (i as f32 / marker_count as f32) * self.duration;
            let x = rect.x + (t / self.duration) * rect.width;
            renderer.draw_line(x, ruler_y, x, ruler_y + 24.0, theme::border(), 1.0);
            renderer.draw_text(
                &format!("{:.0}s", t),
                x - 10.0,
                ruler_y + 4.0,
                10.0,
                theme::text_muted(),
            );
        }

        // Current time indicator
        let current_x = rect.x + (self.current_time / self.duration) * rect.width;
        renderer.draw_line(
            current_x,
            ruler_y,
            current_x,
            ruler_y + 24.0,
            theme::accent(),
            2.0,
        );

        // Tracks
        let mut current_y = ruler_y + 28.0;
        for track in &self.tracks {
            let track_rect = Rect {
                x: rect.x,
                y: current_y,
                width: rect.width,
                height: track_h,
            };
            renderer.fill_rounded_rect(track_rect, 4.0, theme::input_bg());
            renderer.stroke_rounded_rect(track_rect, 4.0, theme::border(), 1.0);

            renderer.draw_text(
                &track.name,
                track_rect.x + 8.0,
                track_rect.y + 14.0,
                12.0,
                theme::text_muted(),
            );

            // Draw keyframes
            for kf in &track.keyframes {
                let x = rect.x + (kf.time / self.duration) * rect.width;
                let y = current_y + track_h / 2.0;
                let kf_rect = Rect {
                    x: x - 5.0,
                    y: y - 5.0,
                    width: 10.0,
                    height: 10.0,
                };
                renderer.fill_ellipse(kf_rect, theme::accent());
                renderer.stroke_ellipse(kf_rect, theme::accent(), 1.0);
            }

            current_y += track_h + 4.0;
        }
    }
}

impl LayoutView for TimelineEditor {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        let height = 32.0 + (self.tracks.len() as f32 * 44.0) + 40.0;
        Size {
            width: 600.0,
            height,
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
