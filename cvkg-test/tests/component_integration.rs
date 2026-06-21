// No nightly features required
use cvkg_components::{Button, Hvergelmir, Text, VStack};
use cvkg_core::{Rect, Renderer, View};
use cvkg_macros::view_component;

struct MockRenderer {
    commands: Vec<String>,
}

impl MockRenderer {
    fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
}

impl cvkg_core::ElapsedTime for MockRenderer {
    fn elapsed_time(&self) -> f32 {
        0.0
    }
    fn delta_time(&self) -> f32 {
        0.016
    }
}

impl Renderer for MockRenderer {
    fn fill_rect(&mut self, _rect: Rect, _color: [f32; 4]) {}
    fn fill_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: [f32; 4]) {}
    fn fill_ellipse(&mut self, _rect: Rect, _color: [f32; 4]) {}
    fn stroke_rect(&mut self, _rect: Rect, _color: [f32; 4], _width: f32) {}
    fn stroke_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: [f32; 4], _width: f32) {}
    fn stroke_ellipse(&mut self, _rect: Rect, _color: [f32; 4], _width: f32) {}
    fn draw_line(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _color: [f32; 4], _width: f32) {
    }
    fn fill_polygon(&mut self, vertices: &[[f32; 2]], _color: [f32; 4]) {
        self.commands
            .push(format!("FillPolygon(points: {})", vertices.len()));
    }
    fn stroke_polygon(&mut self, vertices: &[[f32; 2]], _color: [f32; 4], _width: f32) {
        self.commands
            .push(format!("StrokePolygon(points: {})", vertices.len()));
    }
    fn shape_rich_text(
        &mut self,
        _spans: &[cvkg_runic_text::TextSpan],
        _max_width: Option<f32>,
        _align: cvkg_runic_text::TextAlign,
        _overflow: cvkg_runic_text::TextOverflow,
    ) -> Option<cvkg_runic_text::ShapedText> {
        let mut engine = cvkg_runic_text::TextEngine::new();
        engine.shape_layout(_spans, _max_width, _align, _overflow).ok()
    }
    fn draw_shaped_text(&mut self, shaped: &cvkg_runic_text::ShapedText, _x: f32, _y: f32) {
        let text = shaped.spans.iter().map(|s| s.text.as_str()).collect::<Vec<&str>>().join("");
        self.commands.push(format!("DrawText({})", text));
    }

    fn push_vnode(&mut self, _rect: Rect, name: &'static str) {
        self.commands.push(format!("PushVNode({})", name));
    }
    fn pop_vnode(&mut self) {
        self.commands.push("PopVNode".to_string());
    }
    fn set_key(&mut self, _key: &str) {}
    fn set_aria_role(&mut self, _role: &str) {}
    fn set_aria_label(&mut self, _label: &str) {}
    fn register_handler(
        &mut self,
        _event: &str,
        _handler: std::sync::Arc<dyn Fn(cvkg_core::Event) + Send + Sync>,
    ) {
    }
    fn memoize(&mut self, _id: u64, _data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer)) {
        render_fn(self);
    }
}

#[allow(non_snake_case)]
#[view_component]
fn ComplexDashboard(title: String, score: u32) {
    VStack::new(10.0)
        .alignment(cvkg_core::Alignment::Center)
        .child(Text::new(title).font_size(24.0))
        .child(Hvergelmir::new(50.0).color([0.0, 1.0, 0.0, 1.0]))
        .child(Button::new(format!("Score: {}", score), || {}))
}

#[test]
fn test_cross_crate_component_integration() {
    // 1. Build a complex component tree
    let view = VStack::new(10.0)
        .alignment(cvkg_core::Alignment::Center)
        .child(Text::new("System Alpha").font_size(24.0))
        .child(Hvergelmir::new(50.0).color([0.0, 1.0, 0.0, 1.0]))
        .child(Button::new("Score: 9000", || {}));

    // 2. Perform layout and render
    let mut renderer = MockRenderer::new();
    let root_rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    };

    // 3. Render and verify command stream
    view.render(&mut renderer, root_rect);

    // Verify all components were rendered in the stack
    assert!(
        renderer
            .commands
            .iter()
            .any(|c| c.contains("DrawText(System Alpha)"))
    );
    assert!(
        renderer
            .commands
            .iter()
            .any(|c| c.contains("FillPolygon(points: 6)"))
    );
    assert!(
        renderer
            .commands
            .iter()
            .any(|c| c.contains("DrawText(Score: 9000)"))
    );
}
