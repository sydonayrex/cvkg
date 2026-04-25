use cvkg::prelude::*;
use cvkg_core::Renderer;
use rand::Rng;
use std::cell::RefCell;
use std::time::Instant;

struct Particle {
    pos: [f32; 2],
    vel: [f32; 2],
    color: [f32; 4],
    life: f32,
    size: f32,
}

/// BerserkerFireDemo — A high-fidelity showcase of the Berserker rendering pipeline.
/// Features macOS-style vibrant glassmorphism and dynamic particle physics.
struct BerserkerFireDemo {
    start_time: Instant,
    last_frame: RefCell<Instant>,
    particles: RefCell<Vec<Particle>>,
}

impl BerserkerFireDemo {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
            last_frame: RefCell::new(Instant::now()),
            particles: RefCell::new(Vec::new()),
        }
    }

    fn update_particles(&self, _rect: Rect, ember_pos: [f32; 2]) {
        let mut last_frame = self.last_frame.borrow_mut();
        let now = Instant::now();
        let dt = now.duration_since(*last_frame).as_secs_f32();
        *last_frame = now;

        let mut particles = self.particles.borrow_mut();
        let mut rng = rand::thread_rng();

        // Spawn new particles at the ember position
        for _ in 0..8 {
            particles.push(Particle {
                pos: ember_pos,
                vel: [rng.gen_range(-200.0..200.0), rng.gen_range(-200.0..200.0)],
                color: [1.0, rng.gen_range(0.3..0.8), 0.2, 1.0],
                life: 1.0,
                size: rng.gen_range(8.0..24.0),
            });
        }

        // Update and prune particles
        particles.retain_mut(|p| {
            p.pos[0] += p.vel[0] * dt;
            p.pos[1] += p.vel[1] * dt;
            p.life -= dt * 1.8;
            p.size *= 0.96;
            p.life > 0.0
        });
    }
}

impl View for BerserkerFireDemo {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = self.start_time.elapsed().as_secs_f32();

        // 1. Render Ginnungagap Nebula (Vibrant Background for glass blur)
        // Multiple overlapping gradients for a complex look
        renderer.draw_linear_gradient(rect, [0.1, 0.0, 0.3, 1.0], [0.0, 0.1, 0.4, 1.0], t * 0.2);

        // Add a secondary moving "nebula" glow
        let neb_x = rect.width * 0.5 + (t * 0.5).cos() * 300.0;
        let neb_y = rect.height * 0.5 + (t * 0.7).sin() * 200.0;
        renderer.draw_radial_gradient(
            Rect {
                x: neb_x - 400.0,
                y: neb_y - 400.0,
                width: 800.0,
                height: 800.0,
            },
            [0.2, 0.0, 0.5, 0.4],
            [0.0, 0.0, 0.0, 0.0],
        );

        // 2. Render Tactical Grid (Behind glass)
        let grid_size = 80.0;
        for x in (0..(rect.width as i32)).step_by(grid_size as usize) {
            renderer.draw_line(
                x as f32,
                0.0,
                x as f32,
                rect.height,
                [0.0, 1.0, 1.0, 0.1],
                1.0,
            );
        }
        for y in (0..(rect.height as i32)).step_by(grid_size as usize) {
            renderer.draw_line(
                0.0,
                y as f32,
                rect.width,
                y as f32,
                [0.0, 1.0, 1.0, 0.1],
                1.0,
            );
        }

        // 3. Render Individual Particles BEFORE cards (so they get blurred)
        let ember_x = rect.width / 2.0 + (t * 1.4).cos() * 500.0 + (t * 2.5).sin() * 40.0;
        let ember_y = rect.height / 2.0 + (t * 1.1).sin() * 320.0 + (t * 3.1).cos() * 20.0;

        self.update_particles(rect, [ember_x, ember_y]);

        let particles = self.particles.borrow();
        for p in particles.iter() {
            let mut p_color = p.color;
            p_color[3] *= p.life;
            renderer.fill_ellipse(
                Rect {
                    x: p.pos[0] - p.size / 2.0,
                    y: p.pos[1] - p.size / 2.0,
                    width: p.size,
                    height: p.size,
                },
                p_color,
            );
        }

        // 4. Render High-Fidelity macOS-style Glass Cards
        for i in 0..3 {
            let offset_x = (i as f32 * 400.0) + 100.0;
            let card_rect = Rect {
                x: offset_x,
                y: 250.0,
                width: 340.0,
                height: 400.0,
            };

            // Drop Shadow (Grounding - Tier 3)
            renderer.draw_drop_shadow(card_rect, 24.0, [0.0, 0.0, 0.0, 0.25], 20.0, 4.0);

            // Bifrost Glassmorphism (Vibrant Mode 7)
            renderer.bifrost(card_rect, 24.0, 1.0, 0.3); // High blur, high saturation boost

            // Premium White Rim (macOS Dock style)
            renderer.stroke_rounded_rect(card_rect, 24.0, [1.0, 1.0, 1.0, 0.4], 1.0);

            // Typography
            renderer.draw_text(
                &format!("NODE PROTOCOL {:02}", i + 7),
                card_rect.x + 40.0,
                card_rect.y + 60.0,
                26.0,
                [1.0, 1.0, 1.0, 1.0],
            );
            renderer.draw_text(
                "SYSTEM: STABLE",
                card_rect.x + 40.0,
                card_rect.y + 100.0,
                16.0,
                [0.0, 1.0, 1.0, 0.9],
            );

            // Progress Bar
            renderer.fill_rounded_rect(
                Rect {
                    x: card_rect.x + 40.0,
                    y: card_rect.y + 340.0,
                    width: 260.0,
                    height: 6.0,
                },
                3.0,
                [1.0, 1.0, 1.0, 0.1],
            );
            renderer.fill_rounded_rect(
                Rect {
                    x: card_rect.x + 40.0,
                    y: card_rect.y + 340.0,
                    width: 260.0 * (0.5 + (t * 0.5).sin() * 0.5),
                    height: 6.0,
                },
                3.0,
                [0.0, 1.0, 1.0, 0.8],
            );
        }

        // 5. Core Ember Spark (Drawn on top)
        renderer.fill_ellipse(
            Rect {
                x: ember_x - 16.0,
                y: ember_y - 16.0,
                width: 32.0,
                height: 32.0,
            },
            [1.0, 1.0, 0.9, 1.0],
        );

        // 6. Reactive Mjolnir Lightning
        if (t * 2.5).sin() > 0.96 {
            renderer.draw_mjolnir_bolt(
                [ember_x, ember_y],
                [rect.width / 2.0, rect.height / 2.0],
                [1.0, 0.6, 0.0, 1.0],
            );
        }
    }
}

fn main() {
    println!("Forging Berserker Fire Demo (High-Fidelity Glass Pass)...");
    cvkg::native::NativeRenderer::run(BerserkerFireDemo::new());
}
