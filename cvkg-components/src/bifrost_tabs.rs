use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::Arc;

/// Liquid glass tabs with chromatic aberration (inspired by liquid_glass_widgets).
/// Section 4.7: "Tactile realm-switching navigation with fluid feedback."
pub struct BifrostTabs {
    pub options: Vec<String>,
    pub selected_index: usize,
    pub on_select: Arc<dyn Fn(usize) + Send + Sync>,
}

impl BifrostTabs {
    pub fn new(options: Vec<String>, selected: usize, on_select: impl Fn(usize) + Send + Sync + 'static) -> Self {
        Self {
            options,
            selected_index: selected,
            on_select: Arc::new(on_select),
        }
    }
}

impl View for BifrostTabs {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        let tab_width = rect.width / self.options.len() as f32;

        // 1. Background Glass
        renderer.bifrost(rect, 20.0, 1.0, 0.8);
        renderer.stroke_rounded_rect(rect, 8.0, [0.3, 0.3, 0.4, 0.3], 1.0);

        // 2. Liquid Selection Indicator (The Bifrost bridge)
        let target_x = rect.x + (self.selected_index as f32 * tab_width);
        
        // Animated indicator with "jelly" physics (sinusoidal wobble)
        let wobble = (t * 4.0).sin() * 2.0;
        let indicator_rect = Rect {
            x: target_x + 4.0,
            y: rect.y + 4.0 + wobble,
            width: tab_width - 8.0,
            height: rect.height - 8.0,
        };

        renderer.gungnir(indicator_rect, [0.0, 0.8, 1.0, 0.6], 10.0, 0.8);
        renderer.fill_rounded_rect(indicator_rect, 6.0, [0.0, 0.5, 0.8, 0.4]);

        // 3. Tab Labels
        for (i, option) in self.options.iter().enumerate() {
            let x = rect.x + (i as f32 * tab_width);
            let alpha = if i == self.selected_index { 1.0 } else { 0.6 };
            
            renderer.draw_text(
                option,
                x + tab_width / 2.0 - 20.0,
                rect.y + rect.height / 2.0 + 5.0,
                14.0,
                [1.0, 1.0, 1.0, alpha],
            );

            // Interaction Handler
            let on_select = self.on_select.clone();
            let rect_x = x;
            let rect_w = tab_width;
            let idx = i;
            renderer.register_handler("pointerdown", Arc::new(move |ev| {
                if let cvkg_core::Event::PointerDown { x, .. } = ev {
                    if x >= rect_x && x <= rect_x + rect_w {
                        on_select(idx);
                    }
                }
            }));
        }
    }
}
