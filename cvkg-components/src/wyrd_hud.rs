use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// A high-fidelity tactical data display.
/// Section 4.9: "The Threads of Fate (Wyrd) — Precision telemetry readouts."
pub struct WyrdHUD {
    pub title: String,
    pub values: Vec<(String, String)>,
    pub rage_mode: bool,
}

impl WyrdHUD {
    pub fn new(title: impl Into<String>, values: Vec<(String, String)>) -> Self {
        Self {
            title: title.into(),
            values,
            rage_mode: false,
        }
    }

    pub fn with_rage(mut self, enabled: bool) -> Self {
        self.rage_mode = enabled;
        self
    }
}

impl View for WyrdHUD {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // 1. Technical Grid Background
        renderer.fill_rect(rect, theme::with_alpha(theme::bg(), 0.8));

        // Draw sub-grid lines
        let step = 20.0;
        let mut x = rect.x;
        while x < rect.x + rect.width {
            renderer.draw_line(
                x,
                rect.y,
                x,
                rect.y + rect.height,
                [0.0, 1.0, 1.0, 0.05],
                0.5,
            );
            x += step;
        }

        // 2. Corner 'Tracking' Brackets
        let b = 15.0;
        let c = theme::focus_ring();
        renderer.draw_line(rect.x, rect.y, rect.x + b, rect.y, c, 1.0);
        renderer.draw_line(rect.x, rect.y, rect.x, rect.y + b, c, 1.0);
        renderer.draw_line(
            rect.x + rect.width - b,
            rect.y + rect.height,
            rect.x + rect.width,
            rect.y + rect.height,
            c,
            1.0,
        );
        renderer.draw_line(
            rect.x + rect.width,
            rect.y + rect.height - b,
            rect.x + rect.width,
            rect.y + rect.height,
            c,
            1.0,
        );

        // 3. Header
        renderer.draw_text(
            &self.title,
            rect.x + 10.0,
            rect.y + 25.0,
            18.0,
            theme::warning(),
        );
        renderer.draw_line(
            rect.x + 10.0,
            rect.y + 35.0,
            rect.x + rect.width - 10.0,
            rect.y + 35.0,
            [1.0, 0.8, 0.0, 0.3],
            1.0,
        );

        // 4. Data Readouts
        for (i, (k, v)) in self.values.iter().enumerate() {
            let y = rect.y + 60.0 + (i as f32 * 25.0);

            // Key (Runic style label)
            renderer.draw_text(k, rect.x + 10.0, y, 12.0, theme::text_muted());

            // Value (Glow)
            let val_color = [0.0, 1.0, 1.0, 0.9];
            renderer.draw_text(v, rect.x + rect.width - 60.0, y, 14.0, val_color);
        }

        // 5. Scanning Bar (with Glitch)
        let t = renderer.elapsed_time();
        let scan_y = rect.y + (t * 50.0).fract() * rect.height;
        let mut glitch_offset = (t * 30.0).sin() * 2.0;

        if self.rage_mode {
            glitch_offset *= 2.5; // Amplified glitch in Rage mode

            // Draw red blood-rage vignette corners
            let v_size = 50.0;
            let v_color = [1.0, 0.0, 0.0, 0.2];
            renderer.fill_rect(Rect::new(rect.x, rect.y, v_size, v_size), v_color);
            renderer.fill_rect(
                Rect::new(rect.x + rect.width - v_size, rect.y, v_size, v_size),
                v_color,
            );
            renderer.fill_rect(
                Rect::new(rect.x, rect.y + rect.height - v_size, v_size, v_size),
                v_color,
            );
            renderer.fill_rect(
                Rect::new(
                    rect.x + rect.width - v_size,
                    rect.y + rect.height - v_size,
                    v_size,
                    v_size,
                ),
                v_color,
            );
        }

        // Main bar
        renderer.draw_line(
            rect.x,
            scan_y,
            rect.x + rect.width,
            scan_y,
            [0.0, 1.0, 1.0, 0.25],
            2.0,
        );

        // Chromatic fringes
        renderer.draw_line(
            rect.x,
            scan_y - glitch_offset,
            rect.x + rect.width,
            scan_y - glitch_offset,
            [1.0, 0.0, 0.0, 0.1],
            1.0,
        );
        renderer.draw_line(
            rect.x,
            scan_y + glitch_offset,
            rect.x + rect.width,
            scan_y + glitch_offset,
            [0.0, 0.0, 1.0, 0.1],
            1.0,
        );
    }
}
