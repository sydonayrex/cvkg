use cvkg_core::{Never, Rect, Renderer, View};

/// Rich text segment type
pub enum RichTextSegment {
    Text(String),
    Code(String),
    Image(String),
}

/// A component for displaying mixed content (text, code, images)
pub struct RichText {
    segments: Vec<RichTextSegment>,
}

impl Default for RichText {
    fn default() -> Self {
        Self::new()
    }
}

impl RichText {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn text(mut self, t: impl Into<String>) -> Self {
        self.segments.push(RichTextSegment::Text(t.into()));
        self
    }

    pub fn code(mut self, c: impl Into<String>) -> Self {
        self.segments.push(RichTextSegment::Code(c.into()));
        self
    }

    pub fn image(mut self, i: impl Into<String>) -> Self {
        self.segments.push(RichTextSegment::Image(i.into()));
        self
    }
}

impl View for RichText {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let mut y = rect.y;
        for segment in &self.segments {
            match segment {
                RichTextSegment::Text(t) => {
                    renderer.draw_text(t, rect.x, y, 14.0, [1.0, 1.0, 1.0, 1.0]);
                    y += 20.0;
                }
                RichTextSegment::Code(c) => {
                    // Render code block background
                    renderer.fill_rect(
                        Rect {
                            x: rect.x,
                            y,
                            width: rect.width,
                            height: 25.0,
                        },
                        [0.1, 0.1, 0.1, 1.0],
                    );
                    renderer.draw_text(c, rect.x + 5.0, y + 5.0, 12.0, [0.0, 1.0, 0.0, 1.0]);
                    y += 30.0;
                }
                RichTextSegment::Image(i) => {
                    renderer.draw_image(
                        i,
                        Rect {
                            x: rect.x,
                            y,
                            width: 40.0,
                            height: 40.0,
                        },
                    );
                    y += 45.0;
                }
            }
        }
    }
}
