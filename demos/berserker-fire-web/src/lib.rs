use wasm_bindgen::prelude::*;
use cvkg_render_web::WebRenderer;
use cvkg_core::{ElapsedTime, Rect, Renderer, FrameRenderer};
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

// --- Particle System ---

struct Particle {
    pos: [f32; 2],
    vel: [f32; 2],
    color: [f32; 4],
    life: f32,
    size: f32,
    is_ember: bool,
}

struct Lcg { state: u32 }
impl Lcg {
    fn new(seed: u32) -> Self { Self { state: seed } }
    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        (self.state & 0x7FFFFFFF) as f32 / 2147483647.0
    }
}

// --- Demo State ---

struct BerserkerState {
    counters: [u32; 4],
    particles: Vec<Particle>,
    rng: Lcg,
    last_time: f32,
    bg_rotation: f32,
    bg_pos: [f32; 2],
}

impl BerserkerState {
    fn new() -> Self {
        Self {
            counters: [0; 4],
            particles: Vec::new(),
            rng: Lcg::new(1337),
            last_time: 0.0,
            bg_rotation: 0.0,
            bg_pos: [0.0, 0.0],
        }
    }
}

// --- Main Entry Point ---

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("error initializing log");

    log::info!("Berserker Fire Demo Initializing...");

    let mut renderer = WebRenderer::new();
    renderer.forge().await?;

    let state = Arc::new(Mutex::new(BerserkerState::new()));

    let renderer_rc = Rc::new(RefCell::new(renderer));
    
    // Setup Click Handling
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
        // Check Corners - buttons drawn at (padding, padding), (canvas_w-btn_size-padding, padding), etc.
        if x >= padding && x < padding + btn_size && y >= padding && y < padding + btn_size { s.counters[0] += 1; }
        if x >= canvas_w - btn_size - padding && x < canvas_w - padding && y >= padding && y < padding + btn_size { s.counters[1] += 1; }
        if x >= padding && x < padding + btn_size && y >= canvas_h - btn_size - padding && y < canvas_h - padding { s.counters[2] += 1; }
        if x >= canvas_w - btn_size - padding && x < canvas_w - padding && y >= canvas_h - btn_size - padding && y < canvas_h - padding { s.counters[3] += 1; }
        if x < btn_size && y > canvas_h - btn_size { s.counters[2] += 1; } 
        if x > canvas_w - btn_size && y > canvas_h - btn_size { s.counters[3] += 1; } 
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);
    
    canvas.add_event_listener_with_callback("mousedown", on_click.as_ref().unchecked_ref())?;
    on_click.forget(); 

    // Render Loop
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
        
        r.begin_frame();
        
        let width = 1888.0; 
        let height = 951.0;
        let full_rect = Rect { x: 0.0, y: 0.0, width, height };

        // 6. Background: Rotating floating "cvkg"
        s.bg_rotation += dt * 0.5;
        s.bg_pos[0] = (s.bg_pos[0] - dt * 50.0 + width) % width;
        s.bg_pos[1] = (s.bg_pos[1] + dt * 30.0) % height;
        
        draw_3d_text_bg(&mut *r, full_rect, s.bg_rotation, s.bg_pos);

        // 4. Glassmorphic Cards with Norse Text
        draw_glass_cards(&mut *r, width, height, t);

        // 1, 2, 3. The Flaming Fireball of Glory
        draw_berserker_fire(&mut *r, &mut *s, width, height, t, dt);

        // 5. Interaction Buttons
        draw_corner_buttons(&mut *r, &s.counters, width, height);

        r.end_frame(());
        
        web_sys::window().unwrap()
            .request_animation_frame(loop_f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .expect("should register `requestAnimationFrame` OK");
    }) as Box<dyn FnMut()>));

    web_sys::window().unwrap().request_animation_frame(loop_g.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;
    
    Ok(())
}

fn draw_3d_text_bg(r: &mut dyn Renderer, rect: Rect, rotation: f32, pos: [f32; 2]) {
    r.draw_radial_gradient(rect, [0.01, 0.01, 0.03, 1.0], [0.0, 0.0, 0.0, 1.0]);
    
    for i in 0..5 {
        let offset = i as f32 * 200.0;
        let x = (pos[0] + offset) % rect.width;
        let y = (pos[1] + offset * 0.5) % rect.height;
        
        let scale = 1.5 + (rotation + i as f32).sin() * 0.3;
        let text = "CVKG";
        r.draw_text(text, x, y, 64.0 * scale, [0.1, 0.1, 0.2, 0.4]);
        
        // Add a slight "3D" offset
        r.draw_text(text, x + 4.0, y + 4.0, 64.0 * scale, [0.05, 0.05, 0.1, 0.2]);
    }
}

fn draw_glass_cards(r: &mut dyn Renderer, w: f32, h: f32, t: f32) {
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

    for (i, pos) in card_positions.iter().enumerate() {
        let x = pos[0] + (t * 0.5 + i as f32).sin() * 20.0;
        let y = pos[1] + (t * 0.3 + i as f32).cos() * 20.0;
        let rect = Rect { x, y, width: card_w, height: card_h };
        
        r.bifrost(rect, 40.0, 1.2, 0.6);
        r.fill_rounded_rect(rect, 24.0, [0.05, 0.05, 0.1, 0.2]);
        
        r.draw_text(runes[i], x + 40.0, y + 100.0, 32.0, [0.8, 0.9, 1.0, 1.0]);
        r.draw_text("PROTOCOL_ACTIVE", x + 40.0, y + 140.0, 14.0, [0.0, 0.8, 0.8, 0.8]);
    }
}

fn draw_berserker_fire(r: &mut dyn Renderer, s: &mut BerserkerState, w: f32, h: f32, t: f32, dt: f32) {
    let cx = w * 0.5 + (t * 1.2).cos() * (w * 0.3);
    let cy = h * 0.5 + (t * 0.8).sin() * (h * 0.25);
    
    for _ in 0..5 {
        let angle = s.rng.next_f32() * 6.28;
        let speed = 100.0 + s.rng.next_f32() * 200.0;
        s.particles.push(Particle {
            pos: [cx, cy],
            vel: [angle.cos() * speed, angle.sin() * speed - 50.0], 
            color: [1.0, 0.3 + s.rng.next_f32() * 0.5, 0.0, 1.0],
            life: 1.0 + s.rng.next_f32() * 1.5,
            size: 2.0 + s.rng.next_f32() * 6.0,
            is_ember: s.rng.next_f32() > 0.3,
        });
    }

    s.particles.retain_mut(|p| {
        p.pos[0] += p.vel[0] * dt;
        p.pos[1] += p.vel[1] * dt;
        p.life -= dt;
        
        let alpha = (p.life).min(1.0).max(0.0);
        let p_color = [p.color[0], p.color[1], p.color[2], p.color[3] * alpha];
        
        if p.is_ember {
            r.fill_rect(Rect { x: p.pos[0], y: p.pos[1], width: p.size, height: p.size }, p_color);
        } else {
            r.fill_ellipse(Rect { x: p.pos[0], y: p.pos[1], width: p.size, height: p.size }, p_color);
        }
        p.life > 0.0
    });

    let fire_rect = Rect { x: cx - 60.0, y: cy - 60.0, width: 120.0, height: 120.0 };
    
    r.draw_radial_gradient(fire_rect, [1.0, 0.8, 0.2, 1.0], [1.0, 0.2, 0.0, 0.0]);
    r.draw_radial_gradient(Rect { x: cx - 30.0, y: cy - 30.0, width: 60.0, height: 60.0 }, [1.0, 1.0, 0.8, 1.0], [1.0, 0.5, 0.0, 0.0]);

    if s.rng.next_f32() > 0.92 {
        let angle = s.rng.next_f32() * 6.28;
        let dist = 100.0 + s.rng.next_f32() * 300.0;
        let tx = cx + angle.cos() * dist;
        let ty = cy + angle.sin() * dist;
        r.draw_mjolnir_bolt([cx, cy], [tx, ty], [0.6, 0.9, 1.0, 1.0]);
    }
}

fn draw_corner_buttons(r: &mut dyn Renderer, counters: &[u32; 4], w: f32, h: f32) {
    let btn_size = 100.0;
    let padding = 20.0;
    let corners = [
        (padding, padding, "I"),
        (w - btn_size - padding, padding, "II"),
        (padding, h - btn_size - padding, "III"),
        (w - btn_size - padding, h - btn_size - padding, "IV"),
    ];

    for (i, corner) in corners.iter().enumerate() {
        let x = corner.0;
        let y = corner.1;
        let rect = Rect { x, y, width: btn_size, height: btn_size };
        
        r.fill_rounded_rect(rect, 12.0, [0.2, 0.2, 0.3, 0.8]);
        
        r.draw_text(corner.2, x + 35.0, y + 60.0, 32.0, [1.0, 1.0, 1.0, 1.0]);
        
        let count_str = format!("{}", counters[i]);
        r.draw_text(&count_str, x + btn_size + 10.0, y + 60.0, 24.0, [0.0, 1.0, 0.5, 1.0]);
    }
}
