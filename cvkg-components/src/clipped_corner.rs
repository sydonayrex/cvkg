use cvkg_core::{Never, Rect, Renderer, View};

/// ClippedCornerNode - A tactical, editorial-grade UI node with clipped corners and neon borders.
/// Phase 4.3: "Replace legacy rectangles with tactical, editorial-grade SVG components."
pub struct ClippedCornerNode<V: View> {
    pub content: V,
    pub border_color: [f32; 4],
    pub border_width: f32,
    pub clip_size: f32,
    pub background_opacity: f32,
}

impl<V: View> ClippedCornerNode<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            border_color: [1.0, 0.84, 0.0, 0.9], // Viking Gold
            border_width: 1.5,
            clip_size: 12.0,
            background_opacity: 0.1,
        }
    }

    pub fn border_color(mut self, color: [f32; 4]) -> Self {
        self.border_color = color;
        self
    }

    pub fn clip_size(mut self, size: f32) -> Self {
        self.clip_size = size;
        self
    }
}

impl<V: View> View for ClippedCornerNode<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let s = self.clip_size;

        // 1. Path Points
        let points = [
            (rect.x + s, rect.y),
            (rect.x + rect.width - s, rect.y),
            (rect.x + rect.width, rect.y + s),
            (rect.x + rect.width, rect.y + rect.height - s),
            (rect.x + rect.width - s, rect.y + rect.height),
            (rect.x + s, rect.y + rect.height),
            (rect.x, rect.y + rect.height - s),
            (rect.x, rect.y + s),
        ];

        // 2. Background
        let mut bg_color = self.border_color;
        bg_color[3] = self.background_opacity;
        // Fill a beveled corner by stacking progressively smaller rectangles to approximate the chamfered edge.
        renderer.fill_rect(
            Rect {
                x: rect.x + s,
                y: rect.y,
                width: rect.width - 2.0 * s,
                height: rect.height,
            },
            bg_color,
        );
        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: rect.y + s,
                width: s,
                height: rect.height - 2.0 * s,
            },
            bg_color,
        );
        renderer.fill_rect(
            Rect {
                x: rect.x + rect.width - s,
                y: rect.y + s,
                width: s,
                height: rect.height - 2.0 * s,
            },
            bg_color,
        );

        // 3. Border
        for i in 0..8 {
            let p1 = points[i];
            let p2 = points[(i + 1) % 8];
            renderer.draw_line(p1.0, p1.1, p2.0, p2.1, self.border_color, self.border_width);
        }

        // 4. Content
        let inset = self.border_width + 2.0;
        let content_rect = Rect {
            x: rect.x + inset,
            y: rect.y + inset,
            width: rect.width - 2.0 * inset,
            height: rect.height - 2.0 * inset,
        };
        self.content.render(renderer, content_rect);
    }
}
