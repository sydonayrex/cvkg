#![allow(dead_code, clippy::approx_constant)]

use cvkg::prelude::*;
use cvkg_anim::skeletal::RagdollBlender;
use cvkg_components::context_menu::{ContextMenu, ContextMenuItem};
use cvkg_core::{DisplayEnvironment, PerformanceContract};
use cvkg_physics::ragdoll_bridge::RagdollBridge;
use cvkg_physics::{BodyId, Collider, Constraint, PhysicsWorld, RigidBody, Shape, WorldConfig};
use cvkg_vdom::signals::Signal;
use glam::Vec2;
use std::fs;
use std::path::PathBuf;

// --- Valknut Procedural Animation ---

/// Get a point along the valknut triangle path at parameter t.
/// pts is [p1, p2, p3, p1] (closed loop), t is in [0, 3).
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

/// Draw a single valknut triangle with fuse animation.
fn draw_valknut_triangle(
    r: &mut dyn cvkg_core::Renderer,
    p1: [f32; 2],
    p2: [f32; 2],
    p3: [f32; 2],
    color: [f32; 4],
    glow_color: [f32; 4],
    time: f32,
    speed: f32,
    offset: f32,
) {
    let pts = [p1, p2, p3, p1];

    // Draw the dim background triangle outline
    let dim_color = [color[0], color[1], color[2], 0.25];
    r.draw_line(p1[0], p1[1], p2[0], p2[1], dim_color, 2.5);
    r.draw_line(p2[0], p2[1], p3[0], p3[1], dim_color, 2.5);
    r.draw_line(p3[0], p3[1], p1[0], p1[1], dim_color, 2.5);

    // Fuse animation: a bright traveling segment
    let total_len = 3.0f32;
    let head = ((time + offset) * speed) % total_len;
    let tail_len = 0.5f32;
    let start = head - tail_len;
    if start < 0.0 {
        return;
    }

    // Draw the bright tracing segments
    let num_steps = 16;
    for i in 0..num_steps {
        let t1 = start + (i as f32 / num_steps as f32) * tail_len;
        let t2 = start + ((i + 1) as f32 / num_steps as f32) * tail_len;
        let p_start = get_triangle_point(&pts, t1);
        let p_end = get_triangle_point(&pts, t2);
        let alpha = i as f32 / num_steps as f32;
        let seg_color = [
            glow_color[0] * alpha + dim_color[0] * (1.0 - alpha),
            glow_color[1] * alpha + dim_color[1] * (1.0 - alpha),
            glow_color[2] * alpha + dim_color[2] * (1.0 - alpha),
            alpha,
        ];
        r.draw_line(p_start[0], p_start[1], p_end[0], p_end[1], seg_color, 3.0 + alpha * 2.0);
    }

    // Draw a bright spark at the fuse head
    if head >= 0.0 && head < total_len {
        let spark_pos = get_triangle_point(&pts, head);
        let spark_rect = cvkg_core::Rect {
            x: spark_pos[0] - 4.0,
            y: spark_pos[1] - 4.0,
            width: 8.0,
            height: 8.0,
        };
        r.draw_radial_gradient(spark_rect, [1.0, 1.0, 0.8, 0.9], glow_color);
    }
}
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

// --- Component State ---

struct PhysicsState {
    world: PhysicsWorld,
    cube_ids: Vec<(BodyId, f32)>,
    card_bodies: Vec<(BodyId, BodyId)>,
    dummy_head: BodyId,
    dummy_torso: BodyId,
}

struct AnimState {
    blender: RagdollBlender,
    bridge: RagdollBridge,
}

/// Grid resolution for the Navier-Stokes velocity field.
/// 24x24 keeps per-frame cost under 1ms while still producing fluid motion.
const FLUID_GRID_SIZE: usize = 24;

/// A 2D velocity-based Navier-Stokes grid approximation.
/// This simulates stable advection, projection, and diffusion to drive organic fluid flow for the flame.
struct FluidGrid {
    u: Vec<f32>,
    v: Vec<f32>,
    u_prev: Vec<f32>,
    v_prev: Vec<f32>,
    d: Vec<f32>,
    d_prev: Vec<f32>,
}

impl FluidGrid {
    /// Creates a new FluidGrid with zeroed velocity fields.
    fn new() -> Self {
        let size = FLUID_GRID_SIZE * FLUID_GRID_SIZE;
        Self {
            u: vec![0.0; size],
            v: vec![0.0; size],
            u_prev: vec![0.0; size],
            v_prev: vec![0.0; size],
            d: vec![0.0; size],
            d_prev: vec![0.0; size],
        }
    }

    /// Steps the fluid simulation using stable advection and projection.
    ///
    /// # Arguments
    /// * `dt` - Time step duration.
    /// * `viscosity` - Viscosity factor for diffusion.
    /// * `diffusion` - Diffusion factor for density/temperature.
    fn step(&mut self, dt: f32, viscosity: f32, diffusion: f32) {
        // Add sources
        for i in 0..self.u.len() {
            self.u[i] += dt * self.u_prev[i];
            self.v[i] += dt * self.v_prev[i];
            self.d[i] += dt * self.d_prev[i];
        }
        
        // Swap velocity fields for diffusion
        std::mem::swap(&mut self.u, &mut self.u_prev);
        diffuse(1, &mut self.u, &self.u_prev, viscosity, dt);
        
        std::mem::swap(&mut self.v, &mut self.v_prev);
        diffuse(2, &mut self.v, &self.v_prev, viscosity, dt);
        
        project(&mut self.u, &mut self.v, &mut self.u_prev, &mut self.v_prev);
        
        // Swap velocity fields for advection
        std::mem::swap(&mut self.u, &mut self.u_prev);
        std::mem::swap(&mut self.v, &mut self.v_prev);
        advect(1, &mut self.u, &self.u_prev, &self.u_prev, &self.v_prev, dt);
        advect(2, &mut self.v, &self.v_prev, &self.u_prev, &self.v_prev, dt);
        
        project(&mut self.u, &mut self.v, &mut self.u_prev, &mut self.v_prev);
        
        // Density step
        std::mem::swap(&mut self.d, &mut self.d_prev);
        diffuse(0, &mut self.d, &self.d_prev, diffusion, dt);
        std::mem::swap(&mut self.d, &mut self.d_prev);
        advect(0, &mut self.d, &self.d_prev, &self.u_prev, &self.v_prev, dt);
        
        // Decay velocity/density over time to prevent build-up
        for i in 0..self.u.len() {
            self.u[i] *= 0.95;
            self.v[i] *= 0.95;
            self.d[i] *= 0.95;
            // Clear source accumulators
            self.u_prev[i] = 0.0;
            self.v_prev[i] = 0.0;
            self.d_prev[i] = 0.0;
        }
    }

    /// Samples the velocity field at an arbitrary screen position using bilinear interpolation.
    ///
    /// # Arguments
    /// * `px` - Screen space X coordinate.
    /// * `py` - Screen space Y coordinate.
    /// * `w` - Total screen width.
    /// * `h` - Total screen height.
    fn get_velocity(&self, px: f32, py: f32, w: f32, h: f32) -> [f32; 2] {
        let n = FLUID_GRID_SIZE;
        let gx = (px / w * n as f32).clamp(0.5, n as f32 - 1.5);
        let gy = (py / h * n as f32).clamp(0.5, n as f32 - 1.5);
        
        let i0 = gx.floor() as usize;
        let i1 = i0 + 1;
        let j0 = gy.floor() as usize;
        let j1 = j0 + 1;
        
        let tx = gx - i0 as f32;
        let ty = gy - j0 as f32;
        
        let lerp_u = (1.0 - tx) * ((1.0 - ty) * self.u[i0 + j0 * n] + ty * self.u[i0 + j1 * n])
                   + tx * ((1.0 - ty) * self.u[i1 + j0 * n] + ty * self.u[i1 + j1 * n]);
                   
        let lerp_v = (1.0 - tx) * ((1.0 - ty) * self.v[i0 + j0 * n] + ty * self.v[i0 + j1 * n])
                   + tx * ((1.0 - ty) * self.v[i1 + j0 * n] + ty * self.v[i1 + j1 * n]);
                   
        [lerp_u, lerp_v]
    }
}

/// Applies boundary constraints to the fluid grid edges.
/// 
/// # Arguments
/// * `b` - Boundary type (0 for scalar, 1 for horizontal velocity, 2 for vertical velocity).
/// * `x` - Grid values to clamp/reflect.
fn set_bnd(b: i32, x: &mut [f32]) {
    let n = FLUID_GRID_SIZE;
    for i in 1..n - 1 {
        // Left and right edges
        x[0 * n + i] = if b == 1 { -x[1 * n + i] } else { x[1 * n + i] };
        x[(n - 1) * n + i] = if b == 1 { -x[(n - 2) * n + i] } else { x[(n - 2) * n + i] };
        // Top and bottom edges
        x[i * n + 0] = if b == 2 { -x[i * n + 1] } else { x[i * n + 1] };
        x[i * n + n - 1] = if b == 2 { -x[i * n + n - 2] } else { x[i * n + n - 2] };
    }
    // Corners
    x[0] = 0.5 * (x[1] + x[n]);
    x[n - 1] = 0.5 * (x[n - 2] + x[2 * n - 1]);
    x[(n - 1) * n] = 0.5 * (x[(n - 2) * n] + x[(n - 1) * n + 1]);
    x[(n - 1) * n + n - 1] = 0.5 * (x[(n - 2) * n + n - 1] + x[(n - 1) * n + n - 2]);
}

/// Diffuses a grid field using Jacobi relaxation.
///
/// # Arguments
/// * `b` - Boundary type.
/// * `x` - Output field.
/// * `x0` - Input field.
/// * `diff` - Diffusion rate.
/// * `dt` - Time step.
fn diffuse(b: i32, x: &mut [f32], x0: &[f32], diff: f32, dt: f32) {
    let n = FLUID_GRID_SIZE;
    let a = dt * diff * (n * n) as f32;
    // 3 Jacobi iterations: enough for visual stability at 120fps without budget overrun.
    for _ in 0..3 {
        for j in 1..n - 1 {
            for i in 1..n - 1 {
                let idx = i + j * n;
                x[idx] = (x0[idx] + a * (x[idx - 1] + x[idx + 1] + x[idx - n] + x[idx + n])) / (1.0 + 4.0 * a);
            }
        }
        set_bnd(b, x);
    }
}

/// Advects a quantity along the velocity fields using semi-Lagrangian back-tracing.
///
/// # Arguments
/// * `b` - Boundary type.
/// * `d` - Output field.
/// * `d0` - Input field.
/// * `u` - Horizontal velocity field.
/// * `v` - Vertical velocity field.
/// * `dt` - Time step.
fn advect(b: i32, d: &mut [f32], d0: &[f32], u: &[f32], v: &[f32], dt: f32) {
    let n = FLUID_GRID_SIZE;
    let dt0 = dt * n as f32;
    for j in 1..n - 1 {
        for i in 1..n - 1 {
            let idx = i + j * n;
            let mut x = i as f32 - dt0 * u[idx];
            let mut y = j as f32 - dt0 * v[idx];
            
            if x < 0.5 { x = 0.5; }
            if x > n as f32 - 1.5 { x = n as f32 - 1.5; }
            let i0 = x.floor() as usize;
            let i1 = i0 + 1;
            
            if y < 0.5 { y = 0.5; }
            if y > n as f32 - 1.5 { y = n as f32 - 1.5; }
            let j0 = y.floor() as usize;
            let j1 = j0 + 1;
            
            let s1 = x - i0 as f32;
            let s0 = 1.0 - s1;
            let t1 = y - j0 as f32;
            let t0 = 1.0 - t1;
            
            d[idx] = s0 * (t0 * d0[i0 + j0 * n] + t1 * d0[i0 + j1 * n])
                   + s1 * (t0 * d0[i1 + j0 * n] + t1 * d0[i1 + j1 * n]);
        }
    }
    set_bnd(b, d);
}

/// Projects the velocity fields to enforce mass conservation and incompressibility.
///
/// # Arguments
/// * `u` - Horizontal velocity field.
/// * `v` - Vertical velocity field.
/// * `p` - Pressure field scratch space.
/// * `div` - Divergence field scratch space.
fn project(u: &mut [f32], v: &mut [f32], p: &mut [f32], div: &mut [f32]) {
    let n = FLUID_GRID_SIZE;
    for j in 1..n - 1 {
        for i in 1..n - 1 {
            let idx = i + j * n;
            div[idx] = -0.5 * (u[idx + 1] - u[idx - 1] + v[idx + n] - v[idx - n]) / n as f32;
            p[idx] = 0.0;
        }
    }
    set_bnd(0, div);
    set_bnd(0, p);
    
    // 3 Jacobi iterations for the pressure projection pass.
    for _ in 0..3 {
        for j in 1..n - 1 {
            for i in 1..n - 1 {
                let idx = i + j * n;
                p[idx] = (div[idx] + p[idx - 1] + p[idx + 1] + p[idx - n] + p[idx + n]) / 4.0;
            }
        }
        set_bnd(0, p);
    }
    
    for j in 1..n - 1 {
        for i in 1..n - 1 {
            let idx = i + j * n;
            u[idx] -= 0.5 * n as f32 * (p[idx + 1] - p[idx - 1]);
            v[idx] -= 0.5 * n as f32 * (p[idx + n] - p[idx - n]);
        }
    }
    set_bnd(1, u);
    set_bnd(2, v);
}

/// Integrates a spring-damper state using Runge-Kutta 4th order.
/// This calculates a lagging trailing position that springs around when changing directions.
///
/// # Arguments
/// * `x` - Current position reference.
/// * `y` - Current position reference.
/// * `vx` - Current velocity reference.
/// * `vy` - Current velocity reference.
/// * `cx` - Target position X.
/// * `cy` - Target position Y.
/// * `dt` - Time step.
fn step_rk4(
    x: &mut f32,
    y: &mut f32,
    vx: &mut f32,
    vy: &mut f32,
    cx: f32,
    cy: f32,
    dt: f32,
) {
    let k = 150.0;
    let c = 8.0;

    let f = |pos_x: f32, pos_y: f32, vel_x: f32, vel_y: f32| -> (f32, f32, f32, f32) {
        let ax = -k * (pos_x - cx) - c * vel_x;
        let ay = -k * (pos_y - cy) - c * vel_y;
        (vel_x, vel_y, ax, ay)
    };

    let (dx1, dy1, dvx1, dvy1) = f(*x, *y, *vx, *vy);
    let (dx2, dy2, dvx2, dvy2) = f(*x + 0.5 * dt * dx1, *y + 0.5 * dt * dy1, *vx + 0.5 * dt * dvx1, *vy + 0.5 * dt * dvy1);
    let (dx3, dy3, dvx3, dvy3) = f(*x + 0.5 * dt * dx2, *y + 0.5 * dt * dy2, *vx + 0.5 * dt * dvx2, *vy + 0.5 * dt * dvy2);
    let (dx4, dy4, dvx4, dvy4) = f(*x + dt * dx3, *y + dt * dy3, *vx + dt * dvx3, *vy + dt * dvy3);

    *x += (dt / 6.0) * (dx1 + 2.0 * dx2 + 2.0 * dx3 + dx4);
    *y += (dt / 6.0) * (dy1 + 2.0 * dy2 + 2.0 * dy3 + dy4);
    *vx += (dt / 6.0) * (dvx1 + 2.0 * dvx2 + 2.0 * dvx3 + dvx4);
    *vy += (dt / 6.0) * (dvy1 + 2.0 * dvy2 + 2.0 * dvy3 + dvy4);
}

struct BerserkerState {
    particles: Vec<Particle>,
    rng: Lcg,
    last_time: f32,
    physics: PhysicsState,
    anim: AnimState,
    loaded_svgs: bool,
    fluid: FluidGrid,
    flame_x: f32,
    flame_y: f32,
    flame_vx: f32,
    flame_vy: f32,
    last_cx: f32,
    last_cy: f32,
}

// --- Valknut Triangle Geometry ---
// Three interlocking triangles forming the valknut symbol.
// Coordinates match the original SVG viewBox 0 0 100 100, centered at (50, 50).
// Each triangle is defined by 3 points.

/// Draw the valknut symbol with procedural fuse animation.
fn draw_valknut(r: &mut dyn cvkg_core::Renderer, cx: f32, cy: f32, size: f32, time: f32) {
    // Triangle 1 (top, pointing up) -- original SVG: M50,15 L28.3,52.5 L71.7,52.5 Z
    let t1_p1 = [cx, cy - size * 0.35];
    let t1_p2 = [cx - size * 0.217, cy + size * 0.075];
    let t1_p3 = [cx + size * 0.217, cy + size * 0.075];

    // Triangle 2 (bottom-right) -- original SVG: M18.3,72.5 L61.7,72.5 L40,35 Z
    let t2_p1 = [cx - size * 0.317, cy + size * 0.225];
    let t2_p2 = [cx + size * 0.117, cy + size * 0.225];
    let t2_p3 = [cx - size * 0.10, cy - size * 0.15];

    // Triangle 3 (bottom-left) -- original SVG: M81.7,72.5 L60,35 L38.3,72.5 Z
    let t3_p1 = [cx + size * 0.317, cy + size * 0.225];
    let t3_p2 = [cx + size * 0.10, cy - size * 0.15];
    let t3_p3 = [cx - size * 0.117, cy + size * 0.225];

    // Colors matching the original SVG strokes
    let c1 = [1.0, 0.25, 0.0, 1.0];    // #FF4000
    let c2 = [1.0, 0.5, 0.0, 1.0];     // #FF8000
    let c3 = [1.0, 0.75, 0.0, 1.0];    // #FFC000

    let g1 = [1.0, 0.4, 0.1, 1.0];
    let g2 = [1.0, 0.6, 0.1, 1.0];
    let g3 = [1.0, 0.8, 0.2, 1.0];

    // Draw each triangle with fuse animation, offset in time
    draw_valknut_triangle(r, t1_p1, t1_p2, t1_p3, c1, g1, time, 1.2, 0.0);
    draw_valknut_triangle(r, t2_p1, t2_p2, t2_p3, c2, g2, time, 1.2, 1.0);
    draw_valknut_triangle(r, t3_p1, t3_p2, t3_p3, c3, g3, time, 1.2, 2.0);
}

impl BerserkerState {
    fn new(w: f32, h: f32) -> Self {
        let mut rng = Lcg::new(1337);
        let mut world = PhysicsWorld::new(WorldConfig {
            gravity: Vec2::new(0.0, 150.0),
            ..Default::default()
        });

        world.on_constraint_broken = Some(Box::new(|_c, pos| {
            log::info!("CONSTRAINT BROKEN AT {:?}", pos);
        }));

        let mut cube_ids = Vec::new();
        // Cubes removed for performance — 15 bodies + colliders was ~8ms/frame in XPBD solver

        // Static bounds
        let mut ground = RigidBody::static_body();
        ground.position = Vec2::new(w / 2.0, h + 50.0);
        let ground_id = world.add_body(ground);
        world.add_collider(Collider::new(ground_id, Shape::aabb(Vec2::new(w, 50.0))));

        let mut left_wall = RigidBody::static_body();
        left_wall.position = Vec2::new(-50.0, h / 2.0);
        let lw_id = world.add_body(left_wall);
        world.add_collider(Collider::new(lw_id, Shape::aabb(Vec2::new(50.0, h))));

        let mut right_wall = RigidBody::static_body();
        right_wall.position = Vec2::new(w + 50.0, h / 2.0);
        let rw_id = world.add_body(right_wall);
        world.add_collider(Collider::new(rw_id, Shape::aabb(Vec2::new(50.0, h))));

        let mut top_wall = RigidBody::static_body();
        top_wall.position = Vec2::new(w / 2.0, -50.0);
        let tw_id = world.add_body(top_wall);
        world.add_collider(Collider::new(tw_id, Shape::aabb(Vec2::new(w, 50.0))));

        // Glass cards (breakable)
        let mut card_bodies = Vec::new();
        let card_positions = [[w * 0.2, h * 0.3], [w * 0.7, h * 0.2], [w * 0.5, h * 0.7]];
        for pos in card_positions {
            let shape = Shape::aabb(Vec2::new(100.0, 125.0));
            let mut left_half = RigidBody::new(10.0, &shape);
            left_half.position = Vec2::new(pos[0] - 100.0, pos[1]);
            let id_l = world.add_body(left_half);
            world.add_collider(Collider::new(id_l, shape.clone()));

            let mut right_half = RigidBody::new(10.0, &shape);
            right_half.position = Vec2::new(pos[0] + 100.0, pos[1]);
            let id_r = world.add_body(right_half);
            world.add_collider(Collider::new(id_r, shape));

            let mut constraint = Constraint::pin(id_l, id_r, Vec2::new(pos[0], pos[1]));
            constraint.break_threshold = Some(500.0); // Lower threshold so cards shatter on impact
            world.add_constraint(constraint);
            world.add_constraint(Constraint::distance(
                ground_id,
                id_l,
                Vec2::new(pos[0] - w / 2.0 - 100.0, pos[1] - h),
                Vec2::new(0.0, 0.0),
                0.0,
            ));
            card_bodies.push((id_l, id_r));
        }

        // Ragdoll Dummy — removed, was drawing orange/red rectangles at fixed position
        // without actual skeletal animation. Replaced with empty vec.
        let dummy_head = world.add_body(RigidBody::static_body());
        let dummy_torso = world.add_body(RigidBody::static_body());
        // Suppress unused variable warnings
        let _ = (dummy_head, dummy_torso);

        // Ragdoll bridge/blender removed — was drawing static orange/red rectangles
        // without actual skeletal animation. Keep dummy bodies as static for physics
        // world stability but skip bridge setup.
        let bridge = cvkg_physics::RagdollBridge::new(cvkg_physics::RagdollBridgeConfig::default());
        let blender = RagdollBlender::new(2);

        log::info!(
            "BerserkerState initialized: {} cubes, {} cards",
            cube_ids.len(),
            card_bodies.len()
        );

        let cx = w * 0.5;
        let cy = h * 0.5;

        Self {
            particles: Vec::new(),
            rng,
            last_time: 0.0,
            physics: PhysicsState {
                world,
                cube_ids,
                card_bodies,
                dummy_head,
                dummy_torso,
            },
            anim: AnimState { blender, bridge },
            loaded_svgs: false,
            fluid: FluidGrid::new(),
            flame_x: cx,
            flame_y: cy,
            flame_vx: 0.0,
            flame_vy: 0.0,
            last_cx: cx,
            last_cy: cy,
        }
    }
}

struct BerserkerFireView {
    counters: [Signal<u32>; 4],
    rage: Signal<f32>,
    active_menu: Signal<Option<usize>>,
    state: Arc<Mutex<BerserkerState>>,
    perf: Arc<Mutex<cvkg_components::perf_overlay::PerfOverlay>>,
}

impl BerserkerFireView {
    /// Create a new BerserkerFireView instance with counters, rage trackers, and performance profiling overlay.
    fn new(w: f32, h: f32) -> Self {
        log::info!("Creating BerserkerFireView ({}x{})", w, h);
        Self {
            counters: [
                Signal::new(0),
                Signal::new(0),
                Signal::new(0),
                Signal::new(0),
            ],
            rage: Signal::new(0.0),
            active_menu: Signal::new(None),
            state: Arc::new(Mutex::new(BerserkerState::new(w, h))),
            perf: Arc::new(Mutex::new(
                cvkg_components::perf_overlay::PerfOverlay::new().show(),
            )),
        }
    }
}

impl View for BerserkerFireView {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    /// Track last-known signal values to detect actual state changes.
    /// Returns true only when a signal's value has actually changed since last check.
    fn changed(&self) -> bool {
        // Snapshot current values and compare against previous frame.
        let rage_val = self.rage.get();
        let menu_val = self.active_menu.get();
        let counter_vals: [u32; 4] = [
            self.counters[0].get(),
            self.counters[1].get(),
            self.counters[2].get(),
            self.counters[3].get(),
        ];

        thread_local! {
            static LAST_RAGE: std::cell::Cell<f32> = const { std::cell::Cell::new(f32::NAN) };
            static LAST_MENU: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
            static LAST_COUNTERS: std::cell::Cell<[u32; 4]> = const { std::cell::Cell::new([0; 4]) };
        }

        let changed = LAST_RAGE.with(|l| {
            let prev = l.get();
            l.set(rage_val);
            prev != rage_val
        }) || LAST_MENU.with(|l| {
            let prev = l.get();
            l.set(menu_val);
            prev != menu_val
        }) || LAST_COUNTERS.with(|l| {
            let prev = l.get();
            l.set(counter_vals);
            prev != counter_vals
        });

        changed
    }

    /// Render is the main entry point to draw the Berserker view frame.
    /// CONTRACT: This function is called once per frame. It simulates physics,
    /// decays the Berserker rage over time, updates the GPU uniforms, and renders elements.
    fn render(&self, r: &mut dyn cvkg_core::Renderer, rect: cvkg_core::Rect) {
        let w = rect.width;
        let h = rect.height;

        // Push the root VNode to establish a parent for all interactive elements and decorative batches
        r.push_vnode(rect, "BerserkerRoot");

        // Draw background image (prewarmed on first frame via NativeRenderer::run assets)
        // Push the background's depth far away to prevent depth-buffer Z-fighting with 3D/2D elements.
        r.set_z_index(900.0);
        r.draw_image("background", cvkg_core::Rect { x: 0.0, y: 0.0, width: w, height: h });
        r.set_z_index(0.0);

        // Record telemetry data to the PerfOverlay
        let tel = r.get_telemetry();
        {
            let mut perf = self.perf.lock().expect("Failed to lock PerfOverlay");
            let tris = tel.vertices / 3;
            perf.record_frame(tel.frame_time_ms, tel.draw_calls, tris, tel.vertices);
        }

        let t = r.elapsed_time();
        let mut s = self.state.lock().expect("Berserker state mutex poisoned");

        // Calculate delta time
        let dt = if t > s.last_time {
            let elapsed = (t - s.last_time).min(0.1);
            s.last_time = t;
            elapsed
        } else {
            0.0
        };

        // Naturally decay rage over time (e.g. 15% per second)
        let current_rage = self.rage.get();
        let new_rage = (current_rage - dt * 15.0).max(0.0);
        self.rage.set(new_rage);

        // Propagate rage values to the GPU renderer (normalized to 0.0 - 1.0)
        r.set_rage(new_rage / 100.0);

        // Run the physics and particle simulation step only on active frame ticks (dt > 0)
        // to prevent layout VDOM builds from spawning static ghost particles.
        let t_sim_start = std::time::Instant::now();
        if dt > 0.0 {
            update_berserker_simulation(&mut s, w, h, t, dt, new_rage);
        }
        let t_sim = t_sim_start.elapsed().as_secs_f32() * 1000.0;

        // Cubes removed for performance

        // Draw the glass cards
        let t_cards_start = std::time::Instant::now();
        draw_glass_cards(r, &s, w, h, t);
        let t_cards = t_cards_start.elapsed().as_secs_f32() * 1000.0;

        // Draw the valknut symbol with procedural fuse animation
        let t_valknut_start = std::time::Instant::now();
        let vk_cx = w / 2.0;
        let vk_cy = h / 2.0 - 100.0;
        let vk_size = 120.0;
        draw_valknut(r, vk_cx, vk_cy, vk_size, t);
        let t_valknut = t_valknut_start.elapsed().as_secs_f32() * 1000.0;

        let t_fire_start = std::time::Instant::now();
        let mut t_ragdoll = 0.0f32;
        if t > 0.0 {
            // Clip fire to content area (safe area top is now 0 in content coordinates)
            r.push_clip_rect(cvkg_core::Rect { x: 0.0, y: 0.0, width: w, height: h });
            // Draw particles and Mjolnir lightning bolts
            draw_berserker_fire(r, &s, w, h, t);
            // Draw skeletal/ragdoll elements
            let t_ragdoll_start = std::time::Instant::now();
            draw_ragdoll_dummy(r, &mut s, w, h, new_rage);
            t_ragdoll = t_ragdoll_start.elapsed().as_secs_f32() * 1000.0;
            r.pop_clip_rect();
        }
        let t_fire = t_fire_start.elapsed().as_secs_f32() * 1000.0;

        // Draw chrome components (no state needed)
        let t_chrome_start = std::time::Instant::now();
        draw_corner_buttons(r, &self.counters, &self.rage, w, h);
        draw_dock(r, &self.counters, &self.rage, w, h);
        // Note: draw_nornir_bar is drawn LAST to ensure menu items appear above
        // corner button counter text and other chrome elements
        let t_chrome = t_chrome_start.elapsed().as_secs_f32() * 1000.0;

        if (s.last_time as u32).is_multiple_of(5) {
            log::info!(
                "[Berserker] Draw timings: sim={:.2}ms cards={:.2}ms valknut={:.2}ms fire={:.2}ms ragdoll={:.2}ms chrome={:.2}ms",
                t_sim,
                t_cards,
                t_valknut,
                t_fire,
                t_ragdoll,
                t_chrome
            );
        }

        // Draw the performance overlay
        {
            let perf = self.perf.lock().expect("Failed to lock PerfOverlay");
            perf.render(r, rect);
        }

        // Draw the Nornir bar LAST so menu items always appear above corner buttons/counters
        draw_nornir_bar(
            r,
            &self.counters,
            &self.rage,
            &self.active_menu,
            &self.perf,
            w,
            h,
        );

        // Pop the root VNode
        r.pop_vnode();
    }
}

fn draw_nornir_bar(
    r: &mut dyn cvkg_core::Renderer,
    _counters: &[Signal<u32>; 4],
    _rage: &Signal<f32>,
    active_menu: &Signal<Option<usize>>,
    perf: &Arc<std::sync::Mutex<cvkg_components::perf_overlay::PerfOverlay>>,
    w: f32,
    h: f32,
) {
    let bar_rect = cvkg_core::Rect {
        x: 0.0,
        y: 0.0,
        width: w,
        height: 28.0,
    };
    // Glass background: uses glass material pipeline for frosted blur effect
    r.fill_glass_rect_with_intensity(bar_rect, 4.0, 12.0, 0.35);

    let menu_x = 8.0;
    let items = [
        ("File", 60.0),
        ("Edit", 60.0),
        ("View", 70.0),
        ("Window", 80.0),
        ("Help", 60.0),
    ];
    let mut x = menu_x;
    for (i, (label, width)) in items.iter().enumerate() {
        let item_rect = cvkg_core::Rect {
            x,
            y: 0.0,
            width: *width,
            height: 28.0,
        };
        r.push_vnode(item_rect, "NornirBarItem");
        let (lw, lh) = r.measure_text(label, 13.0);
        let tx = x + (*width - lw) / 2.0;
        let ty = (28.0 - lh) * 0.5;
        r.draw_text(label, tx + 1.0, ty + 1.0, 13.0, [0.0, 0.0, 0.0, 0.35]);
        r.draw_text(label, tx, ty, 13.0, [0.95, 0.95, 0.98, 1.0]);
        let active_menu_clone = active_menu.clone();
        let h_closure = Arc::new(move |_| {
            let current = active_menu_clone.get();
            log::info!("NornirBarItem {} clicked! current={:?}", i, current);
            if current == Some(i) {
                active_menu_clone.set(None);
            } else {
                active_menu_clone.set(Some(i));
            }
        });
        r.register_handler("pointerclick", h_closure);
        r.pop_vnode();
        x += width;
    }

    // Centered window title
    let title_str = format!("BERSERKER v{}", env!("CARGO_PKG_VERSION"));
    let (tw, tlh) = r.measure_text(&title_str, 14.0);
    let title_x = (w - tw) / 2.0;
    let title_y = (28.0 - tlh) * 0.5;
    r.draw_text(&title_str, title_x + 1.0, title_y + 1.0, 14.0, [0.0, 0.0, 0.0, 0.35]);
    r.draw_text(&title_str, title_x, title_y, 14.0, [1.0, 0.35, 0.15, 1.0]);

    // Right-aligned, vertically centered rage meter
    let rage_str = format!("Rage: {:.0}%", _rage.get());
    let (rw, rlh) = r.measure_text(&rage_str, 12.0);
    let rage_x = w - rw - 16.0;
    let rage_y = (28.0 - rlh) * 0.5;
    r.draw_text(&rage_str, rage_x + 1.0, rage_y + 1.0, 12.0, [0.0, 0.0, 0.0, 0.35]);
    r.draw_text(&rage_str, rage_x, rage_y, 12.0, [0.0, 1.0, 0.55, 1.0]);

    // Render the active dropdown menu if open
    if let Some(open_idx) = active_menu.get() {
        // 1. Fullscreen invisible overlay to capture click-outside and dismiss the dropdown
        let overlay_rect = cvkg_core::Rect {
            x: 0.0,
            y: 0.0,
            width: w,
            height: h,
        };
        r.push_vnode(overlay_rect, "DropdownOverlay");
        let active_menu_clone = active_menu.clone();
        r.register_handler(
            "pointerdown",
            Arc::new(move |_| {
                active_menu_clone.set(None);
            }),
        );
        r.pop_vnode();

        let mut menu_pos_x = menu_x;
        for item in items.iter().take(open_idx) {
            menu_pos_x += item.1;
        }

        let menu_items = match open_idx {
            0 => vec![
                ContextMenuItem::new("New Canvas")
                    .shortcut("Ctrl+N")
                    .on_click({
                        let active_menu = active_menu.clone();
                        move || {
                            log::info!("New Canvas clicked");
                            active_menu.set(None);
                        }
                    }),
                ContextMenuItem::new("Open Runes")
                    .shortcut("Ctrl+O")
                    .on_click({
                        let active_menu = active_menu.clone();
                        move || {
                            log::info!("Open Runes clicked");
                            active_menu.set(None);
                        }
                    }),
                ContextMenuItem::new("Save Preset")
                    .shortcut("Ctrl+S")
                    .on_click({
                        let active_menu = active_menu.clone();
                        move || {
                            log::info!("Save Preset clicked");
                            active_menu.set(None);
                        }
                    }),
                ContextMenuItem::new("Exit Demo")
                    .shortcut("Ctrl+Q")
                    .on_click(|| {
                        std::process::exit(0);
                    }),
            ],
            1 => vec![
                ContextMenuItem::new("Undo").shortcut("Ctrl+Z").on_click({
                    let active_menu = active_menu.clone();
                    move || {
                        log::info!("Undo clicked");
                        active_menu.set(None);
                    }
                }),
                ContextMenuItem::new("Redo").shortcut("Ctrl+Y").on_click({
                    let active_menu = active_menu.clone();
                    move || {
                        log::info!("Redo clicked");
                        active_menu.set(None);
                    }
                }),
                ContextMenuItem::new("Cut")
                    .shortcut("Ctrl+X")
                    .disabled(true),
                ContextMenuItem::new("Copy").shortcut("Ctrl+C").on_click({
                    let active_menu = active_menu.clone();
                    move || {
                        log::info!("Copy clicked");
                        active_menu.set(None);
                    }
                }),
                ContextMenuItem::new("Paste").shortcut("Ctrl+V").on_click({
                    let active_menu = active_menu.clone();
                    move || {
                        log::info!("Paste clicked");
                        active_menu.set(None);
                    }
                }),
            ],
            2 => vec![
                ContextMenuItem::new("Toggle Performance Overlay")
                    .shortcut("Ctrl+Shift+P")
                    .on_click({
                        let perf = perf.clone();
                        let active_menu = active_menu.clone();
                        move || {
                            let mut p = perf.lock().unwrap();
                            p.visible = !p.visible;
                            active_menu.set(None);
                        }
                    }),
                ContextMenuItem::new("Zoom In")
                    .shortcut("Ctrl+=")
                    .on_click({
                        let active_menu = active_menu.clone();
                        move || {
                            log::info!("Zoom In clicked");
                            active_menu.set(None);
                        }
                    }),
                ContextMenuItem::new("Zoom Out")
                    .shortcut("Ctrl+-")
                    .on_click({
                        let active_menu = active_menu.clone();
                        move || {
                            log::info!("Zoom Out clicked");
                            active_menu.set(None);
                        }
                    }),
            ],
            3 => vec![
                ContextMenuItem::new("Minimize")
                    .shortcut("Ctrl+M")
                    .on_click({
                        let active_menu = active_menu.clone();
                        move || {
                            log::info!("Minimize clicked");
                            active_menu.set(None);
                        }
                    }),
                ContextMenuItem::new("Close Window")
                    .shortcut("Ctrl+W")
                    .on_click(|| {
                        std::process::exit(0);
                    }),
            ],
            _ => vec![
                ContextMenuItem::new("Viking Codex")
                    .shortcut("F1")
                    .on_click({
                        let active_menu = active_menu.clone();
                        move || {
                            log::info!("Viking Codex clicked");
                            active_menu.set(None);
                        }
                    }),
                ContextMenuItem::new("About Berserker").on_click({
                    let active_menu = active_menu.clone();
                    move || {
                        log::info!("About Berserker clicked");
                        active_menu.set(None);
                    }
                }),
            ],
        };

        let dropdown = ContextMenu::new(menu_items)
            .position(menu_pos_x, 28.0)
            .open(true);
        dropdown.render(
            r,
            cvkg_core::Rect {
                x: 0.0,
                y: 0.0,
                width: w,
                height: h,
            },
        );
    }
}

fn draw_dock(
    r: &mut dyn cvkg_core::Renderer,
    _counters: &[Signal<u32>; 4],
    _rage: &Signal<f32>,
    w: f32,
    h: f32,
) {
    let dock_rect = cvkg_core::Rect {
        x: w * 0.3,
        y: h - 68.0,
        width: w * 0.4,
        height: 56.0,
    };
    // Glass background: uses glass material pipeline for frosted blur effect
    r.fill_glass_rect_with_intensity(dock_rect, 12.0, 16.0, 0.28);

    let icons = ["ATK", "RGE", "DEF", "CRT", "ULT"];
    let icon_size = 48.0;
    let spacing = 16.0;
    let total_width = icons.len() as f32 * icon_size + (icons.len() - 1) as f32 * spacing;
    let start_x = dock_rect.x + (dock_rect.width - total_width) / 2.0;

    for (i, icon) in icons.iter().enumerate() {
        let ix = start_x + i as f32 * (icon_size + spacing);
        let slot_rect = cvkg_core::Rect {
            x: ix,
            y: dock_rect.y,
            width: icon_size,
            height: dock_rect.height,
        };
        r.push_vnode(slot_rect, "HeimdallDockItem");

        // Center text horizontally and vertically inside the 56px high dock using measured bounds
        let text_size = 18.0;
        let (tw, th) = r.measure_text(icon, text_size);
        let tx = ix + (icon_size - tw) / 2.0;
        let ty = dock_rect.y + (dock_rect.height - th) / 2.0;
        r.draw_text(icon, tx + 1.0, ty + 1.0, text_size, [0.0, 0.0, 0.0, 0.35]);
        r.draw_text(icon, tx, ty, text_size, [0.98, 0.98, 1.0, 1.0]);

        if i < 3 {
            // Center the dot horizontally below the icon cell
            let dot_size = 4.0;
            let dot_rect = cvkg_core::Rect {
                x: ix + icon_size / 2.0 - dot_size / 2.0,
                y: dock_rect.y + dock_rect.height - 8.0,
                width: dot_size,
                height: dot_size,
            };
            let accent = [0.0, 1.0, 0.95, 1.0];
            r.fill_ellipse(dot_rect, accent);
        }

        // Register handlers for interactive click feedback
        let c_signal = _counters[i.min(3)].clone();
        let r_signal = _rage.clone();
        let icon_name = icon.to_string();
        let h_closure = Arc::new(move |_| {
            c_signal.set(c_signal.get() + 1);
            r_signal.set((r_signal.get() + 20.0).min(100.0));
            log::info!(
                "Dock item '{}' clicked! Total count: {}",
                icon_name,
                c_signal.get()
            );
        });
        r.register_handler("pointerclick", h_closure);
        r.pop_vnode();
    }
}

fn draw_glass_cards(
    r: &mut dyn cvkg_core::Renderer,
    s: &BerserkerState,
    _w: f32,
    _h: f32,
    _t: f32,
) {
    let runes = ["CVKG!", "CVKG!", "CVKG!"];

    for (i, &(id_l, id_r)) in s.physics.card_bodies.iter().enumerate() {
        if let (Some(bl), Some(br)) = (s.physics.world.body(id_l), s.physics.world.body(id_r)) {
            let dx = bl.position.x - br.position.x;
            let dy = bl.position.y - br.position.y;
            let dist = (dx * dx + dy * dy).sqrt();

            // Card halves are 200 units wide each, centered at body positions.
            // When intact, bodies are ~200 units apart. When broken, they separate.
            let card_width = 200.0;
            let card_height = 250.0;
            let half_w = card_width * 0.5;
            let half_h = card_height * 0.5;

            if dist < card_width * 1.5 {
                // Card is intact or nearly so: render as single centered quad with glass material
                let cx = (bl.position.x + br.position.x) * 0.5;
                let cy = (bl.position.y + br.position.y) * 0.5;
                let rect = cvkg_core::Rect {
                    x: cx - half_w,
                    y: cy - half_h,
                    width: card_width,
                    height: card_height,
                };
                r.fill_glass_rect_with_intensity(rect, 12.0, 12.0, 0.38);
                let (rw, rh) = r.measure_text(runes[i % runes.len()], 32.0);
                r.draw_text(
                    runes[i % runes.len()],
                    cx - rw / 2.0 + 1.0,
                    cy - rh / 2.0 + 1.0,
                    32.0,
                    [0.0, 0.0, 0.0, 0.35],
                );
                r.draw_text(
                    runes[i % runes.len()],
                    cx - rw / 2.0,
                    cy - rh / 2.0,
                    32.0,
                    [0.85, 0.95, 1.0, 1.0],
                );
            } else {
                // Card has broken apart: render two separate halves
                let rect_l = cvkg_core::Rect {
                    x: bl.position.x - half_w,
                    y: bl.position.y - half_h,
                    width: card_width,
                    height: card_height,
                };
                r.fill_glass_rect_with_intensity(rect_l, 12.0, 8.0, 0.24);
                let rect_r = cvkg_core::Rect {
                    x: br.position.x - half_w,
                    y: br.position.y - half_h,
                    width: card_width,
                    height: card_height,
                };
                r.fill_glass_rect_with_intensity(rect_r, 12.0, 8.0, 0.24);
            }
        }
    }
}

fn draw_ragdoll_dummy(
    r: &mut dyn cvkg_core::Renderer,
    s: &mut BerserkerState,
    _w: f32,
    _h: f32,
    rage: f32,
) {
    // Don't draw ragdoll when no rage — avoids visible orange/red rectangles at origin
    if rage <= 0.0 {
        return;
    }
    s.anim.bridge.update(&s.physics.world);
    let transforms = s.anim.bridge.physics_transforms().to_vec();
    s.anim.blender.set_physics(&transforms);
    let blend_weight = (rage / 5.0).clamp(0.0, 1.0);
    s.anim.blender.blend(blend_weight);
    let poses = s.anim.blender.update(0.016);

    let head_pos = poses[0].0;
    r.fill_rounded_rect(
        cvkg_core::Rect {
            x: head_pos.x - 20.0,
            y: head_pos.y - 20.0,
            width: 40.0,
            height: 40.0,
        },
        8.0,
        [0.9, 0.2, 0.2, 1.0],
    );

    let torso_pos = poses[1].0;
    r.fill_rounded_rect(
        cvkg_core::Rect {
            x: torso_pos.x - 30.0,
            y: torso_pos.y - 50.0,
            width: 60.0,
            height: 100.0,
        },
        12.0,
        [0.8, 0.4, 0.1, 1.0],
    );
}

fn update_berserker_simulation(s: &mut BerserkerState, w: f32, h: f32, t: f32, dt: f32, rage: f32) {
    let cx = w * 0.5 + (t * 1.2).cos() * (w * 0.3);
    let cy = h * 0.5 + (t * 0.8).sin() * (h * 0.25);

    // Update RK4 physics for spring-mass tail anchor
    step_rk4(&mut s.flame_x, &mut s.flame_y, &mut s.flame_vx, &mut s.flame_vy, cx, cy, dt);
    let d_x = s.flame_x - cx;
    let d_y = s.flame_y - cy;

    // Fireball velocity
    let f_vx = if dt > 0.0 { (cx - s.last_cx) / dt } else { 0.0 };
    let f_vy = if dt > 0.0 { (cy - s.last_cy) / dt } else { 0.0 };
    s.last_cx = cx;
    s.last_cy = cy;

    // Inject velocity/force impulses into fluid grid around the fireball position
    let n = FLUID_GRID_SIZE;
    let cell_x = (cx / w * n as f32) as i32;
    let cell_y = (cy / h * n as f32) as i32;
    for dy in -2..=2 {
        for dx in -2..=2 {
            let gx = cell_x + dx;
            let gy = cell_y + dy;
            if gx >= 1 && gx < n as i32 - 1 && gy >= 1 && gy < n as i32 - 1 {
                let idx = (gx + gy * n as i32) as usize;
                // Accumulate velocity based on fireball movement, RK4 tail lag, and thermal buoyancy
                s.fluid.u_prev[idx] += f_vx * 0.08 + d_x * 0.6;
                s.fluid.v_prev[idx] += f_vy * 0.08 + d_y * 0.6 - 300.0;
            }
        }
    }

    // Add turbulence/curl to make the flame lick and swirl.
    // Increased speed and frequency so the flame churns visibly at 120fps.
    for _ in 0..6 {
        let turb_angle = s.rng.next_f32() * 6.28;
        let turb_speed = 600.0 * (0.5 + 0.5 * (t * 7.0).cos().abs());
        let turb_x = cx + (s.rng.next_f32() - 0.5) * 90.0;
        let turb_y = cy + (s.rng.next_f32() - 0.5) * 90.0;
        let gx = (turb_x / w * n as f32) as i32;
        let gy = (turb_y / h * n as f32) as i32;
        if gx >= 1 && gx < n as i32 - 1 && gy >= 1 && gy < n as i32 - 1 {
            let idx = (gx + gy * n as i32) as usize;
            s.fluid.u_prev[idx] += turb_angle.cos() * turb_speed;
            s.fluid.v_prev[idx] += turb_angle.sin() * turb_speed;
        }
    }

    // Navier-Stokes fluid simulation removed for performance.
    // Particles now use simple velocity-based advection without fluid coupling.

    if rage > 0.0 {
        let force_mag = rage * 50000.0;
        // Cubes removed — only apply forces to card bodies
        for &(id_l, id_r) in &s.physics.card_bodies {
            let fx = (s.rng.next_f32() - 0.5) * force_mag * 0.5;
            let fy = (s.rng.next_f32() - 0.5) * force_mag * 0.5;
            if let Some(body) = s.physics.world.body_mut(id_l) {
                body.apply_force(Vec2::new(fx, fy));
            }
            if let Some(body) = s.physics.world.body_mut(id_r) {
                body.apply_force(Vec2::new(-fx, -fy));
            }
        }
    }

    s.physics.world.step(dt);

    // Spawn new flame particles/embers
    if s.particles.len() < 120 {
        for _ in 0..3 {
            let angle = s.rng.next_f32() * 6.28;
            let speed = 30.0 + s.rng.next_f32() * 70.0;
            // Shift spawning slightly in direction of spring tail
            let spawn_x = cx + d_x * (0.1 + s.rng.next_f32() * 0.4);
            let spawn_y = cy + d_y * (0.1 + s.rng.next_f32() * 0.4);
            s.particles.push(Particle {
                pos: [spawn_x, spawn_y],
                vel: [
                    s.flame_vx * 0.15 + angle.cos() * speed,
                    s.flame_vy * 0.15 + angle.sin() * speed - 30.0,
                ],
                color: [1.0, 0.3 + s.rng.next_f32() * 0.5, 0.0, 1.0],
                life: 0.8 + s.rng.next_f32() * 1.0,
                size: 3.0 + s.rng.next_f32() * 6.0,
                is_ember: s.rng.next_f32() > 0.88,
            });
        }
    }

    // Fast particle update — simple drag + buoyancy, no fluid coupling
    let mut i = s.particles.len();
    while i > 0 {
        i -= 1;
        let p = &mut s.particles[i];
        p.life -= dt;

        // Simple drag + upward buoyancy
        p.vel[0] *= 0.82;
        p.vel[1] = p.vel[1] * 0.82 - 30.0 * dt;

        p.pos[0] += p.vel[0] * dt;
        p.pos[1] += p.vel[1] * dt;
        if p.life <= 0.0 {
            s.particles.swap_remove(i);
        }
    }
}

/// Draw the fireball with directional flame tongues.
///
/// CONTRACT: `s.flame_x/y` must be pre-updated by `update_berserker_simulation` before calling.
/// The flame is shaped as a teardrop/tongue pointing opposite the movement direction,
/// driven by the RK4 spring tail displacement vector (d_x, d_y).
fn draw_berserker_fire(
    r: &mut dyn cvkg_core::Renderer,
    s: &BerserkerState,
    w: f32,
    h: f32,
    t: f32,
) {
    let cx = w * 0.5 + (t * 1.2).cos() * (w * 0.3);
    let cy = h * 0.5 + (t * 0.8).sin() * (h * 0.25);

    // Notify the renderer so glass cards get correct specular highlights.
    r.set_fireball_pos([cx, cy]);

    // RK4 spring tail displacement: points opposite to direction of travel.
    // Larger displacement = faster movement = bigger flame trail.
    let d_x = s.flame_x - cx;
    let d_y = s.flame_y - cy;
    let tail_len = (d_x * d_x + d_y * d_y).sqrt().max(1.0);

    // Normalized tail direction (unit vector pointing "behind" the fireball)
    let tail_nx = d_x / tail_len;
    let tail_ny = d_y / tail_len;

    // Primary flame direction: opposite to movement (tail) + thermal buoyancy (upward)
    let flame_dir_x = tail_nx * 0.7;
    let flame_dir_y = tail_ny * 0.7 - 0.7;
    let flame_len = (flame_dir_x * flame_dir_x + flame_dir_y * flame_dir_y).sqrt().max(0.01);
    let flame_nx = flame_dir_x / flame_len;
    let flame_ny = flame_dir_y / flame_len;


    // High-frequency phase for rapid oscillations visible at 120fps.
    let phase = t * 11.0;

    // --- Outer heat haze glow ---
    // Wide ellipse elongated in the flame direction.
    // Scaled down from 80x130 to ~12x18 for a ~20px fireball.
    let haze_rx = 12.0 + tail_len * 0.05;
    let haze_ry = 18.0 + tail_len * 0.1;
    r.draw_radial_gradient(
        cvkg_core::Rect {
            x: cx + flame_nx * 3.0 - haze_rx,
            y: cy + flame_ny * 3.0 - haze_ry,
            width: haze_rx * 2.0,
            height: haze_ry * 2.0,
        },
        [1.0, 0.40, 0.02, 0.75],
        [0.18, 0.0, 0.0, 0.0],
    );

    // --- Middle flame corona ---
    let corona_rx = 8.0 + tail_len * 0.03;
    let corona_ry = 14.0 + tail_len * 0.06;
    r.draw_radial_gradient(
        cvkg_core::Rect {
            x: cx + flame_nx * 2.0 - corona_rx,
            y: cy + flame_ny * 2.0 - corona_ry,
            width: corona_rx * 2.0,
            height: corona_ry * 2.0,
        },
        [1.0, 0.78, 0.18, 0.95],
        [1.0, 0.30, 0.03, 0.0],
    );


    // Angle of the flame direction vector relative to screen-up (Y-axis).
    // flame_nx/flame_ny is the normalized direction the flame points.
    // atan2(flame_nx, -flame_ny) gives the clockwise rotation from screen-up.
    let flame_angle = flame_nx.atan2(-flame_ny);

    // --- Flame tongues: 7 tongues drawn in local flame-aligned space ---
    // push_transform rotates the canvas so that screen-Y becomes flame_n.
    // Each tongue is then a simple axis-aligned ellipse in local coords with
    // the center at (perp_offset, -reach/2) relative to the fireball pivot.
    // This means tongues naturally lean in the direction of travel.
    struct TongueDef {
        perp_off: f32,   // lateral offset in local X (perpendicular to flame)
        reach:    f32,   // length of the tongue in local Y (along flame)
        half_w:   f32,   // half-width of the tongue base
        phase_off: f32,  // per-tongue phase offset for independent wobble
        color:    [f32; 4],
    }
    let tongues = [
        TongueDef { perp_off:  0.0, reach: 14.0, half_w: 3.5, phase_off: 0.0, color: [1.0, 0.96, 0.85, 0.95] },
        TongueDef { perp_off: -2.5, reach: 12.0, half_w: 2.5, phase_off: 1.3, color: [1.0, 0.85, 0.25, 0.85] },
        TongueDef { perp_off:  2.5, reach: 12.0, half_w: 2.5, phase_off: 2.6, color: [1.0, 0.85, 0.25, 0.85] },
        TongueDef { perp_off: -4.5, reach:  8.5, half_w: 1.8, phase_off: 0.7, color: [1.0, 0.55, 0.08, 0.72] },
        TongueDef { perp_off:  4.5, reach:  8.5, half_w: 1.8, phase_off: 3.9, color: [1.0, 0.55, 0.08, 0.72] },
        TongueDef { perp_off: -6.5, reach:  5.0, half_w: 1.2, phase_off: 2.1, color: [1.0, 0.28, 0.04, 0.52] },
        TongueDef { perp_off:  6.5, reach:  5.0, half_w: 1.2, phase_off: 4.7, color: [1.0, 0.28, 0.04, 0.52] },
    ];

    // --- Ambient flame aura glow behind everything ---
    r.draw_radial_gradient(
        cvkg_core::Rect {
            x: cx - 12.0,
            y: cy - 14.0,
            width: 24.0,
            height: 28.0,
        },
        [1.0, 0.32, 0.0, 0.65],
        [0.8, 0.08, 0.0, 0.0],
    );

    for tongue in &tongues {
        // Independent high-freq wobble per tongue in local X (side lick)
        let wobble = (phase + tongue.phase_off).sin() * tongue.half_w * 0.40
                   + (phase * 1.61 + tongue.phase_off).sin() * tongue.half_w * 0.18;
        // Stretch pulse along Y
        let stretch = 1.0 + (phase * 0.55 + tongue.phase_off).sin() * 0.13;

        // Local X center of this tongue (with wobble), local Y center = -reach/2 (tip toward flame)
        let local_x = tongue.perp_off + wobble;
        let local_y = -(tongue.reach * 0.5);

        // Push a transform: translate to fireball center, rotate to flame angle.
        // draw_radial_gradient will then be drawn in flame-aligned local space.
        r.push_transform([cx, cy], [1.0, 1.0], flame_angle);
        let tongue_rect = cvkg_core::Rect {
            x: local_x - tongue.half_w,
            y: local_y - tongue.reach * 0.5 * stretch,
            width: tongue.half_w * 2.0,
            height: tongue.reach * stretch,
        };
        // Soft glowing flame gradient fading to transparent red/orange
        r.draw_radial_gradient(
            tongue_rect,
            tongue.color,
            [1.0, 0.25, 0.0, 0.0],
        );
        r.pop_transform();
    }

    // --- Trail smear ellipses along the RK4 displacement vector ---
    // These smear backward from the fireball center to show inertia.
    for i in 1..7 {
        let trail_t = i as f32 / 7.0;
        let trail_x = cx + d_x * trail_t;
        let trail_y = cy + d_y * trail_t;
        // Trail narrows and fades towards the tail tip.
        let trail_w = 5.0 * (1.0 - trail_t * 0.7);
        let trail_h = 5.0 * (1.0 - trail_t * 0.7);
        r.draw_radial_gradient(
            cvkg_core::Rect {
                x: trail_x - trail_w * 0.5,
                y: trail_y - trail_h * 0.5,
                width: trail_w,
                height: trail_h,
            },
            [1.0, 0.52, 0.12, 0.75 * (1.0 - trail_t)],
            [0.9, 0.18, 0.0, 0.0],
        );
    }

    // --- Fireball core: hot concentric volumetric gradients ---
    r.draw_radial_gradient(
        cvkg_core::Rect {
            x: cx - 4.0,
            y: cy - 5.0,
            width: 8.0,
            height: 10.0,
        },
        [1.0, 0.88, 0.45, 0.98],
        [1.0, 0.32, 0.0, 0.0],
    );
    r.draw_radial_gradient(
        cvkg_core::Rect {
            x: cx - 2.5,
            y: cy - 3.0,
            width: 5.0,
            height: 6.0,
        },
        [1.0, 1.0, 0.92, 1.0],
        [1.0, 0.65, 0.1, 0.0],
    );

    // --- Particles (embers and sparks) ---
    // Render in two separate loops grouped by shape to allow perfect draw call batching
    // and avoid CPU-heavy text shaping for embers.

    // 1. Draw all standard circular sparks (fill_ellipse)
    for p in &s.particles {
        if p.is_ember {
            continue;
        }
        let heat = ((p.pos[0] + p.pos[1]) * 0.03 + t * 7.0).sin() * 0.5 + 0.5;
        let p_color = if heat > 0.66 {
            [0.55, 0.82, 1.0, p.life.min(1.0)]
        } else if heat > 0.33 {
            [1.0, 0.88, 0.45, p.life.min(1.0)]
        } else {
            [1.0, 0.28, 0.04, p.life.min(1.0)]
        };
        let rect = cvkg_core::Rect {
            x: p.pos[0],
            y: p.pos[1],
            width: p.size,
            height: p.size,
        };
        r.fill_ellipse(rect, p_color);
    }

    // 2. Draw all ember particles as fast-path squares (fill_rect)
    for p in &s.particles {
        if !p.is_ember {
            continue;
        }
        let heat = ((p.pos[0] + p.pos[1]) * 0.03 + t * 7.0).sin() * 0.5 + 0.5;
        let p_color = [1.0, 0.55 + heat * 0.3, 0.10, p.life.min(1.0)];
        let rect = cvkg_core::Rect {
            x: p.pos[0],
            y: p.pos[1],
            width: p.size,
            height: p.size,
        };
        r.fill_rect(rect, p_color);
    }

    // --- Mjolnir lightning bolt: fires every ~20ms ---
    if ((t * 1000.0) as u32).is_multiple_of(20) {
        let angle = (t * 5.0) % 6.28;
        let dist = 45.0;
        r.draw_mjolnir_bolt(
            [cx, cy],
            [cx + angle.cos() * dist, cy + angle.sin() * dist],
            [0.6, 0.9, 1.0, 1.0],
        );
    }
}

fn draw_corner_buttons(
    r: &mut dyn cvkg_core::Renderer,
    counters: &[Signal<u32>; 4],
    rage: &Signal<f32>,
    w: f32,
    h: f32,
) {
    let btn_size = 100.0;
    let padding = 20.0;
    let corners = [
        (padding, padding + 30.0, "I"),
        (w - btn_size - padding, padding + 30.0, "II"),
        (padding, h - btn_size - padding - 70.0, "III"),
        (w - btn_size - padding, h - btn_size - padding - 70.0, "IV"),
    ];

    for (i, corner) in corners.iter().enumerate() {
        let rect = cvkg_core::Rect {
            x: corner.0,
            y: corner.1,
            width: btn_size,
            height: btn_size,
        };
        r.push_vnode(rect, "CornerButton");
        r.fill_rounded_rect(rect, 12.0, [0.2, 0.2, 0.3, 0.8]);
        let (cw, ch) = r.measure_text(corner.2, 32.0);
        let text_x = corner.0 + (btn_size - cw) / 2.0;
        let text_y = corner.1 + (btn_size - ch) / 2.0;
        r.draw_text(corner.2, text_x + 1.0, text_y + 1.0, 32.0, [0.0, 0.0, 0.0, 0.35]);
        r.draw_text(corner.2, text_x, text_y, 32.0, [1.0, 1.0, 1.0, 1.0]);

        let val = counters[i].get();
        let val_str = format!("{}", val);
        let (_vw, vh) = r.measure_text(&val_str, 24.0);
        let value_x = corner.0 + btn_size + 10.0;
        let value_y = corner.1 + (btn_size - vh) / 2.0;
        r.draw_text(&val_str, value_x + 1.0, value_y + 1.0, 24.0, [0.0, 0.0, 0.0, 0.35]);
        r.draw_text(&val_str, value_x, value_y, 24.0, [0.0, 1.0, 0.55, 1.0]);

        let c_signal = counters[i].clone();
        let r_signal = rage.clone();
        let h_closure = Arc::new(move |_| {
            c_signal.set(c_signal.get() + 1);
            r_signal.set((r_signal.get() + 25.0).min(100.0));
            log::info!("Button {} clicked! Total: {}", i, c_signal.get());
        });
        r.register_handler("pointerclick", h_closure);
        r.pop_vnode();
    }
}

/// Search for a demo asset using the shared CVKG asset layout.
fn find_cvkg_asset_path(name: &str) -> Option<PathBuf> {
    let mut candidates = vec![
        PathBuf::from("assets").join(name),
        PathBuf::from("demos/berserker/assets").join(name),
        PathBuf::from("demos/berserker").join(name),
    ];

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("assets").join(name));
        candidates.push(current_dir.join("demos/berserker/assets").join(name));
        candidates.push(current_dir.join("demos/berserker").join(name));
    }

    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        candidates.push(exe_dir.join("assets").join(name));
        candidates.push(exe_dir.join("demos/berserker/assets").join(name));
        candidates.push(exe_dir.join(name));
        if let Some(parent) = exe_dir.parent() {
            candidates.push(parent.join("assets").join(name));
            candidates.push(parent.join("demos/berserker/assets").join(name));
            candidates.push(parent.join("demos/berserker").join(name));
        }
    }

    candidates.into_iter().find(|path| path.exists())
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info,cvkg=debug,berserker=debug")).init();
    log::info!("═══════════════════════════════════════════════════");
    log::info!(
        "  BERSERKER FIRE v{} — Cyberpunk Viking UI Demo",
        env!("CARGO_PKG_VERSION")
    );
    log::info!("  Display: {:?}", DisplayEnvironment::default());
    log::info!(
        "  Performance Contract: {:?}",
        PerformanceContract::chrome_standard()
    );
    log::info!("═══════════════════════════════════════════════════");
    // Load background image
        let bg_image_data = find_cvkg_asset_path("background.jpg")
            .map(fs::read)
            .unwrap_or_else(|| {
                Err(std::io::Error::other(
                    "background.jpg not found in shared CVKG asset paths",
                ))
            })
            .inspect(|data| log::info!("[Berserker] Loaded background image: {} bytes", data.len()))
            .inspect_err(|e| log::warn!("[Berserker] Failed to load background image: {}", e))
            .ok();

        log::info!("[Berserker] CWD: {:?}", std::env::current_dir());

    std::panic::set_hook(Box::new(|info| {
        log::error!("CRITICAL_FAILURE: Application panicked: {}", info);
    }));

    log::info!("Launching with full debug logging enabled...");
    let prewarm_assets = bg_image_data.map(|data| vec![("background".to_string(), data)]);
    cvkg::native::NativeRenderer::run(
        BerserkerFireView::new(1280.0, 720.0),
        prewarm_assets,
    );
}
