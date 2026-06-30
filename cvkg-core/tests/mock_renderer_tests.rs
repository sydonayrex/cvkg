#[cfg(test)]
mod mock_renderer_tests {
    use crate::testing::MockRenderer;
    use crate::{Rect, Renderer};

    #[test]
    fn test_mock_renderer_records_fill_rect() {
        let mut renderer = MockRenderer::new();
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        renderer.fill_rect(rect, [1.0, 0.0, 0.0, 1.0]);
        renderer.assert_draw_call_count(1);
        renderer.assert_color_at(50.0, 40.0, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_mock_renderer_records_text() {
        let mut renderer = MockRenderer::new();
        renderer.draw_text_raw("Hello", 100.0, 200.0, 14.0, [0.0, 0.0, 0.0, 1.0]);
        renderer.assert_draw_call_count(1);
        renderer.assert_text_rendered("Hello");
    }

    #[test]
    fn test_mock_renderer_records_stroke() {
        let mut renderer = MockRenderer::new();
        let rect = Rect::new(0.0, 0.0, 50.0, 50.0);
        renderer.stroke_rect(rect, [0.0, 1.0, 0.0, 1.0], 2.0);
        renderer.assert_draw_call_count(1);
    }
}
