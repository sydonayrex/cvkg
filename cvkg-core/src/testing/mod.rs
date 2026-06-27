pub mod mock_renderer;

pub use mock_renderer::{DrawCall, MockRenderer};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rect, Renderer};

    #[test]
    fn mock_records_fill_rect() {
        let mut r = MockRenderer::new();
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        r.fill_rect(rect, [1.0, 0.0, 0.0, 1.0]);
        r.assert_draw_call_count(1);
        r.assert_color_at(50.0, 40.0, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn mock_records_text() {
        let mut r = MockRenderer::new();
        r.draw_text("Hello", 100.0, 200.0, 14.0, [0.0, 0.0, 0.0, 1.0]);
        r.assert_draw_call_count(1);
        r.assert_text_rendered("Hello");
    }

    #[test]
    fn mock_records_stroke() {
        let mut r = MockRenderer::new();
        let rect = Rect::new(0.0, 0.0, 50.0, 50.0);
        r.stroke_rect(rect, [0.0, 1.0, 0.0, 1.0], 2.0);
        r.assert_draw_call_count(1);
    }

    #[test]
    fn mock_records_line() {
        let mut r = MockRenderer::new();
        r.draw_line(0.0, 0.0, 100.0, 100.0, [0.5, 0.5, 0.5, 1.0], 1.0);
        r.assert_draw_call_count(1);
    }

    #[test]
    fn mock_records_ellipse() {
        let mut r = MockRenderer::new();
        let rect = Rect::new(200.0, 300.0, 60.0, 60.0);
        r.fill_ellipse(rect, [0.0, 0.0, 1.0, 1.0]);
        r.assert_draw_call_count(1);
    }

    #[test]
    fn mock_draw_text_centered() {
        let mut r = MockRenderer::new();
        r.draw_text_centered("Centered", 50.0, 50.0, 12.0, [1.0, 1.0, 1.0, 1.0]);
        r.assert_draw_call_count(1);
        r.assert_text_rendered("Centered");
    }
}
