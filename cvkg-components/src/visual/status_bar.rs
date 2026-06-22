use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// A horizontal status bar for system indicators.
#[derive(Clone)]
pub struct StatusBar {
    pub segments: Vec<StatusSegment>,
    pub height: f32,
}

#[derive(Clone)]
pub struct StatusSegment {
    pub label: String,
    pub value: f32,
    pub color: [f32; 4],
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            height: 24.0,
        }
    }

    pub fn segment(mut self, label: &str, value: f32, color: [f32; 4]) -> Self {
        self.segments.push(StatusSegment {
            label: label.to_string(),
            value,
            color,
        });
        self
    }
}

impl View for StatusBar {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!("Primitive view has no body")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rect(rect, theme::surface());
        let seg_width = rect.width / self.segments.len().max(1) as f32;
        for (i, seg) in self.segments.iter().enumerate() {
            let seg_rect = Rect {
                x: rect.x + i as f32 * seg_width,
                y: rect.y,
                width: seg_width,
                height: rect.height,
            };
            renderer.fill_rect(seg_rect, seg.color);
            let (tw, th) = renderer.measure_text(&seg.label, 10.0);
            renderer.draw_text(
                &seg.label,
                seg_rect.x + (seg_width - tw) / 2.0,
                seg_rect.y + (rect.height - th) / 2.0,
                10.0,
                theme::text(),
            );
        }
    }
}
