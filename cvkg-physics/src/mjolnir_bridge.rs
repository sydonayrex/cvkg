//! Mjolnir Bridge: helper functions to trigger Mjolnir shatter effects
//! when physics constraints break or at arbitrary positions.
//!
//! This module provides high-level helpers that applications can use in their
//! `PhysicsWorld::on_constraint_broken` callback or anywhere else to trigger
//! Mjolnir shatter effects. Since cvkg-physics doesn't depend on cvkg-render-gpu,
//! these functions take a callback/closure that the application must implement
//! to actually trigger the renderer's shatter effect.

use crate::{BodyId, PhysicsWorld};
use cvkg_core::Rect;

/// Trigger a Mjolnir shatter effect at the midpoint between two bodies
/// that just had their constraint broken.
///
/// # Arguments
/// * `world` - The physics world (to look up body positions)
/// * `body_a` - First body ID from the constraint break callback
/// * `body_b` - Second body ID from the constraint break callback
/// * `shatter_fn` - Closure that triggers the actual Mjolnir shatter effect.
///   Receives: (rect, pieces, force, color)
///
/// # Default parameters
/// * pieces: 12
/// * force: 5.0
/// * color: Shatter neon (magenta-pink) [1.0, 0.0, 0.75, 1.0]
/// * use_fluid: false (use standard shatter)
pub fn shatter_at_constraint_break<F>(
    world: &PhysicsWorld,
    body_a: BodyId,
    body_b: BodyId,
    mut shatter_fn: impl FnMut(Rect, u32, f32, [f32; 4]),
) {
    let pos_a = world.body(body_a).map(|b| {
        if b.is_3d {
            [b.position_3d.x, b.position_3d.y]
        } else {
            [b.position.x, b.position.y]
        }
    });
    let pos_b = world.body(body_b).map(|b| {
        if b.is_3d {
            [b.position_3d.x, b.position_3d.y]
        } else {
            [b.position.x, b.position.y]
        }
    });

    let (Some(a), Some(b)) = (pos_a, pos_b) else {
        log::warn!("Mjolnir shatter: one or both bodies not found");
        return;
    };

    // Midpoint between the two bodies
    let mid_x = (a[0] + b[0]) * 0.5;
    let mid_y = (a[1] + b[1]) * 0.5;

    // Calculate rect size based on distance between bodies
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dist = (dx * dx + dy * dy).sqrt().max(1.0);
    let size = (dist * 1.5).clamp(40.0, 400.0);

    let rect = Rect::new(mid_x - size * 0.5, mid_y - size * 0.5, size, size);

    shatter_fn(rect, 12, 5.0, [1.0, 0.0, 0.75, 1.0]);
}

/// Trigger a Mjolnir shatter effect at an arbitrary world position.
///
/// # Arguments
/// * `position` - World position [x, y] for the shatter center
/// * `shatter_fn` - Closure that triggers the actual Mjolnir shatter effect.
///   Receives: (rect, pieces, force, color)
pub fn shatter_at_position<F>(
    position: [f32; 2],
    mut shatter_fn: impl FnMut(Rect, u32, f32, [f32; 4]),
) {
    const DEFAULT_SIZE: f32 = 100.0;
    let rect = Rect::new(
        position[0] - DEFAULT_SIZE * 0.5,
        position[1] - DEFAULT_SIZE * 0.5,
        DEFAULT_SIZE,
        DEFAULT_SIZE,
    );
    shatter_fn(rect, 8, 4.0, [1.0, 0.0, 0.75, 1.0]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PhysicsWorld, RigidBody, Shape, WorldConfig};

    #[test]
    fn test_shatter_at_constraint_break() {
        let mut world = PhysicsWorld::new(WorldConfig::default());
        let id1 = world.add_body(RigidBody::new(1.0, &Shape::circle(16.0)));
        let id2 = world.add_body(RigidBody::new(1.0, &Shape::circle(16.0)));

        if let Some(body) = world.body_mut(id1) {
            body.position = [100.0, 100.0].into();
        }
        if let Some(body) = world.body_mut(id2) {
            body.position = [200.0, 200.0].into();
        }

        let mut called = false;
        shatter_at_constraint_break::<fn(Rect, u32, f32, [f32; 4])>(
            &world,
            id1,
            id2,
            |rect: Rect, pieces: u32, force: f32, _color: [f32; 4]| {
                called = true;
                assert_eq!(pieces, 12);
                assert!((force - 5.0).abs() < 0.001);
                assert!((rect.x - 44.0).abs() < 5.0); // Midpoint around (150, 150), size ~212, so x ≈ 150 - 106 = 44
            },
        );
        assert!(called);
    }

    #[test]
    fn test_shatter_at_position() {
        let mut called = false;
        shatter_at_position::<fn(Rect, u32, f32, [f32; 4])>(
            [400.0, 300.0],
            |rect: Rect, pieces: u32, _force: f32, _color: [f32; 4]| {
                called = true;
                assert!((rect.x - 350.0).abs() < 1.0);
                assert!((rect.y - 250.0).abs() < 1.0);
                assert_eq!(pieces, 8);
            },
        );
        assert!(called);
    }
}
