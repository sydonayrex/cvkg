//! Semi-implicit Euler integration for rigid bodies.

use glam::Vec2;

use crate::RigidBody;

/// Integrate a rigid body forward by `dt` using semi-implicit Euler.
///
/// This applies forces to get acceleration, updates velocity, then updates position.
/// It's more stable than explicit Euler for oscillatory systems.
///
/// # Arguments
/// * `body` -- The body to integrate (modified in-place).
/// * `dt` -- Time step in seconds.
/// * `gravity` -- Global gravity vector (pixels/s²).
pub fn semi_implicit_euler(body: &mut RigidBody, dt: f32, gravity: Vec2) {
    if body.is_static || body.is_sleeping {
        return;
    }

    // Acceleration = force / mass + gravity
    let acceleration = body.force * body.inv_mass + gravity * body.gravity_scale;

    // Update velocity
    body.velocity += acceleration * dt;

    // Apply damping
    body.velocity *= 1.0 / (1.0 + dt * body.linear_damping);

    // Update position
    body.position += body.velocity * dt;

    // Angular integration
    body.angular_velocity += body.torque * body.inv_inertia * dt;
    body.angular_velocity *= 1.0 / (1.0 + dt * body.angular_damping);
    body.angle += body.angular_velocity * dt;

    // Clear accumulated forces
    body.force = Vec2::ZERO;
    body.torque = 0.0;
}

/// Wake a sleeping body.
pub fn wake(body: &mut RigidBody) {
    body.is_sleeping = false;
    body.sleep_counter = 0;
}

/// Update sleep state. Returns `true` if the body just went to sleep.
pub fn update_sleep(body: &mut RigidBody, sleep_threshold: f32, sleep_delay: u32) -> bool {
    if body.is_static {
        return false;
    }

    let speed_sq = body.velocity.length_squared() + body.angular_velocity * body.angular_velocity;

    if speed_sq < sleep_threshold * sleep_threshold {
        body.sleep_counter += 1;
        if body.sleep_counter >= sleep_delay {
            body.velocity = Vec2::ZERO;
            body.angular_velocity = 0.0;
            body.is_sleeping = true;
            return true; // just fell asleep
        }
    } else {
        body.sleep_counter = 0;
    }
    false
}

// ══════════════════════════════════════════════════════════════════════════
// 3D Integration
// ══════════════════════════════════════════════════════════════════════════

use glam::Vec3;

/// Integrate a 3D rigid body forward by `dt` using semi-implicit Euler.
///
/// Uses quaternion for rotation and Vec3 for angular velocity/torque.
pub fn semi_implicit_euler_3d(body: &mut RigidBody, dt: f32, gravity: Vec3) {
    if body.is_static || body.is_sleeping {
        return;
    }

    // Linear acceleration = force / mass + gravity
    let acceleration = body.force_3d * body.inv_mass + gravity * body.gravity_scale;

    // Update linear velocity
    body.velocity_3d += acceleration * dt;

    // Apply linear damping
    body.velocity_3d *= 1.0 / (1.0 + dt * body.linear_damping);

    // Update position
    body.position_3d += body.velocity_3d * dt;

    // Angular integration (3D)
    // Angular acceleration = torque * inv_inertia (component-wise)
    let angular_acceleration = body.torque_3d * body.inv_inertia_3d;
    body.angular_velocity_3d += angular_acceleration * dt;

    // Apply angular damping
    body.angular_velocity_3d *= 1.0 / (1.0 + dt * body.angular_damping);

    // Update rotation via quaternion derivative
    // dq/dt = 0.5 * omega * q
    let omega = body.angular_velocity_3d;
    let half_dt = 0.5 * dt;
    // Quaternion multiplication: omega * q (treating omega as pure quaternion)
    let omega_quat = glam::Quat::from_xyzw(omega.x, omega.y, omega.z, 0.0);
    let q_dot = omega_quat * body.rotation;
    body.rotation = glam::Quat::from_xyzw(
        body.rotation.x + q_dot.x * half_dt,
        body.rotation.y + q_dot.y * half_dt,
        body.rotation.z + q_dot.z * half_dt,
        body.rotation.w + q_dot.w * half_dt,
    );
    // Normalize to prevent drift
    body.rotation = body.rotation.normalize();

    // Clear accumulated forces
    body.force_3d = Vec3::ZERO;
    body.torque_3d = Vec3::ZERO;
}
