use crate::theme;
use cvkg_core::{Event, Never, Rect, Renderer, View};
use std::sync::Arc;

/// A heavy, metallic, high-density slider.
/// Section 4.1: "Tactile HUD interaction with energy-based feedback."
pub struct MjolnirSlider {
    pub label: String,
    pub value: f32,
    pub range: std::ops::RangeInclusive<f32>,
    pub on_change: Arc<dyn Fn(f32) + Send + Sync>,
}

impl MjolnirSlider {
    pub fn new(
        label: impl Into<String>,
        value: f32,
        range: std::ops::RangeInclusive<f32>,
        on_change: impl Fn(f32) + Send + Sync + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            value,
            range,
            on_change: Arc::new(on_change),
        }
    }
}

impl View for MjolnirSlider {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let normalized =
            (self.value - self.range.start()) / (self.range.end() - self.range.start());
        let fill_width = rect.width * normalized.clamp(0.0, 1.0);

        renderer.push_vnode(rect, "MjolnirSlider");

        // 1. Heavy Metallic Base
        renderer.fill_rounded_rect(rect, 4.0, theme::surface());
        renderer.stroke_rounded_rect(rect, 4.0, theme::border(), 1.5);

        // 2. Energy Fill (Cyan Pulse)
        let t = renderer.elapsed_time();
        let pulse = 0.8 + (t * 5.0).sin() * 0.2;
        let fill_color = [0.0, 0.8 * pulse, 1.0 * pulse, 0.9];

        let fill_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: fill_width,
            height: rect.height,
        };
        renderer.fill_rounded_rect(fill_rect, 4.0, fill_color);

        // 3. Runic Etching
        let label_text = format!("{}: {:.0}%", self.label, normalized * 100.0);
        renderer.draw_text(
            &label_text,
            rect.x + 8.0,
            rect.y + rect.height / 2.0 + 5.0,
            12.0,
            theme::text(),
        );

        // 4. Interaction Handler
        let on_change = self.on_change.clone();
        let range = self.range.clone();
        let rect_x = rect.x;
        let rect_w = rect.width;

        renderer.register_handler(
            "pointerclick",
            Arc::new(move |ev| {
                if let Event::PointerDown { x, .. } | Event::PointerMove { x, .. } = ev {
                    let local_x = x - rect_x;
                    let new_normalized = (local_x / rect_w).clamp(0.0, 1.0);
                    let new_val = range.start() + new_normalized * (range.end() - range.start());
                    on_change(new_val);
                }
            }),
        );

        renderer.pop_vnode();
    }
}
