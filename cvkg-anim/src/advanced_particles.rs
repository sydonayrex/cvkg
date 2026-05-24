//! Advanced particle system with SDF collisions, reaction-diffusion flow fields,
//! pointer-reactive attractors, SoA+SIMD layout, and spline-eased color animation.
//!
//! This module supersedes the basic `RunicEmitter` with a production-grade
//! particle architecture suitable for real-time GPU-accelerated effects.

use cvkg_core::Rect;
use glam::Vec2;
use rand::RngExt;
use std::time::Duration;

// =============================================================================
// SPLINE EASING
// =============================================================================

/// A cubic bezier spline easing function defined by four control points.
///
/// Used to interpolate particle color, scale, and opacity over lifetime
/// with smooth, artist-controllable curves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplineEasing {
    /// P1 x-coordinate (0..1 range expected).
    pub p1x: f32,
    /// P1 y-coordinate.
    pub p1y: f32,
    /// P2 x-coordinate (0..1 range expected).
    pub p2x: f32,
    /// P2 y-coordinate.
    pub p2y: f32,
}

impl SplineEasing {
    /// Linear easing (no curve).
    pub fn linear() -> Self {
        Self {
            p1x: 0.25,
            p1y: 0.25,
            p2x: 0.75,
            p2y: 0.75,
        }
    }

    /// Ease-in-out (smooth start and end).
    pub fn ease_in_out() -> Self {
        Self {
            p1x: 0.42,
            p1y: 0.0,
            p2x: 0.58,
            p2y: 1.0,
        }
    }

    /// Elastic snap (overshoots then settles).
    pub fn elastic() -> Self {
        Self {
            p1x: 0.68,
            p1y: -0.55,
            p2x: 0.265,
            p2y: 1.55,
        }
    }

    /// Evaluate the cubic bezier at parameter `t` in [0, 1].
    ///
    /// Uses Newton-Raphson iteration to solve for the x-coordinate matching `t`,
    /// then returns the corresponding y-coordinate.
    pub fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(self.p1x, self.p1y);
        let p2 = Vec2::new(self.p2x, self.p2y);
        let p3 = Vec2::new(1.0, 1.0);

        // Newton-Raphson: find t_val such that bezier_x(t_val) = t
        let mut t_val = t;
        for _ in 0..8 {
            let x = cubic_bezier_x(p0.x, p1.x, p2.x, p3.x, t_val);
            let dx = cubic_bezier_dx(p0.x, p1.x, p2.x, p3.x, t_val);
            if dx.abs() < 1e-6 {
                break;
            }
            t_val -= (x - t) / dx;
            t_val = t_val.clamp(0.0, 1.0);
        }

        cubic_bezier_x(p0.y, p1.y, p2.y, p3.y, t_val)
    }

    /// Creates a color ramp: maps a lifetime [0, 1] to RGBA using four
    /// SplineEasing curves (one per channel).
    pub fn color_ramp(
        lifetime: f32,
        r_curve: &SplineEasing,
        g_curve: &SplineEasing,
        b_curve: &SplineEasing,
        a_curve: &SplineEasing,
    ) -> [f32; 4] {
        let t = lifetime.clamp(0.0, 1.0);
        [
            r_curve.evaluate(t).clamp(0.0, 1.0),
            g_curve.evaluate(t).clamp(0.0, 1.0),
            b_curve.evaluate(t).clamp(0.0, 1.0),
            a_curve.evaluate(t).clamp(0.0, 1.0),
        ]
    }

    /// Evaluates a scale curve: maps lifetime to a scale factor.
    pub fn scale_curve(&self, lifetime: f32, min: f32, max: f32) -> f32 {
        let t = lifetime.clamp(0.0, 1.0);
        let v = self.evaluate(t).clamp(0.0, 1.0);
        min + (max - min) * v
    }
}

/// Evaluates the cubic bezier x-component at parameter t.
fn cubic_bezier_x(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
}

/// Derivative of the cubic bezier x-component.
fn cubic_bezier_dx(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    3.0 * u * u * (p1 - p0) + 6.0 * u * t * (p2 - p1) + 3.0 * t * t * (p3 - p2)
}

// =============================================================================
// REACTION-DIFFUSION FLOW FIELD
// =============================================================================

/// Gray-Scott reaction-diffusion simulation for organic particle turbulence.
///
/// The simulation runs on a 2D grid where two chemicals (A and B) diffuse
/// and react. The gradient of chemical B is used as a flow field that
/// steers particles along organic, evolving paths.
#[derive(Debug, Clone)]
pub struct ReactionDiffusionField {
    /// Grid width in cells.
    pub width: u32,
    /// Grid height in cells.
    pub height: u32,
    /// Chemical A concentration grid (current frame).
    a_current: Vec<f32>,
    /// Chemical A concentration grid (next frame).
    a_next: Vec<f32>,
    /// Chemical B concentration grid (current frame).
    b_current: Vec<f32>,
    /// Chemical B concentration grid (next frame).
    b_next: Vec<f32>,
    /// Diffusion rate of chemical A.
    pub feed: f32,
    /// Removal rate of chemical B.
    pub kill: f32,
    /// Diffusion coefficient for A.
    pub d_a: f32,
    /// Diffusion coefficient for B.
    pub d_b: f32,
    /// Simulation time step.
    pub dt: f32,
}

impl ReactionDiffusionField {
    /// Creates a new reaction-diffusion field with the standard "Mitosis" preset.
    ///
    /// `width` and `height` should be powers of two for cache efficiency.
    /// The field is initialized with chemical A everywhere and small random
    /// seeds of chemical B.
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        let a_current = vec![1.0f32; size];
        let mut b_current = vec![0.0f32; size];

        // Seed chemical B in random small patches
        let mut rng = rand::rng();
        for _ in 0..(size / 20) {
            let x = rng.random_range(0..width as usize);
            let y = rng.random_range(0..height as usize);
            let idx = y * width as usize + x;
            b_current[idx] = 1.0;
            // Also seed neighbors for a small cluster
            for dx in -2i32..=2 {
                for dy in -2i32..=2 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let ni = ny as usize * width as usize + nx as usize;
                        b_current[ni] = 1.0;
                    }
                }
            }
        }

        Self {
            width,
            height,
            a_current,
            a_next: vec![0.0; size],
            b_current,
            b_next: vec![0.0; size],
            feed: 0.055,
            kill: 0.062,
            d_a: 1.0,
            d_b: 0.5,
            dt: 1.0,
        }
    }

    /// Creates the field with custom feed/kill parameters.
    pub fn with_params(width: u32, height: u32, feed: f32, kill: f32) -> Self {
        let mut field = Self::new(width, height);
        field.feed = feed;
        field.kill = kill;
        field
    }

    /// Steps the simulation forward by one iteration.
    pub fn step(&mut self) {
        let w = self.width as usize;
        let h = self.height as usize;

        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;

                // Laplacian with wrapping boundaries
                let laplace_a = self.laplacian(&self.a_current, x, y, w, h);
                let laplace_b = self.laplacian(&self.b_current, x, y, w, h);

                let a = self.a_current[idx];
                let b = self.b_current[idx];
                let abb = a * b * b;

                self.a_next[idx] = (a
                    + (self.d_a * laplace_a - abb + self.feed * (1.0 - a)) * self.dt)
                    .clamp(0.0, 1.0);
                self.b_next[idx] = (b
                    + (self.d_b * laplace_b + abb - (self.kill + self.feed) * b) * self.dt)
                    .clamp(0.0, 1.0);
            }
        }

        std::mem::swap(&mut self.a_current, &mut self.a_next);
        std::mem::swap(&mut self.b_current, &mut self.b_next);
    }

    /// Steps the simulation forward by `iterations` iterations.
    pub fn step_n(&mut self, iterations: u32) {
        for _ in 0..iterations {
            self.step();
        }
    }

    /// Returns the flow vector at a given UV coordinate in [0, 1].
    ///
    /// The flow is computed as the gradient of the B field, producing
    /// organic swirling motion.
    pub fn flow_at_uv(&self, u: f32, v: f32) -> Vec2 {
        let x = (u * self.width as f32) as usize % self.width as usize;
        let y = (v * self.height as f32) as usize % self.height as usize;
        let idx = y * self.width as usize + x;
        let w = self.width as usize;
        let h = self.height as usize;

        // Central difference gradient
        let left = if x > 0 {
            self.b_current[idx - 1]
        } else {
            self.b_current[idx + w - 1]
        };
        let right = if x < w - 1 {
            self.b_current[idx + 1]
        } else {
            self.b_current[idx + 1 - w]
        };
        let up = if y > 0 {
            self.b_current[idx - w]
        } else {
            self.b_current[idx + (h - 1) * w]
        };
        let down = if y < h - 1 {
            self.b_current[idx + w]
        } else {
            self.b_current[idx - (h - 1) * w]
        };

        Vec2::new(right - left, down - up)
    }

    /// Returns the B chemical concentration at UV coordinates.
    pub fn b_at_uv(&self, u: f32, v: f32) -> f32 {
        let x = (u * self.width as f32) as usize % self.width as usize;
        let y = (v * self.height as f32) as usize % self.height as usize;
        self.b_current[y * self.width as usize + x]
    }

    fn laplacian(&self, grid: &[f32], x: usize, y: usize, w: usize, h: usize) -> f32 {
        let idx = y * w + x;
        let left = if x > 0 {
            grid[idx - 1]
        } else {
            grid[idx + w - 1]
        };
        let right = if x < w - 1 {
            grid[idx + 1]
        } else {
            grid[idx + 1 - w]
        };
        let up = if y > 0 {
            grid[idx - w]
        } else {
            grid[idx + (h - 1) * w]
        };
        let down = if y < h - 1 {
            grid[idx + w]
        } else {
            grid[idx - (h - 1) * w]
        };
        let center = grid[idx];
        left + right + up + down - 4.0 * center
    }
}

// =============================================================================
// POINTER-REACTIVE ATTRACTORS
// =============================================================================

/// A point attractor or repeller that influences nearby particles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerAttractor {
    /// Position in world space.
    pub position: Vec2,
    /// Strength: positive = attract, negative = repel.
    pub strength: f32,
    /// Radius of influence in pixels.
    pub radius: f32,
    /// Falloff exponent. 1.0 = linear, 2.0 = inverse-square, etc.
    pub falloff: f32,
}

impl PointerAttractor {
    /// Creates a new attractor at the given position.
    pub fn new(x: f32, y: f32, strength: f32, radius: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            strength,
            radius,
            falloff: 2.0,
        }
    }

    /// Sets the falloff exponent.
    pub fn with_falloff(mut self, falloff: f32) -> Self {
        self.falloff = falloff;
        self
    }

    /// Computes the force vector applied to a particle at `particle_pos`.
    pub fn force_at(&self, particle_pos: Vec2) -> Vec2 {
        let diff = self.position - particle_pos;
        let dist = diff.length();
        if dist >= self.radius || dist < 0.1 {
            return Vec2::ZERO;
        }

        let t = 1.0 - dist / self.radius;
        let magnitude = self.strength * t.powf(self.falloff);
        diff.normalize_or_zero() * magnitude
    }
}

/// A collection of active attractors/repellers.
#[derive(Debug, Clone)]
pub struct AttractorField {
    pub attractors: Vec<PointerAttractor>,
}

impl AttractorField {
    pub fn new() -> Self {
        Self {
            attractors: Vec::new(),
        }
    }

    /// Adds an attractor.
    pub fn push(&mut self, attractor: PointerAttractor) {
        self.attractors.push(attractor);
    }

    /// Clears all attractors.
    pub fn clear(&mut self) {
        self.attractors.clear();
    }

    /// Computes the net force at a given particle position.
    pub fn net_force_at(&self, pos: Vec2) -> Vec2 {
        let mut result = Vec2::ZERO;
        for a in &self.attractors {
            result += a.force_at(pos);
        }
        result
    }

    /// Adds a cursor-following attractor at the given screen position.
    pub fn add_cursor_attractor(&mut self, x: f32, y: f32, strength: f32, radius: f32) {
        self.attractors
            .push(PointerAttractor::new(x, y, strength, radius));
    }
}

impl Default for AttractorField {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// SDF COLLISION SHAPES
// =============================================================================

/// A signed distance field shape for particle collision.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SdfShape {
    /// Circle at (cx, cy) with radius.
    Circle { cx: f32, cy: f32, radius: f32 },
    /// Axis-aligned rectangle.
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    /// Rounded rectangle.
    RoundedRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
    },
}

impl SdfShape {
    /// Returns the signed distance from a point to this shape.
    /// Negative = inside, positive = outside.
    pub fn distance(&self, px: f32, py: f32) -> f32 {
        match *self {
            SdfShape::Circle { cx, cy, radius } => {
                let dx = px - cx;
                let dy = py - cy;
                (dx * dx + dy * dy).sqrt() - radius
            }
            SdfShape::Rect {
                x,
                y,
                width,
                height,
            } => {
                let dx = (x + width * 0.5 - px).abs() - width * 0.5;
                let dy = (y + height * 0.5 - py).abs() - height * 0.5;
                let outside = (dx.max(0.0) * dx.max(0.0) + dy.max(0.0) * dy.max(0.0)).sqrt();
                let inside = dx.max(dy).min(0.0);
                outside + inside
            }
            SdfShape::RoundedRect {
                x,
                y,
                width,
                height,
                radius,
            } => {
                let half_w = width * 0.5 - radius;
                let half_h = height * 0.5 - radius;
                let dx = (x + width * 0.5 - px).abs() - half_w;
                let dy = (y + height * 0.5 - py).abs() - half_h;
                let outside = (dx.max(0.0) * dx.max(0.0) + dy.max(0.0) * dy.max(0.0)).sqrt();
                let inside = dx.max(dy).min(0.0);
                outside + inside - radius
            }
        }
    }

    /// Returns the (unnormalized) gradient of the SDF at the given point,
    /// pointing away from the surface.
    pub fn gradient(&self, px: f32, py: f32) -> Vec2 {
        let eps = 0.5f32;
        let dx = self.distance(px + eps, py) - self.distance(px - eps, py);
        let dy = self.distance(px, py + eps) - self.distance(px, py - eps);
        Vec2::new(dx, dy)
    }

    /// Collides a particle against this shape, adjusting velocity if needed.
    /// Returns true if a collision occurred.
    pub fn collide_particle(
        &self,
        pos: &mut Vec2,
        vel: &mut Vec2,
        bounce: f32,
        friction: f32,
    ) -> bool {
        let dist = self.distance(pos.x, pos.y);
        if dist < 0.0 {
            // Inside the SDF -- push out
            let grad = self.gradient(pos.x, pos.y);
            let mut normal = grad.normalize_or_zero();
            if normal == Vec2::ZERO {
                normal = Vec2::new(1.0, 0.0);
            }
            *pos += normal * (-dist + 0.1);

            // Reflect velocity
            let dot = vel.dot(normal);
            if dot < 0.0 {
                *vel -= normal * dot * (1.0 + bounce);
                // Apply friction to tangential component
                let tangent = *vel - normal * vel.dot(normal);
                *vel = normal * vel.dot(normal) + tangent * (1.0 - friction);
            }
            true
        } else {
            false
        }
    }
}

// =============================================================================
// SoA PARTICLE SYSTEM
// =============================================================================

/// Structure-of-Arrays particle data for cache-friendly, SIMD-friendly iteration.
///
/// Stores particle properties in separate contiguous arrays rather than
/// interleaved structs, enabling:
/// - Better cache locality when updating a single property (e.g., only positions)
/// - Easier SIMD vectorization (process 4/8 particles at once)
/// - Efficient bulk operations (e.g., fade all alphas)
#[derive(Debug, Clone)]
pub struct SoaParticleSystem {
    /// Maximum number of particles this system can hold.
    pub capacity: usize,
    /// Current number of alive particles.
    pub count: usize,
    /// X positions.
    pub px: Vec<f32>,
    /// Y positions.
    pub py: Vec<f32>,
    /// X velocities.
    pub vx: Vec<f32>,
    /// Y velocities.
    pub vy: Vec<f32>,
    /// Lifetime remaining [0.0, 1.0].
    pub life: Vec<f32>,
    /// Initial lifetime (for computing decay curves).
    pub initial_life: Vec<f32>,
    /// Red channel.
    pub r: Vec<f32>,
    /// Green channel.
    pub g: Vec<f32>,
    /// Blue channel.
    pub b: Vec<f32>,
    /// Alpha channel.
    pub a: Vec<f32>,
    /// Scale multiplier.
    pub scale: Vec<f32>,
    /// Rotation angle in radians.
    pub rotation: Vec<f32>,
    /// Rotation speed in radians/sec.
    pub rotation_speed: Vec<f32>,
    /// Spline easing for color ramp (one per particle, or shared).
    pub color_curve: SplineEasing,
    /// Spline easing for scale ramp.
    pub scale_curve: SplineEasing,
    /// Gravity vector.
    pub gravity: Vec2,
    /// Drag coefficient (0 = no drag, 1 = full stop).
    pub drag: f32,
    /// Bounds for particle spawning.
    pub bounds: Rect,
    /// SDF collision shapes.
    pub collision_shapes: Vec<SdfShape>,
    /// Bounciness factor for SDF collisions.
    pub bounce: f32,
    /// Friction factor for SDF collisions.
    pub friction: f32,
    /// Reaction-diffusion flow field (optional).
    pub flow_field: Option<ReactionDiffusionField>,
    /// Flow field strength multiplier.
    pub flow_strength: f32,
    /// Attractor field.
    pub attractors: AttractorField,
    /// Spawn rate (particles per second).
    pub spawn_rate: f32,
    /// Spawn timer accumulator.
    pub spawn_timer: f32,
    /// Whether the emitter is active.
    pub active: bool,
}

impl SoaParticleSystem {
    /// Creates a new SoA particle system with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            count: 0,
            px: vec![0.0; capacity],
            py: vec![0.0; capacity],
            vx: vec![0.0; capacity],
            vy: vec![0.0; capacity],
            life: vec![0.0; capacity],
            initial_life: vec![0.0; capacity],
            r: vec![0.0; capacity],
            g: vec![0.0; capacity],
            b: vec![0.0; capacity],
            a: vec![0.0; capacity],
            scale: vec![1.0; capacity],
            rotation: vec![0.0; capacity],
            rotation_speed: vec![0.0; capacity],
            color_curve: SplineEasing::ease_in_out(),
            scale_curve: SplineEasing::linear(),
            gravity: Vec2::new(0.0, 50.0),
            drag: 0.01,
            bounds: Rect::new(0.0, 0.0, 800.0, 600.0),
            collision_shapes: Vec::new(),
            bounce: 0.6,
            friction: 0.1,
            flow_field: None,
            flow_strength: 30.0,
            attractors: AttractorField::new(),
            spawn_rate: 20.0,
            spawn_timer: 0.0,
            active: true,
        }
    }

    /// Sets the color ramp spline.
    pub fn with_color_curve(mut self, curve: SplineEasing) -> Self {
        self.color_curve = curve;
        self
    }

    /// Sets the scale ramp spline.
    pub fn with_scale_curve(mut self, curve: SplineEasing) -> Self {
        self.scale_curve = curve;
        self
    }

    /// Sets the gravity vector.
    pub fn with_gravity(mut self, gx: f32, gy: f32) -> Self {
        self.gravity = Vec2::new(gx, gy);
        self
    }

    /// Sets the drag coefficient.
    pub fn with_drag(mut self, drag: f32) -> Self {
        self.drag = drag.clamp(0.0, 1.0);
        self
    }

    /// Sets the spawn bounds.
    pub fn with_bounds(mut self, bounds: Rect) -> Self {
        self.bounds = bounds;
        self
    }

    /// Sets the bounciness for SDF collisions.
    pub fn with_bounce(mut self, bounce: f32) -> Self {
        self.bounce = bounce.clamp(0.0, 1.0);
        self
    }

    /// Attaches a reaction-diffusion flow field.
    pub fn with_flow_field(mut self, field: ReactionDiffusionField, strength: f32) -> Self {
        self.flow_field = Some(field);
        self.flow_strength = strength;
        self
    }

    /// Adds an SDF collision shape.
    pub fn add_collision_shape(&mut self, shape: SdfShape) {
        self.collision_shapes.push(shape);
    }

    /// Spawns a single particle at the given position.
    pub fn spawn(
        &mut self,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        life: f32,
        color: [f32; 4],
        scale: f32,
    ) {
        if self.count >= self.capacity {
            return;
        }
        let i = self.count;
        self.px[i] = x;
        self.py[i] = y;
        self.vx[i] = vx;
        self.vy[i] = vy;
        self.life[i] = life;
        self.initial_life[i] = life;
        self.r[i] = color[0];
        self.g[i] = color[1];
        self.b[i] = color[2];
        self.a[i] = color[3];
        self.scale[i] = scale;
        self.rotation[i] = 0.0;
        self.rotation_speed[i] = 0.0;
        self.count += 1;
    }

    /// Spawns a particle at a random position within the bounds.
    pub fn spawn_random(&mut self) {
        let mut rng = rand::rng();
        let x = rng.random_range(self.bounds.x..(self.bounds.x + self.bounds.width));
        let y = rng.random_range(self.bounds.y..(self.bounds.y + self.bounds.height));
        let vx = rng.random_range(-30.0..30.0);
        let vy = rng.random_range(-60.0..-10.0);
        let life = rng.random_range(1.0..3.0);
        let color = [0.0, 0.8, 1.0, 1.0];
        let scale = rng.random_range(0.5..2.0);
        self.spawn(x, y, vx, vy, life, color, scale);
    }

    /// Updates all particles by `dt` seconds.
    ///
    /// This is the core simulation step that processes:
    /// 1. Gravity and drag
    /// 2. Reaction-diffusion flow field forces
    /// 3. Pointer attractor forces
    /// 4. SDF collision detection and response
    /// 5. Lifetime decay with spline-eased color/scale
    /// 6. Dead particle removal (swap-remove)
    pub fn update(&mut self, dt: Duration) {
        let dt_secs = dt.as_secs_f32();

        // Update flow field if present
        if let Some(ref mut field) = self.flow_field {
            field.step_n(2);
        }

        // Update attractors
        let attractor_forces: Vec<Vec2> = (0..self.count)
            .map(|i| {
                let pos = Vec2::new(self.px[i], self.py[i]);
                self.attractors.net_force_at(pos)
            })
            .collect();

        // Main particle update loop -- SoA layout means each property
        // is updated in a tight, cache-friendly loop.
        for i in 0..self.count {
            // Apply gravity
            self.vx[i] += self.gravity.x * dt_secs;
            self.vy[i] += self.gravity.y * dt_secs;

            // Apply drag
            self.vx[i] *= 1.0 - self.drag;
            self.vy[i] *= 1.0 - self.drag;

            // Apply flow field
            if let Some(ref field) = self.flow_field {
                let u = (self.px[i] - self.bounds.x) / self.bounds.width;
                let v = (self.py[i] - self.bounds.y) / self.bounds.height;
                let flow = field.flow_at_uv(u.clamp(0.0, 1.0), v.clamp(0.0, 1.0));
                self.vx[i] += flow.x * self.flow_strength * dt_secs;
                self.vy[i] += flow.y * self.flow_strength * dt_secs;
            }

            // Apply attractor forces
            self.vx[i] += attractor_forces[i].x * dt_secs;
            self.vy[i] += attractor_forces[i].y * dt_secs;

            // Integrate position
            self.px[i] += self.vx[i] * dt_secs;
            self.py[i] += self.vy[i] * dt_secs;

            // SDF collision
            let mut pos = Vec2::new(self.px[i], self.py[i]);
            let mut vel = Vec2::new(self.vx[i], self.vy[i]);
            for shape in &self.collision_shapes {
                shape.collide_particle(&mut pos, &mut vel, self.bounce, self.friction);
            }
            self.px[i] = pos.x;
            self.py[i] = pos.y;
            self.vx[i] = vel.x;
            self.vy[i] = vel.y;

            // Decay lifetime
            self.life[i] -= dt_secs;

            // Spline-eased color and scale based on normalized lifetime
            let normalized_life = (self.life[i] / self.initial_life[i]).clamp(0.0, 1.0);
            let color = SplineEasing::color_ramp(
                normalized_life,
                &self.color_curve,
                &self.color_curve,
                &self.color_curve,
                &self.color_curve,
            );
            self.r[i] = color[0];
            self.g[i] = color[1];
            self.b[i] = color[2];
            self.a[i] = color[3] * normalized_life;

            // Spline-eased scale
            self.scale[i] = self.scale_curve.scale_curve(normalized_life, 0.2, 2.0);

            // Update rotation
            self.rotation[i] += self.rotation_speed[i] * dt_secs;
        }

        // Remove dead particles (swap-remove for O(1) removal)
        let mut i = 0;
        while i < self.count {
            if self.life[i] <= 0.0 {
                self.count -= 1;
                if i < self.count {
                    self.px[i] = self.px[self.count];
                    self.py[i] = self.py[self.count];
                    self.vx[i] = self.vx[self.count];
                    self.vy[i] = self.vy[self.count];
                    self.life[i] = self.life[self.count];
                    self.initial_life[i] = self.initial_life[self.count];
                    self.r[i] = self.r[self.count];
                    self.g[i] = self.g[self.count];
                    self.b[i] = self.b[self.count];
                    self.a[i] = self.a[self.count];
                    self.scale[i] = self.scale[self.count];
                    self.rotation[i] = self.rotation[self.count];
                    self.rotation_speed[i] = self.rotation_speed[self.count];
                }
            } else {
                i += 1;
            }
        }

        // Spawn new particles
        if self.active {
            self.spawn_timer += dt_secs;
            let interval = 1.0 / self.spawn_rate;
            while self.spawn_timer >= interval && self.count < self.capacity {
                self.spawn_timer -= interval;
                self.spawn_random();
            }
        }
    }

    /// Returns the number of alive particles.
    pub fn particle_count(&self) -> usize {
        self.count
    }

    /// Returns true if the system has no alive particles.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spline_easing_linear() {
        let e = SplineEasing::linear();
        assert!((e.evaluate(0.0) - 0.0).abs() < 0.05);
        assert!((e.evaluate(0.5) - 0.5).abs() < 0.05);
        assert!((e.evaluate(1.0) - 1.0).abs() < 0.05);
    }

    #[test]
    fn spline_easing_clamps() {
        let e = SplineEasing::ease_in_out();
        let v = e.evaluate(-0.5);
        assert!(v >= 0.0);
        let v = e.evaluate(1.5);
        assert!(v <= 1.0);
    }

    #[test]
    fn spline_color_ramp() {
        let r = SplineEasing::linear();
        let g = SplineEasing::ease_in_out();
        let b = SplineEasing::linear();
        let a = SplineEasing::linear();
        let color = SplineEasing::color_ramp(0.5, &r, &g, &b, &a);
        assert!(color[0] >= 0.0 && color[0] <= 1.0);
        assert!(color[3] >= 0.0 && color[3] <= 1.0);
    }

    #[test]
    fn spline_scale_curve() {
        let e = SplineEasing::linear();
        let s = e.scale_curve(0.5, 0.5, 2.0);
        assert!(s >= 0.5 && s <= 2.0);
    }

    #[test]
    fn reaction_diffusion_creates_field() {
        let mut field = ReactionDiffusionField::new(64, 64);
        field.step_n(10);
        let flow = field.flow_at_uv(0.5, 0.5);
        // Flow should be a valid vector
        assert!(flow.x.is_finite());
        assert!(flow.y.is_finite());
    }

    #[test]
    fn reaction_diffusion_b_concentration() {
        let mut field = ReactionDiffusionField::new(32, 32);
        field.step_n(5);
        let b = field.b_at_uv(0.5, 0.5);
        assert!(b >= 0.0 && b <= 1.0);
    }

    #[test]
    fn pointer_attractor_force() {
        let a = PointerAttractor::new(100.0, 100.0, 50.0, 200.0);
        let force = a.force_at(Vec2::new(100.0, 100.0));
        // At the center, force should be zero (dist < 0.1 check)
        assert_eq!(force, Vec2::ZERO);

        let force = a.force_at(Vec2::new(50.0, 100.0));
        // Should be attracted toward center (positive x force)
        assert!(force.x > 0.0);
    }

    #[test]
    fn pointer_attractor_outside_radius() {
        let a = PointerAttractor::new(0.0, 0.0, 10.0, 50.0);
        let force = a.force_at(Vec2::new(200.0, 200.0));
        assert_eq!(force, Vec2::ZERO);
    }

    #[test]
    fn attractor_field_net_force() {
        let mut field = AttractorField::new();
        field.push(PointerAttractor::new(100.0, 100.0, 50.0, 200.0));
        field.push(PointerAttractor::new(200.0, 200.0, -30.0, 200.0));
        let force = field.net_force_at(Vec2::new(150.0, 150.0));
        assert!(force.x.is_finite());
        assert!(force.y.is_finite());
    }

    #[test]
    fn sdf_circle_distance() {
        let circle = SdfShape::Circle {
            cx: 0.0,
            cy: 0.0,
            radius: 10.0,
        };
        assert!((circle.distance(0.0, 0.0) + 10.0).abs() < 0.01);
        assert!((circle.distance(10.0, 0.0)).abs() < 0.01);
        assert!((circle.distance(20.0, 0.0) - 10.0).abs() < 0.01);
    }

    #[test]
    fn sdf_rect_distance() {
        let rect = SdfShape::Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        // Center should be inside (negative distance)
        assert!(rect.distance(50.0, 25.0) < 0.0);
        // Far corner should be outside
        assert!(rect.distance(200.0, 200.0) > 0.0);
    }

    #[test]
    fn sdf_collision_pushes_out() {
        let circle = SdfShape::Circle {
            cx: 0.0,
            cy: 0.0,
            radius: 10.0,
        };
        let mut pos = Vec2::new(0.0, 0.0);
        let mut vel = Vec2::new(1.0, 0.0);
        let collided = circle.collide_particle(&mut pos, &mut vel, 0.5, 0.1);
        assert!(collided);
        // Position should have been pushed outside the circle
        assert!(circle.distance(pos.x, pos.y) >= -0.1);
    }

    #[test]
    fn soa_particle_spawn_and_update() {
        let mut system = SoaParticleSystem::new(100);
        system.spawn(400.0, 300.0, 0.0, -50.0, 2.0, [1.0, 0.5, 0.0, 1.0], 1.0);
        assert_eq!(system.count, 1);

        system.update(Duration::from_millis(16));
        assert_eq!(system.count, 1);
        // Particle should have moved down (gravity) and up (initial vy)
        assert!(system.py[0] != 300.0 || system.vy[0] != -50.0);
    }

    #[test]
    fn soa_particle_death() {
        let mut system = SoaParticleSystem::new(100);
        system.active = false;
        system.spawn(0.0, 0.0, 0.0, 0.0, 0.01, [1.0, 1.0, 1.0, 1.0], 1.0);
        assert_eq!(system.count, 1);

        system.update(Duration::from_millis(100));
        assert_eq!(system.count, 0);
        assert!(system.is_empty());
    }

    #[test]
    fn soa_capacity_limit() {
        let mut system = SoaParticleSystem::new(5);
        for _ in 0..10 {
            system.spawn_random();
        }
        assert_eq!(system.count, 5);
    }

    #[test]
    fn soa_with_flow_field() {
        let field = ReactionDiffusionField::new(32, 32);
        let mut system = SoaParticleSystem::new(50).with_flow_field(field, 20.0);
        system.spawn(400.0, 300.0, 0.0, 0.0, 5.0, [0.0, 0.8, 1.0, 1.0], 1.0);
        system.update(Duration::from_millis(16));
        assert_eq!(system.count, 1);
    }

    #[test]
    fn soa_with_collision() {
        let mut system = SoaParticleSystem::new(50);
        system.add_collision_shape(SdfShape::Circle {
            cx: 400.0,
            cy: 300.0,
            radius: 50.0,
        });
        system.spawn(400.0, 300.0, 0.0, 0.0, 5.0, [1.0, 1.0, 1.0, 1.0], 1.0);
        system.update(Duration::from_millis(16));
        assert_eq!(system.count, 1);
    }
}
