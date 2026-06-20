// Error Boundary Demo

use cvkg_components::{Color, ComponentErrorBoundary};
use cvkg_core::{ComponentErrorState, Rect, Renderer, State, View};
use cvkg_render_gpu::SurtrRenderer;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct ErrorApp {
    window: Option<Arc<Window>>,
    renderer: Option<SurtrRenderer>,
    error_state: State<ComponentErrorState>,
}

impl ApplicationHandler for ErrorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("CVKG Error Boundary Demo")
                        .with_inner_size(winit::dpi::LogicalSize::new(800u32, 600u32)),
                )
                .unwrap(),
        );

        let renderer = pollster::block_on(SurtrRenderer::forge(window.clone()));
        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let renderer = match self.renderer.as_mut() {
            Some(r) => r,
            None => return,
        };
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event: kb_event, ..
            } if kb_event.state.is_pressed() => {
                // Toggle error state on any key press
                let current = self.error_state.get();
                if current.has_error {
                    self.error_state.set(ComponentErrorState::clear());
                } else {
                    self.error_state.set(ComponentErrorState::error(
                        "Simulated System Fault in Bifrost Link",
                        "niflheim_demo::core_reactor",
                    ));
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let encoder = renderer.begin_frame(self.window.as_ref().unwrap().id());

                // ── Background ──
                renderer.fill_rect(
                    Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 800.0,
                        height: 600.0,
                    },
                    [0.02, 0.02, 0.03, 1.0],
                );

                // ── Sub-component with Error Boundary ──
                let error_rect = Rect {
                    x: 200.0,
                    y: 150.0,
                    width: 400.0,
                    height: 300.0,
                };

                // Normal content
                let content = Color::new(0.0, 0.8, 1.0, 0.5); // Cyan semi-transparent

                // Wrap in boundary
                let boundary = ComponentErrorBoundary::new(self.error_state.clone(), content);

                boundary.render(renderer, error_rect);

                renderer.draw_text(
                    "Press any key to toggle Error State",
                    280.0,
                    500.0,
                    18.0,
                    [0.8, 0.8, 0.9, 1.0],
                );

                renderer.end_frame(encoder);
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = ErrorApp {
        window: None,
        renderer: None,
        error_state: State::new(ComponentErrorState::clear()),
    };
    event_loop.run_app(&mut app).unwrap();
}
