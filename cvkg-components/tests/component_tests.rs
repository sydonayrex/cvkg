use cvkg_core::{Renderer, Rect, View, ElapsedTime};
use cvkg_components::{Button, Text, VStack};

struct MockRenderer {
    commands: Vec<String>,
}

impl MockRenderer {
    fn new() -> Self {
        Self { commands: Vec::new() }
    }
}

impl ElapsedTime for MockRenderer {
    fn elapsed_time(&self) -> f32 { 0.0 }
    fn delta_time(&self) -> f32 { 0.0 }
}

impl Renderer for MockRenderer {
    fn fill_rect(&mut self, rect: Rect, _color: [f32; 4]) {
        self.commands.push(format!("FillRect({:?})", rect));
    }
    fn fill_rounded_rect(&mut self, rect: Rect, _radius: f32, _color: [f32; 4]) {
        self.commands.push(format!("FillRoundedRect({:?})", rect));
    }
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
    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        (text.len() as f32 * size * 0.6, size)
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
    fn memoize(&mut self, _id: u64, _data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer)) {
        render_fn(self);
    }
}

#[test]
fn test_button_rendering() {
    let mut renderer = MockRenderer::new();
    let button = Button::new("Submit", || {});
    let rect = Rect { x: 0.0, y: 0.0, width: 100.0, height: 40.0 };
    
    button.render(&mut renderer, rect);
    
    assert!(renderer.commands.iter().any(|c| c.contains("PushVNode(Button)")));
    assert!(renderer.commands.iter().any(|c| c.contains("DrawText(Submit)")));
    assert!(renderer.commands.contains(&"PopVNode".to_string()));
}

#[test]
fn test_vstack_rendering() {
    let mut renderer = MockRenderer::new();
    let vstack = VStack::new(10.0)
        .child(Text::new("Line 1"))
        .child(Text::new("Line 2"));
    let rect = Rect { x: 0.0, y: 0.0, width: 200.0, height: 100.0 };
    
    vstack.render(&mut renderer, rect);
    
    assert!(renderer.commands.iter().any(|c| c.contains("PushVNode(VStack)")));
    assert!(renderer.commands.iter().any(|c| c.contains("DrawText(Line 1)")));
    assert!(renderer.commands.iter().any(|c| c.contains("DrawText(Line 2)")));
}

#[test]
fn test_hvergelmir_rendering() {
    let mut renderer = MockRenderer::new();
    let hex = cvkg_components::Hvergelmir::new(100.0);
    let rect = Rect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 };
    
    hex.render(&mut renderer, rect);
    
    assert!(renderer.commands.iter().any(|c| c.contains("FillPolygon(points: 6)")));
    assert!(renderer.commands.iter().any(|c| c.contains("StrokePolygon(points: 6)")));
}

#[test]
fn test_skjaldborg_rendering() {
    let mut renderer = MockRenderer::new();
    let shield = cvkg_components::Skjaldborg::new([1.0, 0.0, 0.0, 1.0]);
    let rect = Rect { x: 0.0, y: 0.0, width: 200.0, height: 100.0 };
    
    shield.render(&mut renderer, rect);
    
    assert!(renderer.commands.iter().any(|c| c.contains("FillPolygon(points: 4)")));
}

#[test]
fn test_seiðr_rendering() {
    let mut renderer = MockRenderer::new();
    let effect = cvkg_components::Seiðr::default();
    let rect = Rect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 };
    
    effect.render(&mut renderer, rect);
    
    // Should have a rounded rect for background and some lines for scanlines
    assert!(renderer.commands.iter().any(|c| c.contains("FillRoundedRect")));
}

#[test]
fn test_lokiglitch_rendering() {
    let mut renderer = MockRenderer::new();
    let glitch = cvkg_components::LokiGlitch::new("ERROR");
    let rect = Rect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 };
    
    glitch.render(&mut renderer, rect);
    
    assert!(renderer.commands.iter().any(|c| c.contains("DrawText(ERROR)")));
}
