//! Berserker Fire — Desktop GPU Demo
//!
//! Run: cargo run -p cvkg --example berserker_fire_demo --features gpu

use cvkg_core::{BerserkerMode, DrawMaterial, FrameRenderer, Rect, Renderer};
use cvkg_render_gpu::SurtrRenderer;
use std::sync::Arc;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct Particle {
    pos: [f32; 2],
    vel: [f32; 2],
    color: [f32; 4],
    life: f32,
    size: f32,
    is_ember: bool,
}

struct Lcg {
    state: u32,
}

impl Lcg {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }
    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        (self.state & 0x7FFFFFFF) as f32 / 2147483647.0
    }
}

struct DemoState {
    particles: Vec<Particle>,
    rng: Lcg,
    last_time: f32,
    rage: f32,
    total_clicks: u32,
    frame_count: u64,
    start: Instant,
    mouse_pos: [f32; 2],
}

impl DemoState {
    fn new() -> Self {
        Self {
            particles: Vec::with_capacity(4096),
            rng: Lcg::new(1337),
            last_time: 0.0,
            rage: 0.0,
            total_clicks: 0,
            frame_count: 0,
            start: Instant::now(),
            mouse_pos: [0.0, 0.0],
        }
    }

    fn t(&self) -> f32 {
        self.start.elapsed().as_secs_f32()
    }

    fn berserker_mode(&self) -> BerserkerMode {
        if self.rage > 0.8 {
            BerserkerMode::GodMode
        } else if self.rage > 0.5 {
            BerserkerMode::Frenzy
        } else if self.rage > 0.2 {
            BerserkerMode::Rage
        } else {
            BerserkerMode::Normal
        }
    }
}

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<SurtrRenderer>,
    state: DemoState,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            state: DemoState::new(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("CVKG Berserker Fire — Desktop GPU Demo")
                        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0)),
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
                if new_size.width > 0 && new_size.height > 0 {
                    renderer.resize(
                        window.id(),
                        new_size.width,
                        new_size.height,
                        window.scale_factor() as f32,
                    );
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::CursorMoved { position, .. } => {
                self.state.mouse_pos = [position.x as f32, position.y as f32];
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.state.total_clicks += 1;
                self.state.rage = (self.state.total_clicks as f32 / 30.0).clamp(0.0, 1.0);
            }
            WindowEvent::RedrawRequested => {
                let size = window.inner_size();
                if size.width == 0 || size.height == 0 {
                    return;
                }

                let w = size.width as f32;
                let h = size.height as f32;
                let t = self.state.t();
                let dt = (t - self.state.last_time).max(0.0).min(0.1);
                self.state.last_time = t;
                self.state.frame_count += 1;
                self.state.rage = (self.state.rage - dt * 0.015).clamp(0.0, 1.0);

                // ── Frame Lifecycle ─────────────────────────────────────────
                let encoder = renderer.begin_frame(window.id());

                renderer.set_rage(self.state.rage);
                renderer.set_berserker_mode(self.state.berserker_mode());

                let full_rect = Rect {
                    x: 0.0,
                    y: 0.0,
                    width: w,
                    height: h,
                };

                // Background
                renderer.fill_rect(full_rect, [0.02, 0.02, 0.05, 1.0]);

                // Fire core
                let cx = w * 0.5 + (t * 1.2).cos() * (w * 0.3);
                let cy = h * 0.5 + (t * 0.8).sin() * (h * 0.25);

                // Spawn particles
                let spawn = if self.state.rage > 0.5 { 8 } else { 4 };
                for _ in 0..spawn {
                    let angle = self.state.rng.next_f32() * 6.28;
                    let speed = 80.0 + self.state.rng.next_f32() * 180.0;
                    let is_ember = self.state.rng.next_f32() > 0.3;
                    let color = if is_ember {
                        [1.0, 0.3 + self.state.rng.next_f32() * 0.5, 0.0, 1.0]
                    } else {
                        [1.0, 0.6 + self.state.rng.next_f32() * 0.4, 0.1, 1.0]
                    };
                    self.state.particles.push(Particle {
                        pos: [cx, cy],
                        vel: [angle.cos() * speed, angle.sin() * speed - 40.0],
                        color,
                        life: 0.8 + self.state.rng.next_f32() * 1.2,
                        size: 3.0 + self.state.rng.next_f32() * 6.0,
                        is_ember,
                    });
                }

                // Update and draw particles
                renderer.set_material(DrawMaterial::Opaque);
                self.state.particles.retain_mut(|p| {
                    p.pos[0] += p.vel[0] * dt;
                    p.pos[1] += p.vel[1] * dt;
                    p.life -= dt;
                    let alpha = p.life.min(1.0).max(0.0);
                    let c = [p.color[0], p.color[1], p.color[2], p.color[3] * alpha];
                    let rect = Rect {
                        x: p.pos[0],
                        y: p.pos[1],
                        width: p.size,
                        height: p.size,
                    };
                    if p.is_ember {
                        renderer.fill_rect(rect, c);
                    } else {
                        renderer.fill_ellipse(rect, c);
                    }
                    p.life > 0.0
                });

                // Fire glow
                renderer.draw_radial_gradient(
                    Rect {
                        x: cx - 60.0,
                        y: cy - 60.0,
                        width: 120.0,
                        height: 120.0,
                    },
                    [1.0, 0.8, 0.2, 0.8],
                    [1.0, 0.2, 0.0, 0.0],
                );

                // Telemetry
                renderer.set_material(DrawMaterial::TopUI);
                let mode = match self.state.berserker_mode() {
                    BerserkerMode::Normal => "NORMAL",
                    BerserkerMode::Rage => "RAGE",
                    BerserkerMode::Frenzy => "FRENZY",
                    BerserkerMode::GodMode => "GOD_MODE",
                };
                let info = format!(
                    "CVKG v0.2.8 GPU | {} | RAGE: {}% | CLICKS: {} | FRAME: {} | PARTICLES: {}",
                    mode,
                    (self.state.rage * 100.0) as u32,
                    self.state.total_clicks,
                    self.state.frame_count,
                    self.state.particles.len()
                );
                renderer.fill_rect(
                    Rect {
                        x: 8.0,
                        y: 8.0,
                        width: 500.0,
                        height: 24.0,
                    },
                    [0.0, 0.0, 0.0, 0.7],
                );
                renderer.draw_text(&info, 14.0, 24.0, 12.0, [0.0, 1.0, 0.8, 1.0]);
                renderer.set_material(DrawMaterial::Opaque);

                // Upload vertex data to GPU before render passes
                renderer.render_frame();
                renderer.end_frame(encoder);

                window.request_redraw();
            }
            _ => (),
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    println!("Forging Berserker Fire Desktop GPU Demo...");
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
