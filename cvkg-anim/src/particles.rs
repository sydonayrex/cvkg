use cvkg_core::{Rect, Renderer};
use std::time::Duration;
use rand::RngExt;

/// A single glowing rune particle.
#[derive(Clone, Debug)]
pub struct RunicParticle {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub rune: char,
    pub life: f32, // 1.0 down to 0.0
    pub color: [f32; 4],
    pub rotation: f32,
    pub rotation_speed: f32,
}

/// High-performance emitter for Runic particles (inspired by Vortex/Arwes).
/// Section 3.4: "Animated particles for magical/digital artifacts."
pub struct RunicEmitter {
    pub particles: Vec<RunicParticle>,
    pub spawn_rate: f32, // Particles per second
    pub spawn_timer: f32,
    pub bounds: Rect,
    pub active: bool,
}

impl RunicEmitter {
    pub fn new(bounds: Rect) -> Self {
        Self {
            particles: Vec::with_capacity(100),
            spawn_rate: 10.0,
            spawn_timer: 0.0,
            bounds,
            active: true,
        }
    }

    /// Update particle state.
    pub fn update(&mut self, dt: Duration) {
        let dt_secs = dt.as_secs_f32();
        
        // Update existing particles
        for p in &mut self.particles {
            p.position[0] += p.velocity[0] * dt_secs;
            p.position[1] += p.velocity[1] * dt_secs;
            p.life -= dt_secs * 0.5; // Fade out over 2 seconds
            p.rotation += p.rotation_speed * dt_secs;
        }

        // Remove dead particles
        self.particles.retain(|p| p.life > 0.0);

        // Spawn new particles
        if self.active {
            self.spawn_timer += dt_secs;
            let interval = 1.0 / self.spawn_rate;
            while self.spawn_timer >= interval {
                self.spawn_timer -= interval;
                self.spawn_one();
            }
        }
    }

    fn spawn_one(&mut self) {
        
        let mut rng = rand::rng();
        
        // Elder Futhark runes range: 0x16A0 - 0x16F0
        let runes = ['ᚠ', 'ᚢ', 'ᚦ', 'ᚨ', 'ᚱ', 'ᚲ', 'ᚷ', 'ᚹ', 'ᚺ', 'ᚻ', 'ᚼ', 'ᛁ', 'ᛃ', 'ᛇ', 'ᛈ', 'ᛉ', 'ᛊ', 'ᛏ', 'ᛒ', 'ᛖ', 'ᛗ', 'ᛚ', 'ᛜ', 'ᛟ', 'ᛞ'];
        let rune = runes[rng.random_range(0..runes.len())];

        let x = rng.random_range(self.bounds.x..(self.bounds.x + self.bounds.width));
        let y = self.bounds.y + self.bounds.height; // Spawn at bottom

        self.particles.push(RunicParticle {
            position: [x, y],
            velocity: [rng.random_range(-20.0..20.0), rng.random_range(-60.0..-30.0)],
            rune,
            life: 1.0,
            color: [0.0, 0.8, 1.0, 1.0], // Cyan glow
            rotation: rng.random_range(0.0..std::f32::consts::TAU),
            rotation_speed: rng.random_range(-2.0..2.0),
        });
    }

    /// Render particles using the provided renderer.
    pub fn render(&self, renderer: &mut dyn Renderer) {
        for p in &self.particles {
            let mut color = p.color;
            color[3] *= p.life; // Apply fade
            
            // Draw glowing rune
            renderer.draw_text(
                &p.rune.to_string(),
                p.position[0],
                p.position[1],
                12.0 + (1.0 - p.life) * 8.0, // Grow slightly as it fades
                color,
            );
        }
    }
}
