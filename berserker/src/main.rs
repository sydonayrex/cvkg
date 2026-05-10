use cvkg::prelude::*;
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

struct Cube {
    pos: [f32; 2],
    vel: [f32; 2],
    rot: [f32; 3],
    rot_vel: [f32; 3],
    size: f32,
    color: [f32; 4],
}

struct Lcg { state: u32 }
impl Lcg {
    fn new(seed: u32) -> Self { Self { state: seed } }
    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        (self.state & 0x7FFFFFFF) as f32 / 2147483647.0
    }
}

// --- Component State ---

struct BerserkerState {
    counters: [u32; 4],
    particles: Vec<Particle>,
    cubes: Vec<Cube>,
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
            cubes: Vec::new(),
            rng: Lcg::new(1337),
            last_time: 0.0,
            bg_rotation: 0.0,
            bg_pos: [0.0, 0.0],
        }
    }
}

#[derive(Clone)]
struct BerserkerFireView {
    state: Arc<Mutex<BerserkerState>>,
}

impl BerserkerFireView {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(BerserkerState::new())),
        }
    }
}

impl View for BerserkerFireView {
    type Body = Never;
    
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, r: &mut dyn cvkg_core::Renderer, rect: cvkg_core::Rect) {
        log::info!("[Berserker] Render pass start: {}x{}", rect.width, rect.height);
        
        // Essential: Root vnode for VDOM tree construction. 
        // This ensures all subsequent drawing calls are correctly parented for hit-testing.
        r.push_vnode(rect, "BerserkerFireView");
        
        let mut s = self.state.lock().expect("Berserker state mutex poisoned");
        let t = r.elapsed_time();

        let width = rect.width;
        let height = rect.height;

        let rage = r.get_telemetry().berserker_rage;

        // Simulation Step: Only update state if time has advanced.
        // This prevents double-simulation during VDOM capture passes (where t=0).
        if t > s.last_time {
            let dt = (t - s.last_time).min(0.1);
            s.last_time = t;

            // Update Fireball, Particles & Cubes (passing window shake rage)
            update_berserker_simulation(&mut s, width, height, t, dt, rage);
        }
        
        // 1. Background: Floating RK4-physics cubes
        // PERFORMANCE FIX: Skip background in VDOM pass to prevent node-count bloat 
        // and hit-test collisions with interactive elements.
        if t > 0.0 {
            draw_3d_cubes_bg(r, &s, width, height, t);
        }

        // 2. Glassmorphic Cards with Norse Text
        draw_glass_cards(r, width, height, t);

        // 3. The Flaming Fireball of Glory
        // PERFORMANCE FIX: Only render particles during the GPU pass (t > 0).
        // This prevents the VDOM from becoming bloated with thousands of transient nodes,
        // which was causing the stack overflow and hit-testing lag.
        if t > 0.0 {
            draw_berserker_fire(r, &s, width, height, t);
        }

        // 4. Interaction Buttons
        draw_corner_buttons(r, &s, self.state.clone(), width, height);
        
        r.pop_vnode();
        // Request redraw handled by native heartbeat
        // r.request_redraw();
    }
}

fn draw_3d_cubes_bg(r: &mut dyn cvkg_core::Renderer, s: &BerserkerState, w: f32, h: f32, _t: f32) {
    // Fill background with deep void
    r.fill_rect(cvkg_core::Rect { x: 0.0, y: 0.0, width: w, height: h }, [0.01, 0.01, 0.03, 1.0]);
    
    log::info!("[Berserker] Drawing {} cubes in background", s.cubes.len());
    for c in &s.cubes {
        let rect = cvkg_core::Rect {
            x: c.pos[0] - c.size * 0.5,
            y: c.pos[1] - c.size * 0.5,
            width: c.size,
            height: c.size,
        };
        
        // High-Fidelity Raymarched 3D Cube (Mode 21u)
        r.draw_3d_cube(rect, c.color, c.rot);
    }
}

fn draw_glass_cards(r: &mut dyn cvkg_core::Renderer, w: f32, h: f32, t: f32) {
    let card_w = 400.0;
    let card_h = 250.0;
    
    let card_positions = [
        [w * 0.2, h * 0.3],
        [w * 0.7, h * 0.2],
        [w * 0.5, h * 0.7],
    ];
    
    let runes = [
        "\u{16A2}\u{16CF}\u{16B1}\u{16CF}\u{16BF} \u{16A6}\u{16A2}\u{16BF}\u{16D1}\u{16DE}\u{16B1}", 
        "\u{16D2}\u{16D2}\u{16B1}\u{16D2}\u{16CC}\u{16D2}\u{16B1}\u{16D2}\u{16B1} \u{16A0}\u{16CF}\u{16B1}\u{16D2}", 
        "\u{16B4}\u{16B4}\u{16B5} \u{16D1}\u{16CF}\u{16CC}\u{16CF}\u{16D1}\u{16BF}\u{16D1}\u{16B1}", 
    ];

    for (i, pos) in card_positions.iter().enumerate() {
        let x = pos[0] + (t * 0.5 + i as f32).sin() * 20.0;
        let y = pos[1] + (t * 0.3 + i as f32).cos() * 20.0;
        let rect = cvkg_core::Rect { x, y, width: card_w, height: card_h };
        
        r.bifrost(rect, 40.0, 1.2, 0.6);
        r.fill_rounded_rect(rect, 24.0, [0.05, 0.05, 0.1, 0.2]);
        
        r.draw_text(runes[i], x + 40.0, y + 100.0, 32.0, [0.8, 0.9, 1.0, 1.0]);
        r.draw_text("PROTOCOL_ACTIVE", x + 40.0, y + 140.0, 14.0, [0.0, 0.8, 0.8, 0.8]);
    }
}

fn update_berserker_simulation(s: &mut BerserkerState, w: f32, h: f32, t: f32, dt: f32, rage: f32) {
    let cx = w * 0.5 + (t * 1.2).cos() * (w * 0.3);
    let cy = h * 0.5 + (t * 0.8).sin() * (h * 0.25);
    
    // 1. Initialize Cubes if needed
    if s.cubes.is_empty() {
        for _ in 0..15 {
            s.cubes.push(Cube {
                pos: [s.rng.next_f32() * w, s.rng.next_f32() * h],
                vel: [(s.rng.next_f32() - 0.5) * 100.0, (s.rng.next_f32() - 0.5) * 100.0],
                rot: [s.rng.next_f32() * 6.28, s.rng.next_f32() * 6.28, s.rng.next_f32() * 6.28],
                rot_vel: [(s.rng.next_f32() - 0.5) * 2.0, (s.rng.next_f32() - 0.5) * 2.0, (s.rng.next_f32() - 0.5) * 2.0],
                size: 50.0 + s.rng.next_f32() * 100.0,
                color: [0.1, 0.5 + s.rng.next_f32() * 0.3, 0.8 + s.rng.next_f32() * 0.2, 0.6],
            });
        }
    }

    // 2. RK4 Physics for Cubes
    let gravity = [150.0, 150.0]; // Force toward bottom-right
    let shake_force = rage * 3000.0;
    let drag = 0.15;

    for c in &mut s.cubes {
        // Apply shake force in random direction based on rage
        let sx = (s.rng.next_f32() - 0.5) * shake_force;
        let sy = (s.rng.next_f32() - 0.5) * shake_force;
        
        // RK4 Integration Step
        // State is [x, y, vx, vy, rot[3], rv[3]]
        let f = |v: [f32; 2], rv: [f32; 3]| -> ([f32; 2], [f32; 3]) {
            ([gravity[0] + sx - drag * v[0], gravity[1] + sy - drag * v[1]], 
             [-drag * rv[0], -drag * rv[1], -drag * rv[2]])
        };

        // k1
        let k1_v = c.vel;
        let (k1_a, k1_ra) = f(c.vel, c.rot_vel);

        // k2
        let k2_v = [c.vel[0] + k1_a[0] * dt * 0.5, c.vel[1] + k1_a[1] * dt * 0.5];
        let (k2_a, k2_ra) = f(k2_v, [c.rot_vel[0] + k1_ra[0] * dt * 0.5, c.rot_vel[1] + k1_ra[1] * dt * 0.5, c.rot_vel[2] + k1_ra[2] * dt * 0.5]);

        // k3
        let k3_v = [c.vel[0] + k2_a[0] * dt * 0.5, c.vel[1] + k2_a[1] * dt * 0.5];
        let (k3_a, k3_ra) = f(k3_v, [c.rot_vel[0] + k2_ra[0] * dt * 0.5, c.rot_vel[1] + k2_ra[1] * dt * 0.5, c.rot_vel[2] + k2_ra[2] * dt * 0.5]);

        // k4
        let k4_v = [c.vel[0] + k3_a[0] * dt, c.vel[1] + k3_a[1] * dt];
        let (k4_a, k4_ra) = f(k4_v, [c.rot_vel[0] + k3_ra[0] * dt, c.rot_vel[1] + k3_ra[1] * dt, c.rot_vel[2] + k3_ra[2] * dt]);

        // Update Position
        c.pos[0] += (dt / 6.0) * (k1_v[0] + 2.0 * k2_v[0] + 2.0 * k3_v[0] + k4_v[0]);
        c.pos[1] += (dt / 6.0) * (k1_v[1] + 2.0 * k2_v[1] + 2.0 * k3_v[1] + k4_v[1]);
        c.vel[0] += (dt / 6.0) * (k1_a[0] + 2.0 * k2_a[0] + 2.0 * k3_a[0] + k4_a[0]);
        c.vel[1] += (dt / 6.0) * (k1_a[1] + 2.0 * k2_a[1] + 2.0 * k3_a[1] + k4_a[1]);
        
        // Update 3-axis Rotation
        for i in 0..3 {
            let k1_rv = c.rot_vel[i];
            let k2_rv = c.rot_vel[i] + k1_ra[i] * dt * 0.5;
            let k3_rv = c.rot_vel[i] + k2_ra[i] * dt * 0.5;
            let k4_rv = c.rot_vel[i] + k3_ra[i] * dt;
            c.rot[i] += (dt / 6.0) * (k1_rv + 2.0 * k2_rv + 2.0 * k3_rv + k4_rv);
            c.rot_vel[i] += (dt / 6.0) * (k1_ra[i] + 2.0 * k2_ra[i] + 2.0 * k3_ra[i] + k4_ra[i]);
            
            // Add some "shake" torque
            c.rot_vel[i] += (s.rng.next_f32() - 0.5) * rage * 5.0 * dt;
        }

        // 3. Wall Bouncing
        let margin = c.size * 0.5;
        if c.pos[0] < margin { c.pos[0] = margin; c.vel[0] *= -0.8; }
        if c.pos[0] > w - margin { c.pos[0] = w - margin; c.vel[0] *= -0.8; }
        if c.pos[1] < margin { c.pos[1] = margin; c.vel[1] *= -0.8; }
        if c.pos[1] > h - margin { c.pos[1] = h - margin; c.vel[1] *= -0.8; }
    }

    // 4. Spawn new particles
    for _ in 0..5 {
        let angle = s.rng.next_f32() * 6.28;
        let speed = 100.0 + s.rng.next_f32() * 200.0;
        s.particles.push(Particle {
            pos: [cx, cy],
            vel: [angle.cos() * speed, angle.sin() * speed - 50.0], 
            color: [1.0, 0.3 + s.rng.next_f32() * 0.5, 0.0, 1.0],
            life: 1.0 + s.rng.next_f32() * 1.5,
            size: 4.0 + s.rng.next_f32() * 8.0,
            // Vary shapes: 0=Round, 1=Runic Fragment
            is_ember: s.rng.next_f32() > 0.85, 
        });
    }

    // 5. Update existing particles
    s.particles.retain_mut(|p| {
        p.life -= dt;
        p.pos[0] += p.vel[0] * dt;
        p.pos[1] += p.vel[1] * dt;
        p.life > 0.0
    });
}

fn draw_berserker_fire(r: &mut dyn cvkg_core::Renderer, s: &BerserkerState, w: f32, h: f32, t: f32) {
    let cx = w * 0.5 + (t * 1.2).cos() * (w * 0.3);
    let cy = h * 0.5 + (t * 0.8).sin() * (h * 0.25);
    
    // Draw Fireball Core with Layered Radial Gradients
    // Mode 16 now correctly handles outer alpha, so this will be circular.
    r.draw_radial_gradient(
        cvkg_core::Rect { x: cx - 100.0, y: cy - 100.0, width: 200.0, height: 200.0 },
        [1.0, 0.4, 0.0, 0.6], 
        [0.2, 0.0, 0.0, 0.0]
    );
    r.draw_radial_gradient(
        cvkg_core::Rect { x: cx - 60.0, y: cy - 60.0, width: 120.0, height: 120.0 },
        [1.0, 0.8, 0.2, 0.8], 
        [1.0, 0.2, 0.0, 0.0]
    );
    r.draw_radial_gradient(
        cvkg_core::Rect { x: cx - 30.0, y: cy - 30.0, width: 60.0, height: 60.0 },
        [1.0, 1.0, 0.8, 1.0], 
        [1.0, 0.5, 0.0, 0.0]
    );

    // Draw Particles
    for p in &s.particles {
        let p_color = [p.color[0], p.color[1], p.color[2], p.life.min(1.0)];
        let rect = cvkg_core::Rect { x: p.pos[0], y: p.pos[1], width: p.size, height: p.size };
        
        if p.is_ember {
            // Runic Fragments (Procedural Shapes)
            r.draw_text("\u{16A2}", p.pos[0], p.pos[1], p.size * 2.0, p_color);
        } else {
            // Round Fire/Smoke
            r.fill_ellipse(rect, p_color);
        }
    }

    // Occasional lightning bolts from the core
    let mut rng = Lcg::new((t * 1000.0) as u32);
    if rng.next_f32() > 0.95 {
        let angle = rng.next_f32() * 6.28;
        let dist = 100.0 + rng.next_f32() * 300.0;
        let tx = cx + angle.cos() * dist;
        let ty = cy + angle.sin() * dist;
        r.draw_mjolnir_bolt([cx, cy], [tx, ty], [0.6, 0.9, 1.0, 1.0]);
    }
}

fn draw_corner_buttons(r: &mut dyn cvkg_core::Renderer, s: &BerserkerState, state_handle: Arc<Mutex<BerserkerState>>, w: f32, h: f32) {
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
        let rect = cvkg_core::Rect { x, y, width: btn_size, height: btn_size };
        
        // CRITICAL: Wrap each button in its own VNode. 
        // Without this, handlers are registered on the parent 'BerserkerRoot', 
        // causing all clicks to default to the last registered button.
        r.push_vnode(rect, "CornerButton");
        
        r.fill_rounded_rect(rect, 12.0, [0.2, 0.2, 0.3, 0.8]);
        r.draw_text(corner.2, x + 35.0, y + 60.0, 32.0, [1.0, 1.0, 1.0, 1.0]);
        
        let count_str = format!("{}", s.counters[i]);
        r.draw_text(&count_str, x + btn_size + 10.0, y + 60.0, 24.0, [0.0, 1.0, 0.5, 1.0]);

        let counter_ref = state_handle.clone();
        let h = Arc::new(move |_| {
            let mut s = counter_ref.lock().unwrap();
            s.counters[i] += 1;
            log::info!("Button {} clicked! Total: {}", i, s.counters[i]);
        });
        r.register_handler("pointerdown", h.clone());
        r.register_handler("pointerclick", h);

        r.pop_vnode();
    }
}

fn main() {
    // Initialize env_logger to see what's happening
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    // Set a panic hook to capture aborts/panics
    std::panic::set_hook(Box::new(|info| {
        log::error!("CRITICAL_FAILURE: Application panicked: {}", info);
    }));

    log::info!("Launching Berserker Fire Native...");
    cvkg::native::NativeRenderer::run(BerserkerFireView::new());
}
