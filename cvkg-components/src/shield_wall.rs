use cvkg_core::{Never, Rect, Renderer, View};

/// A futuristic Scifi-Viking frame.
/// Features animated glowing borders and 'shield' corner brackets.
pub struct ShieldWall<V: View> {
    pub content: V,
    pub border_color: [f32; 4],
    pub glow_intensity: f32,
}

impl<V: View> ShieldWall<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            border_color: [0.0, 0.8, 1.0, 1.0], // Cyan
            glow_intensity: 0.8,
        }
    }

    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.border_color = color;
        self
    }
}

impl<V: View> View for ShieldWall<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // 1. Render the glowing border (Gungnir effect)
        renderer.gungnir(
            rect,
            self.border_color,
            10.0 * self.glow_intensity,
            self.glow_intensity,
        );

        // 2. Draw the 'Shield' corner brackets
        let bracket_size = 20.0;
        let c = self.border_color;

        // Top-Left
        renderer.draw_line(rect.x, rect.y, rect.x + bracket_size, rect.y, c, 2.0);
        renderer.draw_line(rect.x, rect.y, rect.x, rect.y + bracket_size, c, 2.0);

        // Top-Right
        renderer.draw_line(
            rect.x + rect.width - bracket_size,
            rect.y,
            rect.x + rect.width,
            rect.y,
            c,
            2.0,
        );
        renderer.draw_line(
            rect.x + rect.width,
            rect.y,
            rect.x + rect.width,
            rect.y + bracket_size,
            c,
            2.0,
        );

        // Bottom-Left
        renderer.draw_line(
            rect.x,
            rect.y + rect.height - bracket_size,
            rect.x,
            rect.y + rect.height,
            c,
            2.0,
        );
        renderer.draw_line(
            rect.x,
            rect.y + rect.height,
            rect.x + bracket_size,
            rect.y + rect.height,
            c,
            2.0,
        );

        // Bottom-Right
        renderer.draw_line(
            rect.x + rect.width - bracket_size,
            rect.y + rect.height,
            rect.x + rect.width,
            rect.y + rect.height,
            c,
            2.0,
        );
        renderer.draw_line(
            rect.x + rect.width,
            rect.y + rect.height - bracket_size,
            rect.x + rect.width,
            rect.y + rect.height,
            c,
            2.0,
        );

        // 3. Render content
        self.content.render(renderer, rect);
    }
}
