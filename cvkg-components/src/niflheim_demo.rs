//! # Niflheim Mist Demo
//!
//! A high-fidelity demonstration of the CVKG Phase 6 aesthetics:
//! - **Bifrost**: Frosted glass backdrop blur (Mist of Niflheim)
//! - **Gungnir**: Neon cyan glow
//! - **Ginnungagap**: Deep void background
//! - **Mjolnir**: Geometric shattering and lightning arcs

use crate::primitive::{Shape, Text, Canvas};
use cvkg_core::{StyleResolver, View, Rect, Renderer, AnyView, Never, Color};

/// A simple Z-ordered stack of views.
struct ZStack {
    children: Vec<AnyView>,
}

impl ZStack {
    fn new() -> Self {
        Self { children: Vec::new() }
    }
    fn child<V: View + 'static>(mut self, view: V) -> Self {
        self.children.push(AnyView::new(view));
        self
    }
}

impl View for ZStack {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        log::info!("ZStack rendering {} children", self.children.len());
        renderer.push_vnode(rect, "ZStack");
        for (i, child) in self.children.iter().enumerate() {
            log::info!("  Rendering child {}", i);
            child.render(renderer, rect);
        }
        renderer.pop_vnode();
    }
}

/// Returns a view demonstrating the Niflheim/Bifrost aesthetic.
pub fn niflheim_demo() -> impl View {
    // Resolve tokens from the Yggdrasil environment
    let nifl_cyan = StyleResolver::color("primary");
    let _muspel_magenta = StyleResolver::color("secondary");

    // Z-ordered layer stack
    ZStack::new()
        .child(cyberpunk_background())
        .child(fresnel_boxes())
        .child(lightning_arcs())
        .child(floating_fire_ball())
        .child(niflheim_card(nifl_cyan))
}

fn cyberpunk_background() -> impl View {
    Canvas::new(|renderer, rect| {
        // Draw deep Ginnungagap nebula
        renderer.draw_radial_gradient(
            rect,
            [0.02, 0.01, 0.05, 1.0], // Deep violet void
            [0.0, 0.0, 0.0, 1.0],     // Absolute black
        );
        
        // Draw Tactical Grid
        let grid_size = 60.0;
        let color = [0.0, 0.5, 0.6, 0.1]; // Subtle cyber-cyan
        for x in (0..(rect.width as i32)).step_by(grid_size as usize) {
            renderer.draw_line(rect.x + x as f32, rect.y, rect.x + x as f32, rect.y + rect.height, color, 1.0);
        }
        for y in (0..(rect.height as i32)).step_by(grid_size as usize) {
            renderer.draw_line(rect.x, rect.y + y as f32, rect.x + rect.width, rect.y + y as f32, color, 1.0);
        }
    })
}

fn fresnel_boxes() -> impl View {
    ZStack::new()
        .child(
            Shape::rounded_rect(32.0)
                .fill(Color::new(0.05, 0.05, 0.1, 0.2))
                .bifrost(40.0, 1.5, 0.6)
                .gungnir("#00FFFF", 25.0, 1.2)
                .padding(140.0)
        )
        .child(
            Shape::rounded_rect(16.0)
                .fill(Color::new(0.0, 1.0, 1.0, 0.05))
                .bifrost(20.0, 1.2, 0.4)
                .gungnir("#00FFFF", 10.0, 0.8)
                .padding(200.0)
        )
}

fn lightning_arcs() -> impl View {
    Canvas::new(move |renderer, _rect| {
        let t = renderer.delta_time(); // In real app we'd use a cumulative time
        // Just a placeholder to ensure the lightning_arcs function signature matches
        let _ = t;
    })
}

use std::sync::{Arc, Mutex};

struct Particle {
    pos: [f32; 2],
    vel: [f32; 2],
    color: [f32; 4],
    life: f32,
    size: f32,
}

struct Lcg { state: u32 }
impl Lcg {
    fn new(seed: u32) -> Self { Self { state: seed } }
    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        (self.state & 0x7FFFFFFF) as f32 / 2147483647.0
    }
}

fn floating_fire_ball() -> impl View {
    let particles = Arc::new(Mutex::new(Vec::<Particle>::new()));
    let last_time = Arc::new(Mutex::new(0.0f32));
    let mut rng = Lcg::new(42);

    Canvas::new(move |renderer, rect| {
        let t = renderer.elapsed_time();
        let mut last_t_lock = last_time.lock().unwrap();
        let dt = (t - *last_t_lock).max(0.0).min(0.1);
        *last_t_lock = t;
        
        // Flying position (Berserker Fire motion)
        let cx = rect.x + rect.width * 0.5 + (t * 1.4).cos() * (rect.width * 0.35) + (t * 2.5).sin() * 40.0;
        let cy = rect.y + rect.height * 0.5 + (t * 1.1).sin() * (rect.height * 0.3) + (t * 3.1).cos() * 20.0;
        
        let mut p_list = particles.lock().unwrap();
        
        // 1. Spawn particles
        for _ in 0..3 {
            p_list.push(Particle {
                pos: [cx, cy],
                vel: [rng.next_f32() * 200.0 - 100.0, rng.next_f32() * 200.0 - 100.0],
                color: [1.0, 0.4 + rng.next_f32() * 0.4, 0.1, 1.0],
                life: 1.0,
                size: 4.0 + rng.next_f32() * 12.0,
            });
        }

        // 2. Update and Draw Particles
        p_list.retain_mut(|p| {
            p.pos[0] += p.vel[0] * dt;
            p.pos[1] += p.vel[1] * dt;
            p.life -= dt * 1.5;
            p.size *= 0.98;
            
            let mut p_color = p.color;
            p_color[3] *= p.life;
            renderer.fill_ellipse(
                Rect { x: p.pos[0] - p.size/2.0, y: p.pos[1] - p.size/2.0, width: p.size, height: p.size },
                p_color
            );
            p.life > 0.0
        });

        // 3. Main Fireball Core
        renderer.gungnir(
            Rect { x: cx - 40.0, y: cy - 40.0, width: 80.0, height: 80.0 },
            [1.0, 0.5, 0.1, 1.0],
            25.0,
            1.5,
        );
        renderer.draw_radial_gradient(
            Rect { x: cx - 20.0, y: cy - 20.0, width: 40.0, height: 40.0 },
            [1.0, 0.9, 0.5, 1.0],
            [1.0, 0.2, 0.0, 0.0],
        );
        
        // 4. Lightning Arcs (Mjolnir)
        if (t * 4.0).sin() > 0.95 {
            let target_x = rect.x + rect.width * 0.5;
            let target_y = rect.y + rect.height * 0.5;
            renderer.draw_mjolnir_bolt([cx, cy], [target_x, target_y], [1.0, 0.8, 0.2, 1.0]);
        }

        renderer.request_redraw();
    })
}

fn niflheim_card(color: String) -> impl View {
    Shape::rounded_rect(24.0)
        .fill(Color::new(0.02, 0.02, 0.05, 0.3)) // Translucent glass base
        .bifrost(30.0, 1.3, 0.7) // Stronger Bifrost
        .gungnir(color.clone(), 20.0, 1.5) // Radiant Gungnir
        .mjolnir_slice(15.0, 0.0)
        .padding(60.0)
}

/// A more complex composite demo
pub fn berserker_card() -> impl View {
    let nifl_cyan = StyleResolver::color("primary");

    Text::new("BERSERKER PROTOCOL")
        .gungnir(nifl_cyan, 10.0, 2.0)
        .bifrost(10.0, 1.0, 0.9)
}
