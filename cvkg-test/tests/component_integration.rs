#![feature(impl_trait_in_assoc_type)]
use cvkg_core::{View, Rect, Renderer};
use cvkg_macros::view;
use cvkg_components::{Button, Text, VStack, Hvergelmir};

struct MockRenderer {
    commands: Vec<String>,
}

impl MockRenderer {
    fn new() -> Self {
        Self { commands: Vec::new() }
    }
}

impl cvkg_core::ElapsedTime for MockRenderer {
    fn elapsed_time(&self) -> f32 { 0.0 }
    fn delta_time(&self) -> f32 { 0.016 }
}

impl Renderer for MockRenderer {
    fn fill_rect(&mut self, _rect: Rect, _color: [f32; 4]) {}
    fn fill_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: [f32; 4]) {}
    fn fill_ellipse(&mut self, _rect: Rect, _color: [f32; 4]) {}
    fn stroke_rect(&mut self, _rect: Rect, _color: [f32; 4], _width: f32) {}
    fn stroke_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: [f32; 4], _width: f32) {}
    fn stroke_ellipse(&mut self, _rect: Rect, _color: [f32; 4], _width: f32) {}
    fn draw_line(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _color: [f32; 4], _width: f32) {}
    fn fill_polygon(&mut self, vertices: &[[f32; 2]], _color: [f32; 4]) {
        self.commands.push(format!("FillPolygon(points: {})", vertices.len()));
    }
    fn stroke_polygon(&mut self, vertices: &[[f32; 2]], _color: [f32; 4], _width: f32) {
        self.commands.push(format!("StrokePolygon(points: {})", vertices.len()));
    }
    fn draw_text(&mut self, text: &str, _x: f32, _y: f32, _size: f32, _color: [f32; 4]) {
        self.commands.push(format!("DrawText({})", text));
    }
    fn measure_text(&mut self, _text: &str, _size: f32) -> (f32, f32) { (50.0, 20.0) }
    
    fn push_vnode(&mut self, _rect: Rect, name: &str) {
        self.commands.push(format!("PushVNode({})", name));
    }
    fn pop_vnode(&mut self) {
        self.commands.push("PopVNode".to_string());
    }
}

#[view]
fn ComplexDashboard(title: String, score: u32) -> impl View {
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
    let root_rect = Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 };
    
    // 3. Render and verify command stream
    view.render(&mut renderer, root_rect);
    
    // Verify all components were rendered in the stack
    assert!(renderer.commands.iter().any(|c| c.contains("DrawText(System Alpha)")));
    assert!(renderer.commands.iter().any(|c| c.contains("FillPolygon(points: 6)")));
    assert!(renderer.commands.iter().any(|c| c.contains("DrawText(Score: 9000)")));
}
