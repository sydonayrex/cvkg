use cvkg_components::{Button, HStack, Slider, Spacer, Text, Input, Toggle, VStack};
use cvkg_core::{Color, View};
use std::sync::{Arc, Mutex};

struct DemoState {
    name: String,
    notifications: bool,
    volume: f32,
}

struct MainView {
    state: Arc<Mutex<DemoState>>,
}

impl View for MainView {
    type Body = VStack;

    fn body(self) -> Self::Body {
        let state = self.state.lock().unwrap();

        VStack::new(20.0)
            .child(
                Text::new("CVKG INTERACTIVE DEMO")
                    .font_size(32.0)
                    .color(Color::CYAN),
            )
            .child(
                Input::new("Enter your name")
                    .value(&state.name)
                    .on_change({
                        let state = self.state.clone();
                        move |new_name| {
                            state.lock().unwrap().name = new_name;
                        }
                    }),
            )
            .child(
                HStack::new(10.0)
                    .child(Text::new(format!(
                        "Hello, {}!",
                        if state.name.is_empty() {
                            "Guest"
                        } else {
                            &state.name
                        }
                    )))
                    .child(Spacer::new(0.0)),
            )
            .child(Toggle::new("Enable Notifications", state.notifications, {
                let state = self.state.clone();
                move |val| {
                    state.lock().unwrap().notifications = val;
                }
            }))
            .child(
                VStack::new(5.0)
                    .child(Text::new(format!(
                        "Volume: {}%",
                        (state.volume * 100.0) as i32
                    )))
                    .child(Slider::new(state.volume, 0.0..=1.0, {
                        let state = self.state.clone();
                        move |val| {
                            state.lock().unwrap().volume = val;
                        }
                    })),
            )
            .child(Button::new("Reset Settings", {
                let state = self.state.clone();
                move || {
                    let mut s = state.lock().unwrap();
                    s.name = "".to_string();
                    s.notifications = false;
                    s.volume = 0.5;
                }
            }))
    }
}

fn main() {
    let state = Arc::new(Mutex::new(DemoState {
        name: "".to_string(),
        notifications: true,
        volume: 0.7,
    }));

    // In a real app, we would wrap MainView in a VdomInspector for debugging
    // But VdomInspector is a component too!

    cvkg_render_native::NativeRenderer::run(MainView { state });
}
