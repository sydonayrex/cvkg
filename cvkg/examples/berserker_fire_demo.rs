//! Berserker Fire — Desktop GPU Demo
//!
//! High-fidelity showcase of the CVKG Surtr GPU rendering pipeline.
//! Features:
//!   - Kvasir frame graph with multi-pass rendering (Geometry → Glass → UI → Post)
//!   - Dynamic particle system with ember/fire distinction
//!   - Berserker rage escalation: Normal → Rage → Frenzy → GodMode
//!   - Bifrost glassmorphic panels with Kawase blur pyramid
//!   - Mjolnir lightning bolts and shatter effects
//!   - Real-time telemetry overlay (frame time, draw calls, GPU time)
//!   - Mouse-driven shatter events and click-based rage escalation
//!
//! Run: cargo run -p cvkg --example berserker_fire_demo --features gpu

use cvkg_core::{
    BerserkerMode, ColorTheme, DrawMaterial, Rect, Renderer,
};
use cvkg_render_gpu::SurtrRenderer;
use std::sync::Arc;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

// ── Particle System ──────────────────────────────────────────────────────

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

// ── Demo State ───────────────────────────────────────────────────────────

struct DemoState {
    particles: Vec<Particle>,
    rng: Lcg,
    last_time: f32,
    bg_rotation: f32,
    bg_pos: [f32; 2],
    rage: f32,
    total_clicks: u32,
    shatter_cooldown: f32,
    scene_index: u32,
    frame_count: u64,
    start: Instant,
    mouse_pos: [f32; 2],
    window_width: f32,
    window_height: f32,
}

impl DemoState {
    fn new() -> Self {
        Self {
            particles: Vec::with_capacity(4096),
            rng: Lcg::new(1337),
            last_time: 0.0,
            bg_rotation: 0.0,
            bg_pos: [0.0, 0.0],
            rage: 0.0,
            total_clicks: 0,
            shatter_cooldown: 0.0,
            scene_index: 0,
            frame_count: 0,
            start: Instant::now(),
            mouse_pos: [0.0, 0.0],
            window_width: 1280.0,
            window_height: 720.0,
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

    fn theme(&self) -> ColorTheme {
        let base = ColorTheme::asgard();
        if self.rage < 0.01 {
            return base;
        }
        let r = self.rage.clamp(0.0, 1.0);
        ColorTheme {
            primary_neon: [
                base.primary_neon[0] + r * 0.5,
                base.primary_neon[1] * (1.0 - r * 0.3),
                base.primary_neon[2] * (1.0 - r * 0.5),
                base.primary_neon[3] + r * 0.5,
            ],
            shatter_neon: [1.0, r * 0.2, 0.75 * (1.0 - r * 0.3), 1.5 + r * 0.5],
            ember_core: [0.95 + r * 0.05, 0.12 * (1.0 - r * 0.5), 0.12 * (1.0 - r * 0.8), 1.0],
            background_deep: [0.01 + r * 0.02, 0.01, 0.03 * (1.0 - r * 0.3), 1.0],
            ..base
        }
    }
}

// ── Application ──────────────────────────────────────────────────────────

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

        self.state.window_width = 1280.0;
        self.state.window_height = 720.0;
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
                    self.state.window_width = new_size.width as f32;
                    self.state.window_height = new_size.height as f32;
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

                // Trigger shatter at click position
                if self.state.shatter_cooldown <= 0.0 {
                    self.state.shatter_cooldown = 0.2;
                    renderer.trigger_shatter_event(
                        self.state.mouse_pos,
                        self.state.rage * 2.0,
                    );
                }

                // Cycle scene on right-half clicks
                if self.state.mouse_pos[0] > self.state.window_width * 0.5 {
                    self.state.scene_index = (self.state.scene_index + 1) % 5;
                }
            }

            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                ..
            } => {
                // Right click: burst of particles at cursor
                for _ in 0..20 {
                    let angle = self.state.rng.next_f32() * 6.28;
                    let speed = 50.0 + self.state.rng.next_f32() * 150.0;
                    self.state.particles.push(Particle {
                        pos: self.state.mouse_pos,
                        vel: [angle.cos() * speed, angle.sin() * speed],
                        color: [1.0, 0.4 + self.state.rng.next_f32() * 0.6, 0.0, 1.0],
                        life: 0.5 + self.state.rng.next_f32() * 1.0,
                        size: 2.0 + self.state.rng.next_f32() * 8.0,
                        is_ember: self.state.rng.next_f32() > 0.5,
                    });
                }
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

                // Decay
                if self.state.shatter_cooldown > 0.0 {
                    self.state.shatter_cooldown -= dt;
                }
                self.state.rage = (self.state.rage - dt * 0.015).clamp(0.0, 1.0);

                // ── Frame Lifecycle ─────────────────────────────────────────
                let encoder = renderer.begin_frame(window.id());

                // Set pipeline state
                renderer.set_theme(self.state.theme());
                renderer.set_rage(self.state.rage);
                renderer.set_berserker_mode(self.state.berserker_mode());
                renderer.set_scene_preset(self.state.scene_index);

                let full_rect = Rect { x: 0.0, y: 0.0, width: w, height: h };

                // ── Background (Opaque pass via scene_type) ─────────────────
                // Scene type is set above; the background shader handles it
                renderer.draw_radial_gradient(full_rect, [0.01, 0.01, 0.03, 1.0], [0.0, 0.0, 0.0, 1.0]);

                // Floating CVKG text with transform stack
                draw_background(renderer, full_rect, &mut self.state, t);

                // ── Glassmorphic Cards (Glass pass) ─────────────────────────
                draw_glass_cards(renderer, w, h, t, self.state.rage);

                // ── Berserker Fire (Opaque + TopUI passes) ──────────────────
                draw_berserker_fire(renderer, &mut self.state, w, h, t, dt);

                // ── Telemetry Overlay (TopUI pass) ──────────────────────────
                draw_telemetry(renderer, &self.state, t);

                // ── Rage Vignette (TopUI pass) ──────────────────────────────
                if self.state.rage > 0.1 {
                    draw_rage_vignette(renderer, full_rect, self.state.rage, t);
                }

                renderer.end_frame(encoder);
                window.request_redraw();
            }
            _ => (),
        }
    }
}

// ── Background ───────────────────────────────────────────────────────────

fn draw_background(r: &mut SurtrRenderer, rect: Rect, s: &mut DemoState, _t: f32) {
    s.bg_rotation += 0.5 * 0.016;
    s.bg_pos[0] = (s.bg_pos[0] - 50.0 * 0.016 + rect.width) % rect.width;
    s.bg_pos[1] = (s.bg_pos[1] + 30.0 * 0.016) % rect.height;

    for i in 0..5 {
        let offset = i as f32 * 200.0;
        let x = (s.bg_pos[0] + offset) % rect.width;
        let y = (s.bg_pos[1] + offset * 0.5) % rect.height;
        let scale = 1.5 + (s.bg_rotation + i as f32).sin() * 0.3;

        r.push_transform([x, y], [scale, scale], s.bg_rotation + i as f32 * 0.2);
        r.push_shadow(8.0, [0.0, 0.8, 0.9, 0.4], [4.0, 4.0]);
        r.draw_text("CVKG", 4.0, 4.0, 64.0, [0.05, 0.05, 0.1, 0.3]);
        r.pop_shadow();
        r.draw_text("CVKG", 0.0, 0.0, 64.0, [0.1, 0.1, 0.2, 0.4]);
        r.pop_transform();
    }
}

// ── Glassmorphic Cards ───────────────────────────────────────────────────

fn draw_glass_cards(r: &mut SurtrRenderer, w: f32, h: f32, t: f32, rage: f32) {
    let card_w = 380.0;
    let card_h = 220.0;
    let positions = [
        [w * 0.15, h * 0.25],
        [w * 0.55, h * 0.15],
        [w * 0.35, h * 0.65],
    ];
    let runes = ["ᚢᛁᚴᛁᚿᚵ ᚦᚢᚿᛑᛂᚱ", "ᛒᛂᚱᛂᛌᛂᚱᚴᛂᚱ ᚠᛁᚱᛂ", "ᚴᚠᚴᚵ ᛑᛁᛌᛁᛑᚿᛂᛱ"];
    let labels = ["PROTOCOL_ACTIVE", "BERSERKER_FIRE", "CVKG_DISPATCH"];

    for (i, pos) in positions.iter().enumerate() {
        let x = pos[0] + (t * 0.5 + i as f32).sin() * 15.0;
        let y = pos[1] + (t * 0.3 + i as f32).cos() * 15.0;
        let rect = Rect { x, y, width: card_w, height: card_h };

        // Bifrost glass — routes to Glass pass, samples blur pyramid
        r.bifrost(rect, 40.0 + rage * 20.0, 1.2, 0.6);

        // Card backing
        r.fill_rounded_rect(rect, 20.0, [0.05, 0.05, 0.1, 0.2]);

        // Text — TopUI pass for crisp rendering
        r.set_material(DrawMaterial::TopUI);
        r.draw_text(runes[i], x + 30.0, y + 80.0, 28.0, [0.8, 0.9, 1.0, 1.0]);
        r.draw_text(labels[i], x + 30.0, y + 120.0, 12.0, [0.0, 0.8, 0.8, 0.8]);
        r.set_material(DrawMaterial::Opaque);

        // Neon glow during rage
        if rage > 0.2 {
            r.gungnir(rect, [1.0, 0.2, 0.5, 0.3 * rage], 10.0 + rage * 20.0, 0.5 * rage);
        }
    }
}

// ── Berserker Fire ───────────────────────────────────────────────────────

fn draw_berserker_fire(r: &mut SurtrRenderer, s: &mut DemoState, w: f32, h: f32, t: f32, dt: f32) {
    let cx = w * 0.5 + (t * 1.2).cos() * (w * 0.3);
    let cy = h * 0.5 + (t * 0.8).sin() * (h * 0.25);

    // Spawn particles
    let spawn = if s.rage > 0.5 { 6 } else { 3 };
    for _ in 0..spawn {
        let angle = s.rng.next_f32() * 6.28;
        let speed = 80.0 + s.rng.next_f32() * 180.0;
        let is_ember = s.rng.next_f32() > 0.3;
        let color = if is_ember {
            [1.0, 0.3 + s.rng.next_f32() * 0.5, 0.0, 1.0]
        } else {
            [1.0, 0.6 + s.rng.next_f32() * 0.4, 0.1, 1.0]
        };
        s.particles.push(Particle {
            pos: [cx, cy],
            vel: [angle.cos() * speed, angle.sin() * speed - 40.0],
            color,
            life: 0.8 + s.rng.next_f32() * 1.2,
            size: 2.0 + s.rng.next_f32() * 5.0,
            is_ember,
        });
    }

    // Update and draw
    r.set_material(DrawMaterial::Opaque);
    s.particles.retain_mut(|p| {
        p.pos[0] += p.vel[0] * dt;
        p.pos[1] += p.vel[1] * dt;
        p.life -= dt;
        let alpha = p.life.min(1.0).max(0.0);
        let c = [p.color[0], p.color[1], p.color[2], p.color[3] * alpha];
        let rect = Rect { x: p.pos[0], y: p.pos[1], width: p.size, height: p.size };
        if p.is_ember {
            r.fill_rect(rect, c);
        } else {
            r.fill_ellipse(rect, c);
        }
        p.life > 0.0
    });

    // Fire core
    let core = Rect { x: cx - 50.0, y: cy - 50.0, width: 100.0, height: 100.0 };
    r.draw_radial_gradient(core, [1.0, 0.8, 0.2, 1.0], [1.0, 0.2, 0.0, 0.0]);
    r.draw_radial_gradient(
        Rect { x: cx - 25.0, y: cy - 25.0, width: 50.0, height: 50.0 },
        [1.0, 1.0, 0.8, 1.0],
        [1.0, 0.5, 0.0, 0.0],
    );

    // Mani glow
    r.mani_glow(
        Rect { x: cx - 70.0, y: cy - 70.0, width: 140.0, height: 140.0 },
        [0.0, 0.8, 0.9, 0.3 + s.rage * 0.4],
        25.0 + s.rage * 35.0,
    );

    // Lightning at high rage
    if s.rage > 0.3 && s.rng.next_f32() > (0.93 - s.rage * 0.06) {
        let angle = s.rng.next_f32() * 6.28;
        let dist = 80.0 + s.rng.next_f32() * 250.0;
        r.draw_mjolnir_bolt(
            [cx, cy],
            [cx + angle.cos() * dist, cy + angle.sin() * dist],
            [0.6, 0.9, 1.0, 0.8 + s.rage * 0.2],
        );
    }

    // Shatter at very high rage
    if s.rage > 0.7 && s.rng.next_f32() > 0.95 {
        r.mjolnir_shatter(
            Rect { x: cx - 80.0, y: cy - 80.0, width: 160.0, height: 160.0 },
            8,
            s.rage,
            [1.0, 0.3, 0.5, 0.6],
        );
    }

    r.set_material(DrawMaterial::Opaque);
}

// ── Telemetry Overlay ────────────────────────────────────────────────────

fn draw_telemetry(r: &mut SurtrRenderer, s: &DemoState, _t: f32) {
    r.set_material(DrawMaterial::TopUI);
    r.set_z_index(1000.0);

    let mode = match s.berserker_mode() {
        BerserkerMode::Normal => "NORMAL",
        BerserkerMode::Rage => "RAGE",
        BerserkerMode::Frenzy => "FRENZY",
        BerserkerMode::GodMode => "GOD_MODE",
    };
    let rage_pct = (s.rage * 100.0) as u32;
    let info = format!(
        "CVKG v0.2.8 GPU | {} | RAGE: {}% | CLICKS: {} | FRAME: {} | PARTICLES: {}",
        mode, rage_pct, s.total_clicks, s.frame_count, s.particles.len()
    );

    // Background panel
    r.fill_rect(Rect { x: 8.0, y: 8.0, width: 540.0, height: 26.0 }, [0.0, 0.0, 0.0, 0.7]);
    r.draw_text(&info, 14.0, 26.0, 13.0, [0.0, 1.0, 0.8, 1.0]);

    // Rage bar
    let bx = 560.0;
    let by = 14.0;
    let bw = 180.0;
    let bh = 10.0;
    r.fill_rect(Rect { x: bx, y: by, width: bw, height: bh }, [0.1, 0.1, 0.1, 0.8]);
    let fill = bw * s.rage;
    let color = if s.rage > 0.7 {
        [1.0, 0.1, 0.2, 1.0]
    } else if s.rage > 0.4 {
        [1.0, 0.5, 0.0, 1.0]
    } else {
        [0.0, 0.8, 0.9, 1.0]
    };
    r.fill_rect(Rect { x: bx, y: by, width: fill, height: bh }, color);

    // Controls hint
    r.draw_text(
        "LMB: shatter + rage | RMB: particle burst | Right-half click: cycle scene",
        14.0,
        42.0,
        11.0,
        [0.5, 0.5, 0.6, 0.8],
    );

    r.set_z_index(0.0);
    r.set_material(DrawMaterial::Opaque);
}

// ── Rage Vignette ────────────────────────────────────────────────────────

fn draw_rage_vignette(r: &mut SurtrRenderer, rect: Rect, rage: f32, t: f32) {
    r.set_material(DrawMaterial::TopUI);
    let pulse = 0.5 + 0.5 * (t * 8.0 * rage).sin();
    let a = rage * 0.12 * pulse;
    let c = [0.8, 0.0, 0.1, a];

    let edge = 30.0;
    r.fill_rect(Rect { x: rect.x, y: rect.y, width: rect.width, height: edge }, c);
    r.fill_rect(Rect { x: rect.x, y: rect.height - edge, width: rect.width, height: edge }, c);
    r.fill_rect(Rect { x: rect.x, y: rect.y, width: edge, height: rect.height }, c);
    r.fill_rect(Rect { x: rect.width - edge, y: rect.y, width: edge, height: rect.height }, c);

    r.set_material(DrawMaterial::Opaque);
}

// ── Main ─────────────────────────────────────────────────────────────────

fn main() {
    println!("Forging Berserker Fire Desktop GPU Demo...");
    println!("Controls:");
    println!("  LMB: shatter event + rage escalation");
    println!("  RMB: particle burst at cursor");
    println!("  Right-half click: cycle scene preset (Aurora/Void/Nebula/Glitch/Yggdrasil)");

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
