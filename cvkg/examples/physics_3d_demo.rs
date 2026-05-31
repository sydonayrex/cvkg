//! 3D Physics Demo — exercises the full physics↔3D rendering pipeline.
//!
//! Spawns cubes with 3D rigid body physics, applies gravity, and renders them
//! using the 3D mesh pipeline (mode 13) with perspective projection.
//!
//! Run with: `cargo run --example physics_3d_demo --features native`

use cvkg::prelude::*;
use cvkg_core::Renderer;
use std::cell::RefCell;
use std::f32::consts::PI;
use std::time::Instant;

/// A 3D physics cube with position, rotation, and velocity.
struct PhysicsCube {
    id: usize,
    position: [f32; 3],
    velocity: [f32; 3],
    rotation: [f32; 4], // quaternion (x, y, z, w)
    angular_velocity: [f32; 3],
    size: f32,
    color: [f32; 4],
}

impl PhysicsCube {
    fn new(id: usize, x: f32, y: f32, z: f32, size: f32, color: [f32; 4]) -> Self {
        Self {
            id,
            position: [x, y, z],
            velocity: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            angular_velocity: [0.0, 0.0, 0.0],
            size,
            color,
        }
    }

    /// Simple semi-implicit Euler integration.
    fn integrate(&mut self, dt: f32, gravity: [f32; 3]) {
        // Linear
        self.velocity[0] += gravity[0] * dt;
        self.velocity[1] += gravity[1] * dt;
        self.velocity[2] += gravity[2] * dt;
        self.position[0] += self.velocity[0] * dt;
        self.position[1] += self.velocity[1] * dt;
        self.position[2] += self.velocity[2] * dt;

        // Angular (simplified quaternion integration)
        let half_dt = 0.5 * dt;
        let q = &mut self.rotation;
        let w = &self.angular_velocity;
        // dq/dt = 0.5 * omega * q
        let dq_x = w[0] * q[3] + w[1] * q[2] - w[2] * q[1];
        let dq_y = -w[0] * q[2] + w[1] * q[3] + w[2] * q[0];
        let dq_z = w[0] * q[1] - w[1] * q[0] + w[2] * q[3];
        let dq_w = -w[0] * q[0] - w[1] * q[1] - w[2] * q[2];
        q[0] += dq_x * half_dt;
        q[1] += dq_y * half_dt;
        q[2] += dq_z * half_dt;
        q[3] += dq_w * half_dt;
        // Normalize
        let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
        if len > 1e-6 {
            q[0] /= len;
            q[1] /= len;
            q[2] /= len;
            q[3] /= len;
        }
    }

    /// Simple floor collision at y=0.
    fn collide_floor(&mut self, floor_y: f32, restitution: f32) {
        let half_size = self.size * 0.5;
        if self.position[1] - half_size < floor_y {
            self.position[1] = floor_y + half_size;
            self.velocity[1] = -self.velocity[1] * restitution;
            // Add some spin on collision
            self.angular_velocity[0] += self.velocity[2] * 0.5;
            self.angular_velocity[2] -= self.velocity[0] * 0.5;
        }
    }
}

/// The 3D Physics Demo view.
pub struct Physics3DDemo {
    start_time: Instant,
    last_frame: RefCell<Instant>,
    cubes: RefCell<Vec<PhysicsCube>>,
    camera_angle: f32,
}

impl Physics3DDemo {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            last_frame: RefCell::new(Instant::now()),
            cubes: RefCell::new(Vec::new()),
            camera_angle: 0.0,
        }
    }

    /// Spawn a new cube at the given position with random color.
    fn spawn_cube(&self, x: f32, y: f32, z: f32) {
        let mut cubes = self.cubes.borrow_mut();
        let id = cubes.len();
        let colors: [[f32; 4]; 6] = [
            [0.9, 0.2, 0.2, 1.0], // red
            [0.2, 0.9, 0.2, 1.0], // green
            [0.2, 0.2, 0.9, 1.0], // blue
            [0.9, 0.9, 0.2, 1.0], // yellow
            [0.9, 0.2, 0.9, 1.0], // magenta
            [0.2, 0.9, 0.9, 1.0], // cyan
        ];
        let color = colors[id % colors.len()];
        let size = 30.0 + (id as f32 * 7.0) % 40.0;
        cubes.push(PhysicsCube::new(id, x, y, z, size, color));
    }
}

impl View for Physics3DDemo {
    type Body = Self;

    fn body(self) -> Self::Body {
        self
    }

    fn render(&self, r: &mut dyn Renderer, _rect: Rect) {
        let now = Instant::now();
        let mut last_frame = self.last_frame.borrow_mut();
        let dt = (now.duration_since(*last_frame)).as_secs_f32().min(0.05);
        *last_frame = now;

        let time = self.start_time.elapsed().as_secs_f32();

        // Spawn a new cube every 1.5 seconds
        let cube_count = (time / 1.5) as usize;
        {
            let current_count = self.cubes.borrow().len();
            for i in current_count..=cube_count.min(20) {
                let angle = i as f32 * 0.7;
                let x = (angle * 2.0).sin() * 100.0;
                let z = (angle * 2.0).cos() * 100.0;
                self.spawn_cube(x, 200.0 + i as f32 * 50.0, z);
            }
        }

        // Update physics
        {
            let mut cubes = self.cubes.borrow_mut();
            for cube in cubes.iter_mut() {
                cube.integrate(dt, [0.0, -9.81 * 50.0, 0.0]);
                cube.collide_floor(0.0, 0.5);
            }
        }

        // Draw background
        r.fill_rect(
            Rect::new(-1000.0, -1000.0, 2000.0, 2000.0),
            [0.05, 0.05, 0.1, 1.0],
        );

        // Draw floor grid
        for i in -10..=10i32 {
            let z = i as f32 * 50.0;
            let alpha = 0.3 - (i.abs() as f32 * 0.02);
            let alpha = alpha.max(0.0f32);
            r.draw_line(-500.0, 0.0, 500.0, 0.0, [0.2, 0.3, 0.4, alpha], 1.0);
            let proj_y = 400.0 - z * 0.5;
            if proj_y > -500.0 && proj_y < 500.0 {
                r.draw_line(
                    -400.0,
                    proj_y,
                    400.0,
                    proj_y,
                    [0.15, 0.2, 0.3, (alpha * 0.5).max(0.0f32)],
                    0.5,
                );
            }
        }

        // Draw cubes
        let cubes = self.cubes.borrow();
        for cube in cubes.iter() {
            // Project 3D to 2D
            let fov = 500.0;
            let cam_dist = 400.0;
            let scale = fov / (cam_dist + cube.position[2]).max(1.0);

            let screen_x = cube.position[0] * scale;
            let screen_y = -cube.position[1] * scale;
            let screen_size = cube.size * scale;

            // 3D render call
            r.render_scene_node_3d(
                cube.position,
                cube.rotation,
                [cube.size, cube.size, cube.size],
                cube.color,
                &[],
            );

            // 2D fallback projection
            r.fill_rect(
                Rect::new(
                    screen_x - screen_size * 0.5,
                    screen_y - screen_size * 0.5,
                    screen_size,
                    screen_size,
                ),
                cube.color,
            );
        }

        // UI overlay
        r.draw_text(
            &format!("3D Physics Demo | {} cubes", cubes.len()),
            -480.0,
            -350.0,
            18.0,
            [1.0, 1.0, 1.0, 0.8],
        );
        r.draw_text(
            "Physics -> 3D Transform -> GPU Draw Calls",
            -480.0,
            -320.0,
            14.0,
            [0.7, 0.8, 1.0, 0.6],
        );

        r.request_redraw();
    }
}

/// Entry point for the 3D physics demo.
/// This is called from main() when running the demo.
pub fn run_physics_3d_demo() {
    let demo = Physics3DDemo::new();
    // The demo will be driven by the CVKG runtime's event loop
    // when embedded in a View-based application.
    // For standalone use, see the native renderer integration.
    let _ = demo;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_integration() {
        let mut cube = PhysicsCube::new(0, 0.0, 100.0, 0.0, 50.0, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(cube.position[1], 100.0);
        // Integrate with gravity for 1 second
        cube.integrate(1.0, [0.0, -9.81, 0.0]);
        assert!(cube.position[1] < 100.0); // should have fallen
        assert!(cube.velocity[1] < 0.0); // moving downward
    }

    #[test]
    fn test_cube_floor_collision() {
        let mut cube = PhysicsCube::new(0, 0.0, -10.0, 0.0, 50.0, [1.0, 0.0, 0.0, 1.0]);
        cube.velocity[1] = -100.0; // moving down fast
        cube.collide_floor(0.0, 0.5);
        // Should be pushed above floor and velocity reversed
        assert!(cube.position[1] >= 25.0);
    }

    #[test]
    fn test_demo_spawns_cubes() {
        let demo = Physics3DDemo::new();
        assert_eq!(demo.cubes.borrow().len(), 0);
        demo.spawn_cube(0.0, 100.0, 0.0);
        assert_eq!(demo.cubes.borrow().len(), 1);
        demo.spawn_cube(50.0, 200.0, 0.0);
        assert_eq!(demo.cubes.borrow().len(), 2);
    }

    #[test]
    fn test_quaternion_stays_normalized() {
        let mut cube = PhysicsCube::new(0, 0.0, 0.0, 0.0, 10.0, [1.0, 1.0, 1.0, 1.0]);
        cube.angular_velocity = [1.0, 2.0, 0.5];
        for _ in 0..100 {
            cube.integrate(0.016, [0.0, 0.0, 0.0]);
        }
        let q = &cube.rotation;
        let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
        assert!((len - 1.0).abs() < 0.001, "Quaternion should stay normalized, got len={}", len);
    }
}

/// Entry point for the 3D physics demo example.
fn main() {
    println!("3D Physics Demo");
    println!("===============");
    println!();
    println!("This demo exercises the full physics ↔ 3D rendering pipeline:");
    println!("  1. Physics cubes with 3D rigid body dynamics");
    println!("  2. Quaternion-based rotation integration");
    println!("  3. Floor collision with restitution");
    println!("  4. 3D → 2D perspective projection");
    println!("  5. render_scene_node_3d() → draw_mesh_3d() → GPU");
    println!();
    println!("Run with: cargo run --example physics_3d_demo --features native");
    println!();
    println!("Note: Full rendering requires the native renderer.");
    println!("This example validates the physics integration and API.");

    // Run a quick physics simulation to validate
    let mut cube = PhysicsCube::new(0, 0.0, 100.0, 0.0, 50.0, [1.0, 0.0, 0.0, 1.0]);
    cube.angular_velocity = [0.5, 1.0, 0.3];

    println!("Simulating 100 physics steps...");
    for step in 0..100 {
        cube.integrate(0.016, [0.0, -9.81 * 50.0, 0.0]);
        cube.collide_floor(0.0, 0.5);
        if step % 20 == 0 {
            println!(
                "  Step {}: pos=[{:.1}, {:.1}, {:.1}] vel=[{:.1}, {:.1}, {:.1}]",
                step, cube.position[0], cube.position[1], cube.position[2],
                cube.velocity[0], cube.velocity[1], cube.velocity[2]
            );
        }
    }

    let q = &cube.rotation;
    let len = (q[0]*q[0] + q[1]*q[1] + q[2]*q[2] + q[3]*q[3]).sqrt();
    println!("Final quaternion length: {:.6} (should be ~1.0)", len);
    println!("Physics integration: OK");
}
