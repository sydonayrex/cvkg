// This example requires the GPU feature to be enabled

#[cfg(feature = "gpu")]
use cvkg::render::SurtrRenderer;

#[cfg(feature = "gpu")]
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

#[cfg(feature = "gpu")]
struct ShatterApp {
    shattered: bool,
    force: f32,
    strike_start: Option<std::time::Instant>,
}

#[cfg(feature = "gpu")]
impl View for ShatterApp {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Background
        renderer.fill_rect(rect, [0.05, 0.05, 0.1, 1.0]);

        let center_rect = Rect {
            x: rect.width / 2.0 - 100.0,
            y: rect.height / 2.0 - 100.0,
            width: 200.0,
            height: 200.0,
        };

        if let Some(start) = self.strike_start {
            let elapsed = start.elapsed().as_secs_f32();
            if elapsed < 0.15 {
                // High-Fidelity Electric Storm (Dozens of bolts)
                let center_x = rect.width / 2.0;
                let center_y = rect.height / 2.0;

                for i in 0..8 {
                    // Reduced from 12 to 8 for better performance
                    let seed = (i as f32 * 0.77 + elapsed * 10.0).sin().fract();
                    let start_x = center_x + (seed - 0.5) * rect.width;
                    let target_pos = [
                        center_x + (seed - 0.5) * 80.0,
                        center_y + (seed - 0.5) * 80.0,
                    ];

                    renderer.draw_mjolnir_bolt([start_x, 0.0], target_pos, [0.0, 1.0, 1.0, 1.0]);

                    // High-Fidelity Impact Sparks (Optimized density & color)
                    let spark_rect = Rect {
                        x: target_pos[0] - 2.0,
                        y: target_pos[1] - 2.0,
                        width: 4.0,
                        height: 4.0,
                    };
                    renderer.mjolnir_shatter(spark_rect, 64, 4.0, [0.0, 1.0, 1.0, 1.0]);
                }
            }
        }

        if self.shattered {
            // Apply high-fidelity fluid & jiggle shatter effect (1024 shards)
            renderer.mjolnir_fluid_shatter(center_rect, 1024, self.force, [0.0, 0.8, 1.0, 1.0]);

            let (tw, th) = renderer.measure_text("BERSERKER UNLEASHED", 24.0);
            renderer.draw_text(
                "BERSERKER UNLEASHED",
                rect.width / 2.0 - tw / 2.0,
                rect.height - 50.0 - th / 2.0,
                24.0,
                [0.0, 1.0, 1.0, 1.0],
            );
        } else {
            // Draw a solid glowing shield
            renderer.fill_rounded_rect(center_rect, 20.0, [0.0, 0.8, 1.0, 1.0]);

            let (tw, th) = renderer.measure_text("CLICK TO SHATTER", 20.0);
            renderer.draw_text(
                "CLICK TO SHATTER",
                rect.width / 2.0 - tw / 2.0,
                rect.height / 2.0 - th / 2.0, // Top-Left anchored centering
                20.0,
                [1.0, 1.0, 1.0, 1.0],
            );
        }
    }
}

#[cfg(feature = "gpu")]
struct AppState {
    window: Option<Arc<Window>>,
    renderer: Option<SurtrRenderer>,
    app: ShatterApp,
}

#[cfg(feature = "gpu")]
impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("CVKG - Mjolnir Shatter Demo")
                        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0)),
                )
                .unwrap(),
        );

        let renderer = pollster::block_on(SurtrRenderer::forge(window.clone()));
        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let window = self.window.as_ref().unwrap();
        let renderer = self.renderer.as_mut().unwrap();

        match event {
            WindowEvent::Resized(new_size) => {
                renderer.resize(window.id(), new_size.width, new_size.height, 1.0);
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.app.shattered = true;
                self.app.force = 2.0;
                self.app.strike_start = Some(std::time::Instant::now());
                renderer.reset_time();
            }
            WindowEvent::RedrawRequested => {
                let size = window.inner_size();
                if size.width > 0 && size.height > 0 {
                    let rect = Rect {
                        x: 0.0,
                        y: 0.0,
                        width: size.width as f32,
                        height: size.height as f32,
                    };

                    let encoder = renderer.begin_frame(window.id());
                    self.app.render(renderer, rect);
                    renderer.end_frame(encoder);
                }

                window.request_redraw();
            }
            _ => (),
        }
    }
}

#[cfg(feature = "gpu")]
fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut state = AppState {
        window: None,
        renderer: None,
        app: ShatterApp {
            shattered: false,
            force: 0.0,
            strike_start: None,
        },
    };
    event_loop.run_app(&mut state).unwrap();
}

#[cfg(not(feature = "gpu"))]
fn main() {
    println!("This example requires the 'gpu' feature to be enabled.");
}
