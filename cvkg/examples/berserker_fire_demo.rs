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

fn draw_triangle_fuse(
    renderer: &mut SurtrRenderer,
    p1: [f32; 2],
    p2: [f32; 2],
    p3: [f32; 2],
    time: f32,
    speed: f32,
) {
    let pts = [p1, p2, p3, p1];

    // Draw the dim background triangle
    renderer.draw_line(p1[0], p1[1], p2[0], p2[1], [0.0, 0.4, 0.3, 0.35], 2.5);
    renderer.draw_line(p2[0], p2[1], p3[0], p3[1], [0.0, 0.4, 0.3, 0.35], 2.5);
    renderer.draw_line(p3[0], p3[1], p1[0], p1[1], [0.0, 0.4, 0.3, 0.35], 2.5);

    // Fuse animation: a bright traveling segment
    let total_len = 3.0f32;
    let head = (time * speed) % total_len;
    let tail_len = 0.9f32;
    let start = head - tail_len;

    // Draw the path from start to head
    let num_steps = 24;
    for i in 0..num_steps {
        let t1 = start + (i as f32 / num_steps as f32) * tail_len;
        let t2 = start + ((i + 1) as f32 / num_steps as f32) * tail_len;

        let p_start = get_triangle_point(&pts, t1);
        let p_end = get_triangle_point(&pts, t2);

        // Make it brighter towards the head (fuse tip)
        let alpha = (i as f32 / num_steps as f32).powi(2);
        let color = [0.0, 1.0, 0.8, alpha];
        let thickness = 2.0 + alpha * 2.5;
        renderer.draw_line(p_start[0], p_start[1], p_end[0], p_end[1], color, thickness);
    }

    // Draw a bright spark at the fuse head
    let spark_pos = get_triangle_point(&pts, head);
    renderer.draw_radial_gradient(
        Rect {
            x: spark_pos[0] - 6.0,
            y: spark_pos[1] - 6.0,
            width: 12.0,
            height: 12.0,
        },
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 1.0, 0.8, 0.0],
    );
}

fn get_triangle_point(pts: &[[f32; 2]; 4], mut t: f32) -> [f32; 2] {
    let total_len = 3.0f32;
    while t < 0.0 {
        t += total_len;
    }
    t = t % total_len;

    let segment_idx = t.floor() as usize;
    let local_t = t.fract();

    let p_start = pts[segment_idx];
    let p_end = pts[segment_idx + 1];

    [
        p_start[0] + (p_end[0] - p_start[0]) * local_t,
        p_start[1] + (p_end[1] - p_start[1]) * local_t,
    ]
}

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

#[derive(Clone, Copy)]
struct CardEffect {
    effect_type: u8, // 0 = none, 1 = slice, 2 = shatter
    progress: f32,
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
    card_effects: [CardEffect; 3],
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
            card_effects: [CardEffect {
                effect_type: 0,
                progress: 0.0,
            }; 3],
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
                button,
                ..
            } => {
                if button == MouseButton::Left {
                    self.state.total_clicks += 1;
                    self.state.rage = (self.state.total_clicks as f32 / 30.0).clamp(0.0, 1.0);
                }

                // Check card hits in logical coordinate space
                let size = window.inner_size();
                let sf = window.scale_factor() as f32;
                let w = size.width as f32 / sf;
                let h = size.height as f32 / sf;

                let card_width = 300.0;
                let card_height = 180.0;
                let total_cards_width = 3.0 * card_width;
                let remaining_width = w - total_cards_width;
                let spacing = remaining_width / 4.0;
                let card_y = (h - card_height) * 0.5;

                let mx = self.state.mouse_pos[0] / sf;
                let my = self.state.mouse_pos[1] / sf;

                for i in 0..3 {
                    let card_x = spacing + i as f32 * (card_width + spacing);
                    if mx >= card_x
                        && mx <= card_x + card_width
                        && my >= card_y
                        && my <= card_y + card_height
                    {
                        if button == MouseButton::Left {
                            self.state.card_effects[i].effect_type = 1; // slice
                            self.state.card_effects[i].progress = 0.0;
                        } else if button == MouseButton::Right {
                            self.state.card_effects[i].effect_type = 2; // shatter
                            self.state.card_effects[i].progress = 0.0;
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let size = window.inner_size();
                if size.width == 0 || size.height == 0 {
                    return;
                }

                let sf = window.scale_factor() as f32;
                let w = size.width as f32 / sf;
                let h = size.height as f32 / sf;
                let t = self.state.t();
                let dt = (t - self.state.last_time).max(0.0).min(0.1);
                self.state.last_time = t;
                self.state.frame_count += 1;
                self.state.rage = (self.state.rage - dt * 0.015).clamp(0.0, 1.0);

                // Tick card effects progress
                for effect in &mut self.state.card_effects {
                    if effect.effect_type != 0 {
                        effect.progress += dt * 1.5;
                        if effect.progress >= 1.0 {
                            effect.effect_type = 0;
                            effect.progress = 0.0;
                        }
                    }
                }

                // ── Frame Lifecycle ─────────────────────────────────────────
                let encoder = renderer.begin_frame(window.id());

                renderer.set_rage(self.state.rage);
                renderer.set_berserker_mode(self.state.berserker_mode());

                // ── Synthwave 80s Animated Background ──
                let horizon = h * 0.65;

                // 1. Sky vertical gradient (Deep Blue to Neon Purple)
                renderer.draw_linear_gradient(
                    Rect {
                        x: 0.0,
                        y: 0.0,
                        width: w,
                        height: horizon,
                    },
                    [0.02, 0.01, 0.08, 1.0],
                    [0.25, 0.02, 0.35, 1.0],
                    90.0,
                );

                // 3. Floor Grid (Deep Purple to Black gradient)
                renderer.draw_linear_gradient(
                    Rect {
                        x: 0.0,
                        y: horizon,
                        width: w,
                        height: h - horizon,
                    },
                    [0.08, 0.01, 0.15, 1.0],
                    [0.01, 0.0, 0.03, 1.0],
                    90.0,
                );

                // 4. Perspective vertical columns
                let num_grid_cols = 18;
                let neon_cyan = [0.0, 0.9, 0.9, 0.35];
                let vanish_x = w * 0.5;
                for i in 0..=num_grid_cols {
                    let fraction = (i as f32) / (num_grid_cols as f32);
                    let bottom_x = w * (-0.4 + fraction * 1.8); // Fan outwards
                    renderer.draw_line(vanish_x, horizon, bottom_x, h, neon_cyan, 1.5);
                }

                // 5. Scrolling horizontal lines (moves forward over time)
                let scroll_speed = 0.6;
                let time_offset = (t * scroll_speed).fract();
                let num_horiz_lines = 14;
                for i in 0..num_horiz_lines {
                    let k = (i as f32) + time_offset;
                    let depth = k / (num_horiz_lines as f32);
                    let y = horizon + depth * depth * (h - horizon); // Exponential perspective spacing
                    let alpha = depth * 0.45; // Fade out as it approaches horizon
                    renderer.draw_line(0.0, y, w, y, [0.0, 0.9, 0.9, alpha], 1.5);
                }

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

                    // Render particles as soft glowing points using radial gradients instead of hard quads
                    let size_multiplier = if p.is_ember { 2.5 } else { 2.0 };
                    let radius = p.size * size_multiplier;
                    let rect = Rect {
                        x: p.pos[0] - radius,
                        y: p.pos[1] - radius,
                        width: radius * 2.0,
                        height: radius * 2.0,
                    };
                    renderer.draw_radial_gradient(
                        rect,
                        c,
                        [p.color[0], p.color[1], p.color[2], 0.0],
                    );
                    p.life > 0.0
                });

                // ── Animated, Multi-Layered Burning Fire Ball ──
                // Layer 1: Large soft outer atmospheric heat distortion/glow
                let glow_size = 160.0 + (t * 2.0).sin() * 10.0;
                renderer.draw_radial_gradient(
                    Rect {
                        x: cx - glow_size * 0.5,
                        y: cy - glow_size * 0.5,
                        width: glow_size,
                        height: glow_size,
                    },
                    [1.0, 0.3, 0.0, 0.2],
                    [1.0, 0.1, 0.0, 0.0],
                );

                // Layer 2: Main burning flame body (wobbles organically)
                let flame_w = 110.0 + (t * 4.5).cos() * 8.0;
                let flame_h = 120.0 + (t * 3.7).sin() * 10.0;
                renderer.draw_radial_gradient(
                    Rect {
                        x: cx - flame_w * 0.5,
                        y: cy - flame_h * 0.5,
                        width: flame_w,
                        height: flame_h,
                    },
                    [1.0, 0.5, 0.05, 0.55],
                    [0.8, 0.1, 0.0, 0.0],
                );

                // Layer 3: Dynamic turbulent flame blobs (simulates burning material/lumps rising/shifting)
                for i in 0..3 {
                    let offset_angle = t * 3.0 + i as f32 * 2.09; // 120 deg spacing
                    let bx = cx + offset_angle.cos() * 15.0;
                    let by = cy + offset_angle.sin() * 12.0 - 15.0; // drift upward slightly
                    let blob_size = 45.0 + (t * 5.0 + i as f32).sin() * 6.0;
                    renderer.draw_radial_gradient(
                        Rect {
                            x: bx - blob_size * 0.5,
                            y: by - blob_size * 0.5,
                            width: blob_size,
                            height: blob_size,
                        },
                        [1.0, 0.7, 0.15, 0.7],
                        [0.9, 0.2, 0.0, 0.0],
                    );
                }

                // Layer 4: Hot white-hot core
                let core_size = 35.0 + (t * 8.0).cos() * 4.0;
                renderer.draw_radial_gradient(
                    Rect {
                        x: cx - core_size * 0.5,
                        y: cy - core_size * 0.5,
                        width: core_size,
                        height: core_size,
                    },
                    [1.0, 0.95, 0.6, 0.9],
                    [1.0, 0.4, 0.0, 0.0],
                );

                // ── Liquid Glass Cards ──
                let card_width = 300.0;
                let card_height = 180.0;
                let total_cards_width = 3.0 * card_width;
                let remaining_width = w - total_cards_width;
                let spacing = remaining_width / 4.0;
                let card_y = (h - card_height) * 0.5;

                for i in 0..3 {
                    let card_x = spacing + i as f32 * (card_width + spacing);
                    if i == 1 {
                        let cx = card_x + card_width * 0.5;
                        let cy = card_y - 70.0;
                        let s = 55.0;

                        // Triangle 1 (top, points pointing up)
                        let t1_p1 = [cx, cy - s * 0.9];
                        let t1_p2 = [cx + s * 0.5, cy + s * 0.1];
                        let t1_p3 = [cx - s * 0.5, cy + s * 0.1];

                        // Triangle 2 (bottom-right, points pointing up, shifted right and down)
                        let t2_p1 = [cx + s * 0.3, cy - s * 0.3];
                        let t2_p2 = [cx + s * 0.8, cy + s * 0.6];
                        let t2_p3 = [cx - s * 0.2, cy + s * 0.6];

                        // Triangle 3 (bottom-left, points pointing up, shifted left and down)
                        let t3_p1 = [cx - s * 0.3, cy - s * 0.3];
                        let t3_p2 = [cx + s * 0.2, cy + s * 0.6];
                        let t3_p3 = [cx - s * 0.8, cy + s * 0.6];

                        // Draw all three interlocking triangles with offset animation times so they trace continuously!
                        draw_triangle_fuse(renderer, t1_p1, t1_p2, t1_p3, t, 1.2);
                        draw_triangle_fuse(renderer, t2_p1, t2_p2, t2_p3, t + 1.0, 1.2);
                        draw_triangle_fuse(renderer, t3_p1, t3_p2, t3_p3, t + 2.0, 1.2);
                    }
                    let card_rect = Rect {
                        x: card_x,
                        y: card_y,
                        width: card_width,
                        height: card_height,
                    };
                    let effect = self.state.card_effects[i];

                    if effect.effect_type == 2 {
                        // Shatter effect: render shards of the glass card using mjolnir_shatter
                        // As progress goes from 0.0 to 1.0, the force increases from 0.0 to 6.0
                        let force = effect.progress * 6.0;
                        renderer.mjolnir_shatter(
                            card_rect,
                            256,
                            force,
                            [0.0, 1.0, 0.8, 1.0 - effect.progress],
                        );
                    } else {
                        // Draw normal card (or sliced card)
                        if effect.effect_type == 1 {
                            // Slice cut: split card into two halves sliding apart
                            let angle = 35.0f32;
                            let angle_rad = angle.to_radians();
                            let nx = angle_rad.cos();
                            let ny = angle_rad.sin();
                            let separation = effect.progress * 80.0; // Slide apart by up to 80 units

                            // Original card center for screen-space slice clipping
                            let orig_cx = card_rect.x + card_rect.width * 0.5;
                            let orig_cy = card_rect.y + card_rect.height * 0.5;
                            let d = orig_cx * nx + orig_cy * ny;

                            // Half 1: Shift positive along normal
                            let shift_x1 = nx * separation;
                            let shift_y1 = ny * separation;
                            renderer.push_mjolnir_slice(angle, d);
                            let rect1 = Rect {
                                x: card_rect.x + shift_x1,
                                y: card_rect.y + shift_y1,
                                width: card_rect.width,
                                height: card_rect.height,
                            };
                            renderer.bifrost(rect1, 16.0, 1.0, 0.4);
                            let (tw, th) = renderer.measure_text("CVKG !!!!", 28.0);
                            renderer.draw_text(
                                "CVKG !!!!",
                                rect1.x + (card_width - tw) * 0.5,
                                rect1.y + (card_height - th) * 0.5,
                                28.0,
                                [0.0, 1.0, 0.8, 1.0],
                            );
                            renderer.pop_mjolnir_slice();

                            // Half 2: Shift negative along normal
                            let shift_x2 = -nx * separation;
                            let shift_y2 = -ny * separation;
                            renderer.push_mjolnir_slice(angle + 180.0, -d);
                            let rect2 = Rect {
                                x: card_rect.x + shift_x2,
                                y: card_rect.y + shift_y2,
                                width: card_rect.width,
                                height: card_rect.height,
                            };
                            renderer.bifrost(rect2, 16.0, 1.0, 0.4);
                            renderer.draw_text(
                                "CVKG !!!!",
                                rect2.x + (card_width - tw) * 0.5,
                                rect2.y + (card_height - th) * 0.5,
                                28.0,
                                [0.0, 1.0, 0.8, 1.0],
                            );
                            renderer.pop_mjolnir_slice();
                        } else {
                            // Draw normal card
                            renderer.bifrost(card_rect, 16.0, 1.0, 0.4);
                            let (tw, th) = renderer.measure_text("CVKG !!!!", 28.0);
                            renderer.draw_text(
                                "CVKG !!!!",
                                card_x + (card_width - tw) * 0.5,
                                card_y + (card_height - th) * 0.5,
                                28.0,
                                [0.0, 1.0, 0.8, 1.0],
                            );
                        }
                    }
                }

                // Telemetry
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
                // Render larger telemetry at size 32
                renderer.fill_rect(
                    Rect {
                        x: 8.0,
                        y: 8.0,
                        width: w - 16.0,
                        height: 48.0,
                    },
                    [0.0, 0.0, 0.0, 0.7],
                );
                renderer.draw_text(&info, 18.0, 42.0, 32.0, [0.0, 1.0, 0.8, 1.0]);

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
