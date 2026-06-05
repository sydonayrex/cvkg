#![allow(
    dead_code,
    unused_imports,
    clippy::approx_constant,
    clippy::manual_clamp
)]

use cvkg_core::{
    BerserkerMode, ColorTheme, ElapsedTime, FrameRenderer, Rect, Renderer,
};
use cvkg_render_web::WebRenderer;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

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

struct BerserkerState {
    counters: [u32; 4],
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
}

impl BerserkerState {
    fn new() -> Self {
        Self {
            counters: [0; 4],
            particles: Vec::with_capacity(2048),
            rng: Lcg::new(1337),
            last_time: 0.0,
            bg_rotation: 0.0,
            bg_pos: [0.0, 0.0],
            rage: 0.0,
            total_clicks: 0,
            shatter_cooldown: 0.0,
            scene_index: 0,
            frame_count: 0,
        }
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
        // Lerp toward rage colors as intensity increases
        let r = self.rage.clamp(0.0, 1.0);
        ColorTheme {
            primary_neon: [
                base.primary_neon[0] + r * 0.5,
                base.primary_neon[1] * (1.0 - r * 0.3),
                base.primary_neon[2] * (1.0 - r * 0.5),
                base.primary_neon[3] + r * 0.5,
            ],
            shatter_neon: [
                1.0,
                0.0 + r * 0.2,
                0.75 * (1.0 - r * 0.3),
                1.5 + r * 0.5,
            ],
            ember_core: [
                0.95 + r * 0.05,
                0.12 * (1.0 - r * 0.5),
                0.12 * (1.0 - r * 0.8),
                1.0,
            ],
            background_deep: [
                0.01 + r * 0.02,
                0.01,
                0.03 * (1.0 - r * 0.3),
                1.0,
            ],
            ..base
        }
    }
}

// ── Main Entry Point ─────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info)
        .expect("error initializing log");

    log::info!("Berserker Fire Demo — CVKG v0.2.8");

    let mut renderer = WebRenderer::new();
    renderer.forge().await?;

    let state = Arc::new(Mutex::new(BerserkerState::new()));
    let renderer_rc = Rc::new(RefCell::new(renderer));

    // ── Click Handling ──────────────────────────────────────────────────
    let canvas = renderer_rc.borrow().canvas().unwrap().clone();
    let state_click = state.clone();
    let on_click = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        let x = event.offset_x() as f32;
        let y = event.offset_y() as f32;
        let mut s = state_click.lock().unwrap();

        let canvas_w = 1888.0;
        let canvas_h = 951.0;
        let btn_size = 100.0;
        let padding = 20.0;

        // Corner buttons
        if x >= padding && x < padding + btn_size && y >= padding && y < padding + btn_size {
            s.counters[0] += 1;
            s.total_clicks += 1;
        }
        if x >= canvas_w - btn_size - padding
            && x < canvas_w - padding
            && y >= padding
            && y < padding + btn_size
        {
            s.counters[1] += 1;
            s.total_clicks += 1;
        }
        if x >= padding
            && x < padding + btn_size
            && y >= canvas_h - btn_size - padding
            && y < canvas_h - padding
        {
            s.counters[2] += 1;
            s.total_clicks += 1;
        }
        if x >= canvas_w - btn_size - padding
            && x < canvas_w - padding
            && y >= canvas_h - btn_size - padding
            && y < canvas_h - padding
        {
            s.counters[3] += 1;
            s.total_clicks += 1;
        }

        // Rage escalation: every 10 clicks increases rage
        s.rage = (s.total_clicks as f32 / 50.0).clamp(0.0, 1.0);

        // Trigger shatter on click (with cooldown)
        if s.shatter_cooldown <= 0.0 {
            s.shatter_cooldown = 0.3;
        }
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);

    canvas
        .add_event_listener_with_callback("mousedown", on_click.as_ref().unchecked_ref())?;
    on_click.forget();

    // ── Render Loop ─────────────────────────────────────────────────────
    let loop_f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let loop_g = loop_f.clone();
    let state_loop = state.clone();
    let renderer_loop = renderer_rc.clone();

    *loop_g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut r = renderer_loop.borrow_mut();
        let mut s = state_loop.lock().unwrap();

        let t = r.elapsed_time();
        let dt = (t - s.last_time).max(0.0).min(0.1);
        s.last_time = t;
        s.frame_count += 1;

        // Decay shatter cooldown
        if s.shatter_cooldown > 0.0 {
            s.shatter_cooldown -= dt;
        }

        // Decay rage slowly
        s.rage = (s.rage - dt * 0.02).clamp(0.0, 1.0);

        // ── Frame Lifecycle ─────────────────────────────────────────────
        r.begin_frame();

        let width = 1888.0;
        let height = 951.0;
        let full_rect = Rect {
            x: 0.0,
            y: 0.0,
            width,
            height,
        };

        // Set pipeline state
        r.set_theme(s.theme());
        r.set_rage(s.rage);
        r.set_berserker_mode(s.berserker_mode());
        r.set_scene_preset(s.scene_index);

        // Trigger shatter if cooldown just expired this frame
        if s.shatter_cooldown > 0.0 && s.shatter_cooldown < 0.05 && s.rage > 0.3 {
            r.trigger_shatter_event([width * 0.5, height * 0.5], s.rage * 2.0);
        }

        // ── Background (Opaque pass) ────────────────────────────────────
        s.bg_rotation += dt * 0.5;
        s.bg_pos[0] = (s.bg_pos[0] - dt * 50.0 + width) % width;
        s.bg_pos[1] = (s.bg_pos[1] + dt * 30.0) % height;

        draw_background(&mut *r, full_rect, s.bg_rotation, s.bg_pos, t);

        // ── Glassmorphic Cards (Glass pass) ─────────────────────────────
        draw_glass_cards(&mut *r, width, height, t, s.rage);

        // ── Berserker Fire (Opaque + TopUI passes) ──────────────────────
        draw_berserker_fire(&mut *r, &mut *s, width, height, t, dt);

        // ── Corner Buttons (TopUI pass) ─────────────────────────────────
        draw_corner_buttons(&mut *r, &s.counters, width, height);

        // ── Telemetry Overlay (TopUI pass) ──────────────────────────────
        draw_telemetry(&mut *r, &s, t);

        // ── Rage Vignette (TopUI pass) ──────────────────────────────────
        if s.rage > 0.1 {
            draw_rage_vignette(&mut *r, full_rect, s.rage, t);
        }

        r.end_frame(());

        web_sys::window()
            .unwrap()
            .request_animation_frame(
                loop_f.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
            )
            .expect("should register `requestAnimationFrame` OK");
    }) as Box<dyn FnMut()>));

    web_sys::window()
        .unwrap()
        .request_animation_frame(
            loop_g.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
        )?;

    Ok(())
}

// ── Background ───────────────────────────────────────────────────────────

fn draw_background(
    r: &mut dyn Renderer,
    rect: Rect,
    rotation: f32,
    pos: [f32; 2],
    _t: f32,
) {
    // Deep space background with radial gradient
    r.draw_radial_gradient(
        rect,
        [0.01, 0.01, 0.03, 1.0],
        [0.0, 0.0, 0.0, 1.0],
    );

    // Floating "CVKG" text with transform stack
    for i in 0..5 {
        let offset = i as f32 * 200.0;
        let x = (pos[0] + offset) % rect.width;
        let y = (pos[1] + offset * 0.5) % rect.height;
        let scale = 1.5 + (rotation + i as f32).sin() * 0.3;

        // Use transform stack for proper 2D rotation/scale
        r.push_transform(
            [x, y],
            [scale, scale],
            rotation + i as f32 * 0.2,
        );

        // Shadow layer (pushed behind)
        r.push_shadow(8.0, [0.0, 0.8, 0.9, 0.4], [4.0, 4.0]);
        r.draw_text("CVKG", 4.0, 4.0, 64.0, [0.05, 0.05, 0.1, 0.3]);
        r.pop_shadow();

        // Main text
        r.draw_text("CVKG", 0.0, 0.0, 64.0, [0.1, 0.1, 0.2, 0.4]);

        r.pop_transform();
    }
}

// ── Glassmorphic Cards ───────────────────────────────────────────────────

fn draw_glass_cards(r: &mut dyn Renderer, w: f32, h: f32, t: f32, rage: f32) {
    let card_w = 400.0;
    let card_h = 250.0;
    let card_positions = [
        [w * 0.2, h * 0.3],
        [w * 0.7, h * 0.2],
        [w * 0.5, h * 0.7],
    ];
    let runes = [
        "ᚢᛁᚴᛁᚿᚵ ᚦᚢᚿᛑᛂᚱ",
        "ᛒᛂᚱᛂᛌᛂᚱᚴᛂᚱ ᚠᛁᚱᛂ",
        "ᚴᚠᚴᚵ ᛑᛁᛌᛁᛑᚿᛂᛱ",
    ];
    let labels = [
        "PROTOCOL_ACTIVE",
        "BERSERKER_FIRE",
        "CVKG_DISPATCH",
    ];

    for (i, pos) in card_positions.iter().enumerate() {
        let x = pos[0] + (t * 0.5 + i as f32).sin() * 20.0;
        let y = pos[1] + (t * 0.3 + i as f32).cos() * 20.0;
        let rect = Rect {
            x,
            y,
            width: card_w,
            height: card_h,
        };

        // Bifrost glass effect — routes to Glass pass
        let blur = 40.0 + rage * 20.0;
        r.bifrost(rect, blur, 1.2, 0.6);

        // Card backing — opaque pass
        r.fill_rounded_rect(rect, 24.0, [0.05, 0.05, 0.1, 0.2]);

        // Rune text — TopUI pass for crisp rendering
        r.set_material(cvkg_core::DrawMaterial::TopUI);
        r.draw_text(runes[i], x + 40.0, y + 100.0, 32.0, [0.8, 0.9, 1.0, 1.0]);
        r.draw_text(labels[i], x + 40.0, y + 140.0, 14.0, [0.0, 0.8, 0.8, 0.8]);

        // Neon edge glow using gungnir
        if rage > 0.2 {
            r.gungnir(
                rect,
                [1.0, 0.2, 0.5, 0.3 * rage],
                10.0 + rage * 20.0,
                0.5 * rage,
            );
        }

        // Reset to opaque
        r.set_material(cvkg_core::DrawMaterial::Opaque);
    }
}

// ── Berserker Fire ───────────────────────────────────────────────────────

fn draw_berserker_fire(
    r: &mut dyn Renderer,
    s: &mut BerserkerState,
    w: f32,
    h: f32,
    t: f32,
    dt: f32,
) {
    let cx = w * 0.5 + (t * 1.2).cos() * (w * 0.3);
    let cy = h * 0.5 + (t * 0.8).sin() * (h * 0.25);

    // Spawn particles
    let spawn_count = if s.rage > 0.5 { 8 } else { 5 };
    for _ in 0..spawn_count {
        let angle = s.rng.next_f32() * 6.28;
        let speed = 100.0 + s.rng.next_f32() * 200.0;
        let is_ember = s.rng.next_f32() > 0.3;
        let base_color = if is_ember {
            [1.0, 0.3 + s.rng.next_f32() * 0.5, 0.0, 1.0]
        } else {
            [1.0, 0.6 + s.rng.next_f32() * 0.4, 0.1, 1.0]
        };
        s.particles.push(Particle {
            pos: [cx, cy],
            vel: [angle.cos() * speed, angle.sin() * speed - 50.0],
            color: base_color,
            life: 1.0 + s.rng.next_f32() * 1.5,
            size: 2.0 + s.rng.next_f32() * 6.0,
            is_ember,
        });
    }

    // Update and draw particles
    r.set_material(cvkg_core::DrawMaterial::Opaque);
    s.particles.retain_mut(|p| {
        p.pos[0] += p.vel[0] * dt;
        p.pos[1] += p.vel[1] * dt;
        p.life -= dt;

        let alpha = p.life.min(1.0).max(0.0);
        let p_color = [p.color[0], p.color[1], p.color[2], p.color[3] * alpha];

        if p.is_ember {
            r.fill_rect(
                Rect {
                    x: p.pos[0],
                    y: p.pos[1],
                    width: p.size,
                    height: p.size,
                },
                p_color,
            );
        } else {
            r.fill_ellipse(
                Rect {
                    x: p.pos[0],
                    y: p.pos[1],
                    width: p.size,
                    height: p.size,
                },
                p_color,
            );
        }
        p.life > 0.0
    });

    // Fire core glow — uses radial gradients
    let fire_rect = Rect {
        x: cx - 60.0,
        y: cy - 60.0,
        width: 120.0,
        height: 120.0,
    };
    r.draw_radial_gradient(fire_rect, [1.0, 0.8, 0.2, 1.0], [1.0, 0.2, 0.0, 0.0]);
    r.draw_radial_gradient(
        Rect {
            x: cx - 30.0,
            y: cy - 30.0,
            width: 60.0,
            height: 60.0,
        },
        [1.0, 1.0, 0.8, 1.0],
        [1.0, 0.5, 0.0, 0.0],
    );

    // Mani glow around fire center
    r.mani_glow(
        Rect {
            x: cx - 80.0,
            y: cy - 80.0,
            width: 160.0,
            height: 160.0,
        },
        [0.0, 0.8, 0.9, 0.3 + s.rage * 0.4],
        30.0 + s.rage * 40.0,
    );

    // Lightning bolts at high rage
    if s.rage > 0.3 && s.rng.next_f32() > (0.92 - s.rage * 0.05) {
        let angle = s.rng.next_f32() * 6.28;
        let dist = 100.0 + s.rng.next_f32() * 300.0;
        let tx = cx + angle.cos() * dist;
        let ty = cy + angle.sin() * dist;
        r.draw_mjolnir_bolt(
            [cx, cy],
            [tx, ty],
            [0.6, 0.9, 1.0, 0.8 + s.rage * 0.2],
        );
    }

    // Shatter effect at very high rage
    if s.rage > 0.7 && s.rng.next_f32() > 0.95 {
        r.mjolnir_shatter(
            Rect {
                x: cx - 100.0,
                y: cy - 100.0,
                width: 200.0,
                height: 200.0,
            },
            8,
            s.rage,
            [1.0, 0.3, 0.5, 0.6],
        );
    }

    r.set_material(cvkg_core::DrawMaterial::Opaque);
}

// ── Corner Buttons ───────────────────────────────────────────────────────

fn draw_corner_buttons(r: &mut dyn Renderer, counters: &[u32; 4], w: f32, h: f32) {
    let btn_size = 100.0;
    let padding = 20.0;
    let corners = [
        (padding, padding, "I"),
        (w - btn_size - padding, padding, "II"),
        (padding, h - btn_size - padding, "III"),
        (w - btn_size - padding, h - btn_size - padding, "IV"),
    ];

    r.set_material(cvkg_core::DrawMaterial::TopUI);

    for (i, corner) in corners.iter().enumerate() {
        let x = corner.0;
        let y = corner.1;
        let rect = Rect {
            x,
            y,
            width: btn_size,
            height: btn_size,
        };

        // Button shadow
        r.push_shadow(6.0, [0.0, 0.0, 0.0, 0.5], [2.0, 2.0]);
        r.fill_rounded_rect(rect, 12.0, [0.2, 0.2, 0.3, 0.8]);
        r.pop_shadow();

        // Button label
        r.draw_text(corner.2, x + 35.0, y + 60.0, 32.0, [1.0, 1.0, 1.0, 1.0]);

        // Counter
        let count_str = format!("{}", counters[i]);
        r.draw_text(
            &count_str,
            x + btn_size + 10.0,
            y + 60.0,
            24.0,
            [0.0, 1.0, 0.5, 1.0],
        );
    }

    r.set_material(cvkg_core::DrawMaterial::Opaque);
}

// ── Telemetry Overlay ────────────────────────────────────────────────────

fn draw_telemetry(r: &mut dyn Renderer, s: &BerserkerState, _t: f32) {
    r.set_material(cvkg_core::DrawMaterial::TopUI);
    r.set_z_index(1000.0);

    let mode_str = match s.berserker_mode() {
        BerserkerMode::Normal => "NORMAL",
        BerserkerMode::Rage => "RAGE",
        BerserkerMode::Frenzy => "FRENZY",
        BerserkerMode::GodMode => "GOD_MODE",
    };

    let rage_pct = (s.rage * 100.0) as u32;
    let info = format!(
        "CVKG v0.2.8 | {} | RAGE: {}% | CLICKS: {} | FRAME: {}",
        mode_str, rage_pct, s.total_clicks, s.frame_count
    );

    // Semi-transparent background for readability
    r.fill_rect(
        Rect {
            x: 10.0,
            y: 10.0,
            width: 520.0,
            height: 28.0,
        },
        [0.0, 0.0, 0.0, 0.6],
    );

    r.draw_text(&info, 16.0, 30.0, 14.0, [0.0, 1.0, 0.8, 1.0]);

    // Rage bar
    let bar_w = 200.0;
    let bar_h = 8.0;
    let bar_x = 540.0;
    let bar_y = 18.0;

    // Background
    r.fill_rect(
        Rect {
            x: bar_x,
            y: bar_y,
            width: bar_w,
            height: bar_h,
        },
        [0.1, 0.1, 0.1, 0.8],
    );

    // Fill
    let fill_w = bar_w * s.rage;
    let rage_color = if s.rage > 0.7 {
        [1.0, 0.1, 0.2, 1.0]
    } else if s.rage > 0.4 {
        [1.0, 0.5, 0.0, 1.0]
    } else {
        [0.0, 0.8, 0.9, 1.0]
    };
    r.fill_rect(
        Rect {
            x: bar_x,
            y: bar_y,
            width: fill_w,
            height: bar_h,
        },
        rage_color,
    );

    r.set_z_index(0.0);
    r.set_material(cvkg_core::DrawMaterial::Opaque);
}

// ── Rage Vignette ────────────────────────────────────────────────────────

fn draw_rage_vignette(r: &mut dyn Renderer, rect: Rect, rage: f32, t: f32) {
    r.set_material(cvkg_core::DrawMaterial::TopUI);

    let pulse = 0.5 + 0.5 * (t * 8.0 * rage).sin();
    let alpha = rage * 0.15 * pulse;

    // Top edge
    r.fill_rect(
        Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: 40.0,
        },
        [0.8, 0.0, 0.1, alpha],
    );

    // Bottom edge
    r.fill_rect(
        Rect {
            x: rect.x,
            y: rect.height - 40.0,
            width: rect.width,
            height: 40.0,
        },
        [0.8, 0.0, 0.1, alpha],
    );

    // Left edge
    r.fill_rect(
        Rect {
            x: rect.x,
            y: rect.y,
            width: 40.0,
            height: rect.height,
        },
        [0.8, 0.0, 0.1, alpha],
    );

    // Right edge
    r.fill_rect(
        Rect {
            x: rect.width - 40.0,
            y: rect.y,
            width: 40.0,
            height: rect.height,
        },
        [0.8, 0.0, 0.1, alpha],
    );

    r.set_material(cvkg_core::DrawMaterial::Opaque);
}
