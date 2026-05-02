use cvkg::prelude::*;
use cvkg_core::Renderer;
use std::sync::{Arc, Mutex};

struct HitTestState {
    click_count: u32,
    text: String,
    toggle: bool,
}

struct HitTestView {
    state: Arc<Mutex<HitTestState>>,
}

impl View for HitTestView {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let state = self.state.lock().unwrap();

        // Background
        renderer.fill_rect(rect, [0.02, 0.02, 0.05, 1.0]);

        // Title
        renderer.draw_text(
            "CVKG INTERACTION TEST SUITE",
            rect.x + 40.0,
            rect.y + 60.0,
            32.0,
            [0.0, 1.0, 1.0, 1.0], // CYAN
        );

        // Column Layout
        let content_x = rect.x + 40.0;

        // 1. Button Section
        let button_rect = Rect {
            x: content_x,
            y: rect.y + 120.0,
            width: 200.0,
            height: 50.0,
        };
        renderer.fill_rounded_rect(button_rect, 8.0, [0.1, 0.1, 0.2, 1.0]);
        renderer.stroke_rect(button_rect, [0.0, 0.8, 1.0, 1.0], 2.0);
        renderer.draw_text(
            "Click Me",
            button_rect.x + 50.0,
            button_rect.y + 32.0,
            18.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        let state_clone = self.state.clone();
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |_| {
                state_clone.lock().unwrap().click_count += 1;
            }),
        );

        renderer.draw_text(
            &format!("Clicks: {}", state.click_count),
            content_x + 220.0,
            rect.y + 152.0,
            18.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        // 2. TextField Section (Hardened)
        let field_rect = Rect {
            x: content_x,
            y: rect.y + 200.0,
            width: 300.0,
            height: 40.0,
        };
        let field_text = state.text.clone();
        let state_clone = self.state.clone();
        cvkg_components::Input::new("Enter command...")
            .value(field_text)
            .on_change(move |t| {
                state_clone.lock().unwrap().text = t;
            })
            .render(renderer, field_rect);

        // 3. Toggle Section
        let toggle_rect = Rect {
            x: content_x,
            y: rect.y + 270.0,
            width: 60.0,
            height: 30.0,
        };
        let toggle_val = state.toggle;
        let state_clone = self.state.clone();
        cvkg_components::Toggle::new("Toggle", toggle_val, move |v| {
            state_clone.lock().unwrap().toggle = v;
        })
        .render(renderer, toggle_rect);

        renderer.draw_text(
            if state.toggle {
                "SYSTEM ONLINE"
            } else {
                "SYSTEM OFFLINE"
            },
            content_x + 80.0,
            rect.y + 292.0,
            18.0,
            if state.toggle {
                [0.0, 1.0, 0.0, 1.0]
            } else {
                [1.0, 0.0, 0.0, 1.0]
            },
        );
    }
}

fn main() {
    let state = Arc::new(Mutex::new(HitTestState {
        click_count: 0,
        text: String::new(),
        toggle: false,
    }));
    cvkg_render_native::NativeRenderer::run(HitTestView { state });
}
