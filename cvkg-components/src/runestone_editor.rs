use cvkg_core::{Never, Rect, Renderer, View};

/// A code editor with runic syntax highlighting.
/// Section 4.3: "Scriptorium components for runic logic definition."
pub struct RunestoneEditor {
    pub text: String,
    pub language: String,
}

impl RunestoneEditor {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            language: "runic".to_string(),
        }
    }
}

impl View for RunestoneEditor {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // 1. Etched Stone Background
        renderer.fill_rect(rect, [0.03, 0.03, 0.05, 1.0]);
        renderer.stroke_rect(rect, [0.2, 0.2, 0.3, 1.0], 1.0);

        // 2. Line Numbers (Gutter)
        let gutter_width = 40.0;
        let gutter_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: gutter_width,
            height: rect.height,
        };
        renderer.fill_rect(gutter_rect, [0.05, 0.05, 0.08, 1.0]);

        let lines: Vec<&str> = self.text.lines().collect();
        for (i, _) in lines.iter().enumerate() {
            renderer.draw_text(
                &(i + 1).to_string(),
                rect.x + 10.0,
                rect.y + 20.0 + (i as f32 * 20.0),
                12.0,
                [0.4, 0.4, 0.5, 1.0],
            );
        }

        // 3. Syntax Highlighting (Pseudo-Runic)
        for (i, line) in lines.iter().enumerate() {
            let mut current_x = rect.x + gutter_width + 10.0;
            let y = rect.y + 20.0 + (i as f32 * 20.0);

            // Simple word-based highlighting
            for word in line.split_whitespace() {
                let color = match word {
                    "fn" | "let" | "pub" | "use" => [1.0, 0.6, 0.0, 1.0], // Gold keywords
                    "rune" | "spell" | "incantation" => [0.0, 1.0, 1.0, 1.0], // Cyan "magic" types
                    _ => [0.8, 0.8, 0.9, 1.0],                            // White text
                };

                renderer.draw_text(word, current_x, y, 14.0, color);
                let (w, _) = renderer.measure_text(word, 14.0);
                current_x += w + 8.0;
            }
        }

        // 4. Cursor (Pulsing Amber)
        let t = renderer.elapsed_time();
        if (t * 2.0).fract() > 0.5 {
            let cursor_x = rect.x + gutter_width + 10.0;
            let cursor_y = rect.y + 10.0;
            renderer.draw_line(
                cursor_x,
                cursor_y,
                cursor_x,
                cursor_y + 16.0,
                [1.0, 0.5, 0.0, 1.0],
                2.0,
            );
        }
    }
}
