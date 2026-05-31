//! Semi-implicit Euler integration for rigid bodies.

use glam::Vec2;

use crate::RigidBody;

/// Integrate a rigid body forward by `dt` using semi-implicit Euler.
///
/// This applies forces to get acceleration, updates velocity, then updates position.
/// It's more stable than explicit Euler for oscillatory systems.
///
/// # Arguments
/// * `body` — The body to integrate (modified in-place).
/// * `dt` — Time step in seconds.
/// * `gravity` — Global gravity vector (pixels/s²).
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
