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

struct BerserkerState {
    particles: Vec<Particle>,
    rng: Lcg,
    last_time: f32,
    physics: PhysicsState,
    anim: AnimState,
    loaded_svgs: bool,
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
                ground_id,
                id_l,
                Vec2::new(pos[0] - w / 2.0 - 100.0, pos[1] - h),
                Vec2::new(0.0, 0.0),
                0.0,
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
            dummy_head,
            dummy_torso,
            Vec2::new(w * 0.8, h * 0.45),
        ));
        world.add_constraint(Constraint::pin(
            ground_id,
            dummy_head,
            Vec2::new(w * 0.8, h * 0.4),
        ));

        let mut bridge_config = cvkg_physics::RagdollBridgeConfig::default();
        bridge_config.bone_mappings.push(cvkg_physics::BoneBodyMap {
            bone_index: 0,
            body_id: dummy_head,
            local_offset: glam::Vec3::ZERO,
            local_rotation: glam::Quat::IDENTITY,
        });
        bridge_config.bone_mappings.push(cvkg_physics::BoneBodyMap {
            bone_index: 1,
            body_id: dummy_torso,
            local_offset: glam::Vec3::ZERO,
            local_rotation: glam::Quat::IDENTITY,
        });
        let bridge = cvkg_physics::RagdollBridge::new(bridge_config);
        let blender = RagdollBlender::new(2);

        log::info!(
            "BerserkerState initialized: {} cubes, {} cards",
            cube_ids.len(),
            card_bodies.len()
        );

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

        // Draw background image (prewarmed on first frame via NativeRenderer::run assets)
        r.draw_image("background", cvkg_core::Rect { x: 0.0, y: 0.0, width: w, height: h });

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
        if dt > 0.0 {
            update_berserker_simulation(&mut s, w, h, t, dt, new_rage);
        }

        // Draw the 3D rotating cubes in the background
        let t_cubes_start = std::time::Instant::now();
        draw_3d_cubes_bg(r, &s, w, h, t);
        let t_cubes = t_cubes_start.elapsed().as_secs_f32() * 1000.0;

        // Draw the glass cards
        let t_cards_start = std::time::Instant::now();
        draw_glass_cards(r, &s, w, h, t);
        let t_cards = t_cards_start.elapsed().as_secs_f32() * 1000.0;

        // Draw the valknut symbol with procedural fuse animation
        let vk_cx = w / 2.0;
        let vk_cy = h / 2.0 - 100.0;
        let vk_size = 120.0;
        draw_valknut(r, vk_cx, vk_cy, vk_size, t);

        let t_fire_start = std::time::Instant::now();
        if t > 0.0 {
            // Clip fire to content area (safe area top is now 0 in content coordinates)
            r.push_clip_rect(cvkg_core::Rect { x: 0.0, y: 0.0, width: w, height: h });
            // Draw particles and Mjolnir lightning bolts
            draw_berserker_fire(r, &s, w, h, t);
            // Draw skeletal/ragdoll elements
            draw_ragdoll_dummy(r, &mut s, w, h, new_rage);
            r.pop_clip_rect();
        }
        let t_fire = t_fire_start.elapsed().as_secs_f32() * 1000.0;

        // Draw chrome components (no state needed)
        let t_chrome_start = std::time::Instant::now();
        draw_nornir_bar(
            r,
            &self.counters,
            &self.rage,
            &self.active_menu,
            &self.perf,
            w,
            h,
        );
        draw_dock(r, &self.counters, &self.rage, w, h);
        draw_corner_buttons(r, &self.counters, &self.rage, w, h);
        let t_chrome = t_chrome_start.elapsed().as_secs_f32() * 1000.0;

        if (s.last_time as u32).is_multiple_of(5) {
            log::info!(
                "[Berserker] Draw timings: cubes={:.2}ms cards={:.2}ms fire={:.2}ms chrome={:.2}ms",
                t_cubes,
                t_cards,
                t_fire,
                t_chrome
            );
        }

        // Draw the performance overlay
        {
            let perf = self.perf.lock().expect("Failed to lock PerfOverlay");
            perf.render(r, rect);
        }
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
        // The draw_text y parameter is the text origin (top of cell).
        // Glyphs are baseline-relative, with the baseline roughly 75% down the cell.
        // To visually center: shift up so the baseline lands at bar_height/2 + descent/2.
        // Approximation: y = (bar_height - lh) / 2 - lh * 0.5
        let ty = (28.0 - lh) * 0.5 - lh * 0.5;
        r.draw_text(label, tx + 1.0, ty + 1.0, 13.0, [0.0, 0.0, 0.0, 0.45]);
        r.draw_text(label, tx, ty, 13.0, [0.9, 0.9, 0.92, 1.0]);
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
        r.register_handler("pointerdown", h_closure.clone());
        r.register_handler("pointerclick", h_closure);
        x += width;
    }

    // Centered window title
    let title_str = format!("BERSERKER v{}", env!("CARGO_PKG_VERSION"));
    let (tw, tlh) = r.measure_text(&title_str, 14.0);
    let title_x = (w - tw) / 2.0;
    let title_y = (28.0 - tlh) * 0.5 - tlh * 0.5;
    r.draw_text(&title_str, title_x + 1.0, title_y + 1.0, 14.0, [0.0, 0.0, 0.0, 0.45]);
    r.draw_text(&title_str, title_x, title_y, 14.0, [1.0, 0.3, 0.1, 1.0]);

    // Right-aligned, vertically centered rage meter
    let rage_str = format!("Rage: {:.0}%", _rage.get());
    let (rw, rlh) = r.measure_text(&rage_str, 12.0);
    let rage_x = w - rw - 16.0;
    let rage_y = (28.0 - rlh) * 0.5 - rlh * 0.5;
    r.draw_text(&rage_str, rage_x + 1.0, rage_y + 1.0, 12.0, [0.0, 0.0, 0.0, 0.45]);
    r.draw_text(&rage_str, rage_x, rage_y, 12.0, [0.0, 1.0, 0.5, 1.0]);

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
        let ty = dock_rect.y + (dock_rect.height - th) / 2.0 - th * 0.25;
        r.draw_text(icon, tx + 1.0, ty + 1.0, text_size, [0.0, 0.0, 0.0, 0.45]);
        r.draw_text(icon, tx, ty, text_size, [0.95, 0.95, 0.98, 1.0]);

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
        r.register_handler("pointerdown", h_closure.clone());
        r.register_handler("pointerclick", h_closure);
        r.pop_vnode();
    }
}

fn draw_3d_cubes_bg(r: &mut dyn cvkg_core::Renderer, s: &BerserkerState, _w: f32, _h: f32, _t: f32) {
    // Background is now drawn by the main render() function via draw_image

    for &(id, size) in &s.physics.cube_ids {
        if let Some(body) = s.physics.world.body(id) {
            let rect = cvkg_core::Rect {
                x: body.position.x - size * 0.5,
                y: body.position.y - size * 0.5,
                width: size,
                height: size,
            };
            let rot = [body.angle, body.angle * 0.5, body.angle * 0.2];
            r.draw_3d_cube(rect, [0.1, 0.6, 0.9, 0.6], rot);
        }
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
                    [0.0, 0.0, 0.0, 0.45],
                );
                r.draw_text(
                    runes[i % runes.len()],
                    cx - rw / 2.0,
                    cy - rh / 2.0,
                    32.0,
                    [0.8, 0.9, 1.0, 1.0],
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
            if let Some(body) = s.physics.world.body_mut(id_l) {
                body.apply_force(Vec2::new(fx, fy));
            }
            if let Some(body) = s.physics.world.body_mut(id_r) {
                body.apply_force(Vec2::new(-fx, -fy));
            }
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

fn draw_berserker_fire(
    r: &mut dyn cvkg_core::Renderer,
    s: &BerserkerState,
    w: f32,
    h: f32,
    t: f32,
) {
    let cx = w * 0.5 + (t * 1.2).cos() * (w * 0.3);
    let cy = h * 0.5 + (t * 0.8).sin() * (h * 0.25);

    // Update fireball position for glass specular highlights
    r.set_fireball_pos([cx, cy]);

    r.draw_radial_gradient(
        cvkg_core::Rect {
            x: cx - 100.0,
            y: cy - 100.0,
            width: 200.0,
            height: 200.0,
        },
        [1.0, 0.55, 0.05, 0.85],
        [0.15, 0.0, 0.0, 0.0],
    );
    r.draw_radial_gradient(
        cvkg_core::Rect {
            x: cx - 60.0,
            y: cy - 60.0,
            width: 120.0,
            height: 120.0,
        },
        [1.0, 0.85, 0.3, 0.95],
        [1.0, 0.35, 0.05, 0.0],
    );
    r.draw_radial_gradient(
        cvkg_core::Rect {
            x: cx - 30.0,
            y: cy - 30.0,
            width: 60.0,
            height: 60.0,
        },
        [1.0, 1.0, 0.95, 1.0],
        [1.0, 0.75, 0.15, 0.0],
    );
    let phase = t * 4.0;
    let drift_x = (t * 1.2).cos() * 120.0;
    let drift_y = (t * 0.8).sin() * 90.0;
    let flame_tongues = [
        (0.0_f32, 34.0_f32, [0.55, -0.15], [1.0, 1.0, 1.0, 0.60]),
        (1.4, 30.0, [-0.35, 0.25], [0.65, 0.95, 1.0, 0.50]),
        (2.7, 38.0, [0.25, 0.55], [1.0, 0.60, 0.12, 0.48]),
        (4.1, 32.0, [-0.55, 0.10], [1.0, 0.25, 0.08, 0.40]),
        (5.2, 28.0, [0.10, 0.75], [0.60, 0.82, 1.0, 0.34]),
    ];
    for (offset, radius, drift, color) in flame_tongues {
        let wobble = (phase + offset).sin();
        let stretch = 1.0 + (phase * 0.8 + offset).cos() * 0.18;
        let flame_x = cx + drift[0] * 8.0 + wobble * 6.0;
        let flame_y = cy + drift[1] * 10.0 - radius * 0.35;
        r.fill_ellipse(
            cvkg_core::Rect {
                x: flame_x - radius * 0.5 * stretch,
                y: flame_y - radius * 0.8,
                width: radius * stretch,
                height: radius * 1.6,
            },
            color,
        );
    }
    for i in 1..6 {
        let trail_t = i as f32 / 6.0;
        let trail_x = cx - drift_x * 0.02 * trail_t;
        let trail_y = cy - drift_y * 0.02 * trail_t + trail_t * 18.0;
        let trail_w = 26.0 + trail_t * 12.0;
        let trail_h = 12.0 + trail_t * 28.0;
        let alpha = (0.25 * (1.0 - trail_t)).max(0.0);
        r.fill_ellipse(
            cvkg_core::Rect {
                x: trail_x - trail_w * 0.5,
                y: trail_y - trail_h * 0.5,
                width: trail_w,
                height: trail_h,
            },
            [1.0, 0.45 + trail_t * 0.3, 0.08, alpha],
        );
    }
    r.fill_ellipse(
        cvkg_core::Rect {
            x: cx - 18.0,
            y: cy - 18.0,
            width: 36.0,
            height: 36.0,
        },
        [0.85, 0.98, 1.0, 0.95],
    );
    r.fill_ellipse(
        cvkg_core::Rect {
            x: cx - 14.0,
            y: cy - 14.0,
            width: 28.0,
            height: 28.0,
        },
        [1.0, 0.72, 0.18, 0.92],
    );
    r.fill_ellipse(
        cvkg_core::Rect {
            x: cx - 9.0,
            y: cy - 9.0,
            width: 18.0,
            height: 18.0,
        },
        [0.95, 0.18, 0.05, 0.82],
    );

    for p in &s.particles {
        let heat = ((p.pos[0] + p.pos[1]) * 0.03 + t * 3.5).sin() * 0.5 + 0.5;
        let p_color = if p.is_ember {
            [1.0, 0.50 + heat * 0.3, 0.08, p.life.min(1.0)]
        } else if heat > 0.66 {
            [0.55, 0.82, 1.0, p.life.min(1.0)]
        } else if heat > 0.33 {
            [1.0, 0.88, 0.45, p.life.min(1.0)]
        } else {
            [1.0, 0.30, 0.05, p.life.min(1.0)]
        };
        let rect = cvkg_core::Rect {
            x: p.pos[0],
            y: p.pos[1],
            width: p.size,
            height: p.size,
        };
        if p.is_ember {
            r.draw_text("*", p.pos[0], p.pos[1], (p.size * 2.0).round(), p_color);
        } else {
            r.fill_ellipse(rect, p_color);
        }
    }

    if ((t * 1000.0) as u32).is_multiple_of(20) {
        let angle = (t * 5.0) % 6.28;
        let dist = 100.0 + 200.0;
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
        let text_y = corner.1 + (btn_size - ch) / 2.0 - ch * 0.25;
        r.draw_text(corner.2, text_x + 1.0, text_y + 1.0, 32.0, [0.0, 0.0, 0.0, 0.45]);
        r.draw_text(corner.2, text_x, text_y, 32.0, [1.0, 1.0, 1.0, 1.0]);

        let val = counters[i].get();
        let val_str = format!("{}", val);
        let (_vw, vh) = r.measure_text(&val_str, 24.0);
        let value_x = corner.0 + btn_size + 10.0;
        let value_y = corner.1 + (btn_size - vh) / 2.0 - vh * 0.25;
        r.draw_text(&val_str, value_x + 1.0, value_y + 1.0, 24.0, [0.0, 0.0, 0.0, 0.45]);
        r.draw_text(&val_str, value_x, value_y, 24.0, [0.0, 1.0, 0.5, 1.0]);

        let c_signal = counters[i].clone();
        let r_signal = rage.clone();
        let h_closure = Arc::new(move |_| {
            c_signal.set(c_signal.get() + 1);
            r_signal.set((r_signal.get() + 25.0).min(100.0));
            log::info!("Button {} clicked! Total: {}", i, c_signal.get());
        });
        r.register_handler("pointerdown", h_closure.clone());
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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
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
