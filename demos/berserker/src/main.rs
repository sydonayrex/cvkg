#![allow(dead_code, clippy::approx_constant)]

use cvkg::prelude::*;
use cvkg_core::{ColorTheme, DisplayEnvironment, PerformanceContract};
use cvkg_anim::skeletal::RagdollBlender;
use cvkg_physics::ragdoll_bridge::RagdollBridge;
use cvkg_physics::{BodyId, Collider, Constraint, PhysicsWorld, RigidBody, Shape, WorldConfig};
use cvkg_vdom::signals::Signal;
use glam::Vec2;
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

struct BerserkerState {
    particles: Vec<Particle>,
    rng: Lcg,
    last_time: f32,
    physics: PhysicsState,
    anim: AnimState,
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
        for _ in 0..15 {
            let size = 50.0 + rng.next_f32() * 100.0;
            let shape = Shape::aabb(Vec2::new(size / 2.0, size / 2.0));
            let mut body = RigidBody::new(size * size * 0.01, &shape);
            body.position = Vec2::new(rng.next_f32() * w, rng.next_f32() * h);
            body.velocity = Vec2::new(
                (rng.next_f32() - 0.5) * 100.0,
                (rng.next_f32() - 0.5) * 100.0,
            );
            body.angular_velocity = (rng.next_f32() - 0.5) * 2.0;
            let id = world.add_body(body);
            world.add_collider(Collider::new(id, shape));
            cube_ids.push((id, size));
        }

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
            constraint.break_threshold = Some(15000.0);
            world.add_constraint(constraint);
            world.add_constraint(Constraint::distance(
                ground_id, id_l,
                Vec2::new(pos[0] - w / 2.0 - 100.0, pos[1] - h),
                Vec2::new(0.0, 0.0), 0.0,
            ));
            card_bodies.push((id_l, id_r));
        }

        // Ragdoll Dummy
        let head_shape = Shape::aabb(Vec2::new(20.0, 20.0));
        let mut dummy_head_body = RigidBody::new(5.0, &head_shape);
        dummy_head_body.position = Vec2::new(w * 0.8, h * 0.4);
        let dummy_head = world.add_body(dummy_head_body);
        world.add_collider(Collider::new(dummy_head, head_shape));

        let torso_shape = Shape::aabb(Vec2::new(30.0, 50.0));
        let mut dummy_torso_body = RigidBody::new(15.0, &torso_shape);
        dummy_torso_body.position = Vec2::new(w * 0.8, h * 0.5);
        let dummy_torso = world.add_body(dummy_torso_body);
        world.add_collider(Collider::new(dummy_torso, torso_shape));

        world.add_constraint(Constraint::pin(
            dummy_head, dummy_torso,
            Vec2::new(w * 0.8, h * 0.45),
        ));
        world.add_constraint(Constraint::pin(
            ground_id, dummy_head,
            Vec2::new(w * 0.8, h * 0.4),
        ));

        let mut bridge_config = cvkg_physics::RagdollBridgeConfig::default();
        bridge_config.bone_mappings.push(cvkg_physics::BoneBodyMap {
            bone_index: 0, body_id: dummy_head,
            local_offset: glam::Vec3::ZERO, local_rotation: glam::Quat::IDENTITY,
        });
        bridge_config.bone_mappings.push(cvkg_physics::BoneBodyMap {
            bone_index: 1, body_id: dummy_torso,
            local_offset: glam::Vec3::ZERO, local_rotation: glam::Quat::IDENTITY,
        });
        let bridge = cvkg_physics::RagdollBridge::new(bridge_config);
        let blender = RagdollBlender::new(2);

        log::info!("BerserkerState initialized: {} cubes, {} cards", cube_ids.len(), card_bodies.len());

        Self {
            particles: Vec::new(), rng, last_time: 0.0,
            physics: PhysicsState { world, cube_ids, card_bodies, dummy_head, dummy_torso },
            anim: AnimState { blender, bridge },
        }
    }
}

#[derive(Clone)]
struct BerserkerFireView {
    counters: [Signal<u32>; 4],
    rage: Signal<f32>,
    state: Arc<Mutex<BerserkerState>>,
}

impl BerserkerFireView {
    fn new(w: f32, h: f32) -> Self {
        log::info!("Creating BerserkerFireView ({}x{})", w, h);
        Self {
            counters: [Signal::new(0), Signal::new(0), Signal::new(0), Signal::new(0)],
            rage: Signal::new(0.0),
            state: Arc::new(Mutex::new(BerserkerState::new(w, h))),
        }
    }
}

impl View for BerserkerFireView {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, r: &mut dyn cvkg_core::Renderer, rect: cvkg_core::Rect) {
        let w = rect.width;
        let h = rect.height;

        // Physics step: lock, update, release immediately
        let t = r.elapsed_time();
        let current_rage = self.rage.get();
        {
            let mut s = self.state.lock().expect("Berserker state mutex poisoned");
            if t > s.last_time {
                let dt = (t - s.last_time).min(0.1);
                s.last_time = t;
                update_berserker_simulation(&mut s, w, h, t, dt, current_rage);
            }
        } // Mutex released here

        r.push_vnode(rect, "BerserkerFireView");

        // Drawing: re-lock only for reading state
        let s = self.state.lock().expect("Berserker state mutex poisoned");

        // Draw background
        draw_3d_cubes_bg(r, &s, w, h, t);

        // Glass cards
        draw_glass_cards(r, &s, w, h, t);

        if t > 0.0 {
            draw_berserker_fire(r, &s, w, h, t);
            // Ragdoll needs mutable access for animation blending
            drop(s);
            let mut s = self.state.lock().expect("Berserker state mutex poisoned");
            draw_ragdoll_dummy(r, &mut s, w, h, current_rage);
        } else {
            drop(s);
        }

        // Draw chrome components (no state needed)
        draw_nornir_bar(r, &self.counters, &self.rage, w, h);
        draw_dock(r, &self.counters, &self.rage, w, h);
        draw_corner_buttons(r, &self.counters, &self.rage, w, h);

        r.pop_vnode();
    }
}

fn draw_nornir_bar(r: &mut dyn cvkg_core::Renderer, _counters: &[Signal<u32>; 4], _rage: &Signal<f32>, w: f32, _h: f32) {
    let bar_rect = cvkg_core::Rect { x: 0.0, y: 0.0, width: w, height: 28.0 };
    r.push_vnode(bar_rect, "NornirBar");
    // Glass-like background: dark translucent fill (bifrost requires multi-pass pipeline)
    r.fill_rounded_rect(bar_rect, 0.0, [0.02, 0.02, 0.04, 0.75]);

    let menu_x = 8.0;
    let items = [("File", 60.0), ("Edit", 60.0), ("View", 70.0), ("Window", 80.0), ("Help", 60.0)];
    let mut x = menu_x;
    for (label, width) in items {
        r.draw_text(label, x, 8.0, 13.0, [0.9, 0.9, 0.92, 1.0]);
        x += width;
    }

    r.draw_text("BERSERKER v2.0", w * 0.5 - 60.0, 8.0, 14.0, [1.0, 0.3, 0.1, 1.0]);
    r.draw_text(&format!("Rage: {:.0}%", _rage.get()), w - 120.0, 8.0, 12.0, [0.0, 1.0, 0.5, 1.0]);

    r.pop_vnode();
}

fn draw_dock(r: &mut dyn cvkg_core::Renderer, _counters: &[Signal<u32>; 4], _rage: &Signal<f32>, w: f32, h: f32) {
    let dock_rect = cvkg_core::Rect { x: w * 0.3, y: h - 68.0, width: w * 0.4, height: 56.0 };
    r.push_vnode(dock_rect, "HeimdallDock");
    // Glass-like background: dark translucent fill (bifrost requires multi-pass pipeline)
    r.fill_rounded_rect(dock_rect, 16.0, [0.04, 0.04, 0.06, 0.85]);

    let icons = ["⚔️", "🔥", "🛡️", "💀", "🌋"];
    let icon_size = 48.0;
    let start_x = dock_rect.x + 12.0;
    let center_y = dock_rect.y + dock_rect.height / 2.0;

    for (i, icon) in icons.iter().enumerate() {
        let ix = start_x + i as f32 * (icon_size + 8.0);
        r.draw_text(icon, ix + 8.0, center_y - 8.0, 24.0, [0.9, 0.9, 0.92, 0.9]);

        if i < 3 {
            let dot_rect = cvkg_core::Rect { x: ix + icon_size / 2.0 - 2.0, y: center_y + icon_size / 2.0 + 4.0, width: 4.0, height: 4.0 };
            let accent = [0.0, 1.0, 0.95, 1.0];
            r.fill_ellipse(dot_rect, accent);
        }
    }

    r.pop_vnode();
}

fn draw_3d_cubes_bg(r: &mut dyn cvkg_core::Renderer, s: &BerserkerState, w: f32, h: f32, _t: f32) {
    r.fill_rect(cvkg_core::Rect { x: 0.0, y: 28.0, width: w, height: h - 96.0 }, [0.01, 0.01, 0.03, 1.0]);

    for &(id, size) in &s.physics.cube_ids {
        if let Some(body) = s.physics.world.body(id) {
            let rect = cvkg_core::Rect {
                x: body.position.x - size * 0.5,
                y: body.position.y - size * 0.5,
                width: size, height: size,
            };
            let rot = [body.angle, body.angle * 0.5, body.angle * 0.2];
            r.draw_3d_cube(rect, [0.1, 0.6, 0.9, 0.6], rot);
        }
    }
}

fn draw_glass_cards(r: &mut dyn cvkg_core::Renderer, s: &BerserkerState, _w: f32, _h: f32, _t: f32) {
    let runes = ["CVK!!!", "CVK!!!", "CVK!!!"];

    for (i, &(id_l, id_r)) in s.physics.card_bodies.iter().enumerate() {
        if let (Some(bl), Some(br)) = (s.physics.world.body(id_l), s.physics.world.body(id_r)) {
            let rect_l = cvkg_core::Rect { x: bl.position.x - 100.0, y: bl.position.y - 125.0, width: 200.0, height: 250.0 };
            let rect_r = cvkg_core::Rect { x: br.position.x - 100.0, y: br.position.y - 125.0, width: 200.0, height: 250.0 };

            r.push_vnode(rect_l, "CardLeft");
            r.fill_rounded_rect(rect_l, 12.0, [0.05, 0.05, 0.1, 0.4]);
            r.pop_vnode();

            r.push_vnode(rect_r, "CardRight");
            r.fill_rounded_rect(rect_r, 12.0, [0.05, 0.05, 0.1, 0.4]);

            let cx = (bl.position.x + br.position.x) / 2.0;
            let cy = (bl.position.y + br.position.y) / 2.0;
            r.draw_text(runes[i % runes.len()], cx - 50.0, cy, 32.0, [0.8, 0.9, 1.0, 1.0]);
            r.pop_vnode();
        }
    }
}

fn draw_ragdoll_dummy(r: &mut dyn cvkg_core::Renderer, s: &mut BerserkerState, _w: f32, _h: f32, rage: f32) {
    s.anim.bridge.update(&s.physics.world);
    let transforms = s.anim.bridge.physics_transforms().to_vec();
    s.anim.blender.set_physics(&transforms);
    let blend_weight = (rage / 5.0).clamp(0.0, 1.0);
    s.anim.blender.blend(blend_weight);
    let poses = s.anim.blender.update(0.016);

    let head_pos = poses[0].0;
    r.fill_rounded_rect(cvkg_core::Rect { x: head_pos.x - 20.0, y: head_pos.y - 20.0, width: 40.0, height: 40.0 }, 8.0, [0.9, 0.2, 0.2, 1.0]);

    let torso_pos = poses[1].0;
    r.fill_rounded_rect(cvkg_core::Rect { x: torso_pos.x - 30.0, y: torso_pos.y - 50.0, width: 60.0, height: 100.0 }, 12.0, [0.8, 0.4, 0.1, 1.0]);
}

fn update_berserker_simulation(s: &mut BerserkerState, w: f32, h: f32, t: f32, dt: f32, rage: f32) {
    let cx = w * 0.5 + (t * 1.2).cos() * (w * 0.3);
    let cy = h * 0.5 + (t * 0.8).sin() * (h * 0.25);

    if rage > 0.0 {
        let force_mag = rage * 50000.0;
        for &(id, _) in &s.physics.cube_ids {
            let fx = (s.rng.next_f32() - 0.5) * force_mag;
            let fy = (s.rng.next_f32() - 0.5) * force_mag;
            if let Some(body) = s.physics.world.body_mut(id) {
                body.apply_force(Vec2::new(fx, fy));
            }
        }
        for &(id_l, id_r) in &s.physics.card_bodies {
            let fx = (s.rng.next_f32() - 0.5) * force_mag * 0.5;
            let fy = (s.rng.next_f32() - 0.5) * force_mag * 0.5;
            if let Some(body) = s.physics.world.body_mut(id_l) { body.apply_force(Vec2::new(fx, fy)); }
            if let Some(body) = s.physics.world.body_mut(id_r) { body.apply_force(Vec2::new(-fx, -fy)); }
        }
    }

    s.physics.world.step(dt);

    // Cap particle count to prevent unbounded growth (max 80 particles)
    if s.particles.len() < 80 {
        for _ in 0..3 {
            let angle = s.rng.next_f32() * 6.28;
            let speed = 50.0 + s.rng.next_f32() * 100.0;
            s.particles.push(Particle {
                pos: [cx, cy],
                vel: [angle.cos() * speed, angle.sin() * speed - 50.0],
                color: [1.0, 0.3 + s.rng.next_f32() * 0.5, 0.0, 1.0],
                life: 0.5 + s.rng.next_f32() * 1.0,
                size: 2.0 + s.rng.next_f32() * 4.0,
                is_ember: s.rng.next_f32() > 0.9,
            });
        }
    }

    // Fast particle update: inline, no retain_mut (swap_remove is faster)
    let mut i = s.particles.len();
    while i > 0 {
        i -= 1;
        let p = &mut s.particles[i];
        p.life -= dt;
        p.pos[0] += p.vel[0] * dt;
        p.pos[1] += p.vel[1] * dt;
        if p.life <= 0.0 {
            s.particles.swap_remove(i);
        }
    }
}

fn draw_berserker_fire(r: &mut dyn cvkg_core::Renderer, s: &BerserkerState, w: f32, h: f32, t: f32) {
    let cx = w * 0.5 + (t * 1.2).cos() * (w * 0.3);
    let cy = h * 0.5 + (t * 0.8).sin() * (h * 0.25);

    r.draw_radial_gradient(cvkg_core::Rect { x: cx - 100.0, y: cy - 100.0, width: 200.0, height: 200.0 }, [1.0, 0.4, 0.0, 0.6], [0.2, 0.0, 0.0, 0.0]);
    r.draw_radial_gradient(cvkg_core::Rect { x: cx - 60.0, y: cy - 60.0, width: 120.0, height: 120.0 }, [1.0, 0.8, 0.2, 0.8], [1.0, 0.2, 0.0, 0.0]);
    r.draw_radial_gradient(cvkg_core::Rect { x: cx - 30.0, y: cy - 30.0, width: 60.0, height: 60.0 }, [1.0, 1.0, 0.8, 1.0], [1.0, 0.5, 0.0, 0.0]);

    for p in &s.particles {
        let p_color = [p.color[0], p.color[1], p.color[2], p.life.min(1.0)];
        let rect = cvkg_core::Rect { x: p.pos[0], y: p.pos[1], width: p.size, height: p.size };
        if p.is_ember { r.draw_text("*", p.pos[0], p.pos[1], (p.size * 2.0).round(), p_color); }
        else { r.fill_ellipse(rect, p_color); }
    }

    if (t * 1000.0) as u32 % 20 == 0 {
        let angle = (t * 5.0) % 6.28;
        let dist = 100.0 + 200.0;
        r.draw_mjolnir_bolt([cx, cy], [cx + angle.cos() * dist, cy + angle.sin() * dist], [0.6, 0.9, 1.0, 1.0]);
    }
}

fn draw_corner_buttons(r: &mut dyn cvkg_core::Renderer, counters: &[Signal<u32>; 4], rage: &Signal<f32>, w: f32, h: f32) {
    let btn_size = 100.0;
    let padding = 20.0;
    let corners = [
        (padding, padding + 30.0, "I"),
        (w - btn_size - padding, padding + 30.0, "II"),
        (padding, h - btn_size - padding - 70.0, "III"),
        (w - btn_size - padding, h - btn_size - padding - 70.0, "IV"),
    ];

    for (i, corner) in corners.iter().enumerate() {
        let rect = cvkg_core::Rect { x: corner.0, y: corner.1, width: btn_size, height: btn_size };
        r.push_vnode(rect, "CornerButton");
        r.fill_rounded_rect(rect, 12.0, [0.2, 0.2, 0.3, 0.8]);
        r.draw_text(corner.2, corner.0 + 35.0, corner.1 + 60.0, 32.0, [1.0, 1.0, 1.0, 1.0]);

        let val = counters[i].get();
        r.draw_text(&format!("{}", val), corner.0 + btn_size + 10.0, corner.1 + 60.0, 24.0, [0.0, 1.0, 0.5, 1.0]);

        let c_signal = counters[i].clone();
        let r_signal = rage.clone();
        let h_closure = Arc::new(move |_| {
            c_signal.set(c_signal.get() + 1);
            r_signal.set(r_signal.get() + 1.0);
            log::info!("Button {} clicked! Total: {}", i, c_signal.get());
        });
        r.register_handler("pointerdown", h_closure.clone());
        r.register_handler("pointerclick", h_closure);
        r.pop_vnode();
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    log::info!("═══════════════════════════════════════════════════");
    log::info!("  BERSERKER FIRE v2.0 — Cyberpunk Viking UI Demo");
    log::info!("  Display: {:?}", DisplayEnvironment::default());
    log::info!("  Performance Contract: {:?}", PerformanceContract::chrome_standard());
    log::info!("═══════════════════════════════════════════════════");

    std::panic::set_hook(Box::new(|info| {
        log::error!("CRITICAL_FAILURE: Application panicked: {}", info);
    }));

    log::info!("Launching with full debug logging enabled...");
    cvkg::native::NativeRenderer::run(BerserkerFireView::new(1280.0, 720.0));
}
