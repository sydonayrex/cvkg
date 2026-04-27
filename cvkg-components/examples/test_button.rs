use cvkg_core::{Never, Rect, Renderer};
use cvkg_macros::cvkg_component;

#[cvkg_component]
pub struct TestButton {
    pub label: String,
}

impl TestButton {
    pub fn new(label: impl Into<String>, _on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self::builder()
            .label(label.into())
            .build()
    }
}

impl cvkg_core::View for TestButton {
    type Body = Never;
    
    fn body(self) -> Self::Body {
        unreachable!()
    }
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "TestButton");
        renderer.set_key(&self.label);
        renderer.set_aria_role("button");
        renderer.set_aria_label(&self.label);
        
        // Background: dark panel
        let bg = [0.12, 0.12, 0.18, 1.0];
        renderer.fill_rounded_rect(rect, 6.0, bg);
        
        // Label text
        let text_x = rect.x + 8.0;
        let text_y = rect.y + (rect.height - 14.0) / 2.0;
        renderer.draw_text(&self.label, text_x, text_y, 14.0, [1.0, 1.0, 1.0, 1.0]);
        
        renderer.pop_vnode();
    }
}

fn main() {
    println!("TestButton example compiled successfully.");
}
