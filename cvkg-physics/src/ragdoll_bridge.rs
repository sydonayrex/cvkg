//! Ragdoll Bridge: maps physics bodies to animation bones and feeds physics
//! transforms into cvkg-anim's RagdollBlender.
//!
//! This bridge allows physics-driven ragdoll simulation to blend with
//! keyframed animation via cvkg-anim's RagdollBlender.

use crate::{BodyId, PhysicsWorld};
use glam::{Quat, Vec3};

/// Maps a physics body to an animation bone.
#[derive(Debug, Clone)]
pub struct BoneBodyMap {
    /// Index of the bone in the skeleton.
    pub bone_index: usize,
    /// Physics body ID that drives this bone.
    pub body_id: BodyId,
    /// Local offset from body center to bone joint.
    pub local_offset: Vec3,
    /// Local rotation from body frame to bone frame.
    pub local_rotation: Quat,
}

/// Configuration for the ragdoll bridge.
#[derive(Debug, Clone)]
pub struct RagdollBridgeConfig {
    /// Maps physics bodies to skeleton bones.
    pub bone_mappings: Vec<BoneBodyMap>,
    /// Whether to apply physics transforms to the body (true) or
    /// read animated transforms and apply to body (false).
    pub physics_drives_animation: bool,
}

impl Default for RagdollBridgeConfig {
    fn default() -> Self {
        Self {
            bone_mappings: Vec::new(),
            physics_drives_animation: true,
        }
    }
}

/// Bridge that syncs physics bodies with animation bones.
pub struct RagdollBridge {
    config: RagdollBridgeConfig,
    /// Cached physics transforms for bones (world space).
    physics_transforms: Vec<(Vec3, Quat)>,
    /// Whether the bridge is active.
    pub enabled: bool,
}

impl RagdollBridge {
    /// Create a new ragdoll bridge with the given configuration.
    pub fn new(config: RagdollBridgeConfig) -> Self {
        let bone_count = config.bone_mappings.len();
        Self {
            config,
            physics_transforms: vec![(Vec3::ZERO, Quat::IDENTITY); bone_count],
            enabled: true,
        }
    }

    /// Update the bridge: extract physics transforms and apply to bones.
    ///
    /// If `physics_drives_animation` is true, reads physics body transforms
    /// and writes to `physics_transforms` for consumption by RagdollBlender.
    /// If false, reads animated transforms (not implemented here) and applies
    /// to physics bodies.
    pub fn update(&mut self, world: &PhysicsWorld) {
        if !self.enabled {
            return;
        }

        for (i, mapping) in self.config.bone_mappings.iter().enumerate() {
            if let Some(body) = world.body(mapping.body_id) {
                // Get physics body transform
                let (pos, rot) = if body.is_3d {
                    (body.position_3d, body.rotation)
                } else {
                    (body.position.extend(0.0), Quat::from_rotation_z(body.angle))
                };

                // Apply local offset and rotation to get bone transform
                let bone_pos = pos + rot * mapping.local_offset;
                let bone_rot = rot * mapping.local_rotation;

                self.physics_transforms[i] = (bone_pos, bone_rot);
            }
        }
    }

    /// Get the current physics transforms for all mapped bones.
    ///
    /// Returns a slice of (position, rotation) pairs, one per bone mapping.
    /// This can be passed directly to `RagdollBlender::set_physics()`.
    pub fn physics_transforms(&self) -> &[(Vec3, Quat)] {
        &self.physics_transforms
    }

    /// Apply animated transforms to physics bodies (for animation-driven physics).
    ///
    /// This is the inverse of `update()`: reads animated bone transforms
    /// and applies them to the corresponding physics bodies.
    pub fn apply_animated(
        &mut self,
        world: &mut PhysicsWorld,
        animated_transforms: &[(Vec3, Quat)],
    ) {
        if !self.enabled || self.config.physics_drives_animation {
            return;
        }

        for (i, mapping) in self.config.bone_mappings.iter().enumerate() {
            if let Some(animated) = animated_transforms.get(i)
                && let Some(body) = world.body_mut(mapping.body_id)
            {
                // Convert bone transform back to body transform
                let inv_rot = mapping.local_rotation.inverse();
                let body_rot = animated.1 * inv_rot;
                let body_pos = animated.0 - body_rot * mapping.local_offset;

                if body.is_3d {
                    body.position_3d = body_pos;
                    body.rotation = body_rot;
                } else {
                    body.position = body_pos.truncate();
                    body.angle = body_rot.to_euler(glam::EulerRot::XYZ).2; // Z rotation
                }
            }
        }
    }

    /// Enable or disable the bridge.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get the number of bone mappings.
    pub fn bone_count(&self) -> usize {
        self.config.bone_mappings.len()
    }

    /// Get a reference to the bone mappings.
    pub fn bone_mappings(&self) -> &[BoneBodyMap] {
        &self.config.bone_mappings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PhysicsWorld, RigidBody, Shape, WorldConfig};

    #[test]
    fn test_ragdoll_bridge_creation() {
        let mut world = PhysicsWorld::new(WorldConfig::default());
        let body_id = world.add_body(RigidBody::new_3d(1.0, &Shape::sphere(0.5)));

        let config = RagdollBridgeConfig {
            bone_mappings: vec![BoneBodyMap {
                bone_index: 0,
                body_id,
                local_offset: Vec3::ZERO,
                local_rotation: Quat::IDENTITY,
            }],
            physics_drives_animation: true,
        };

        let bridge = RagdollBridge::new(config);
        assert_eq!(bridge.bone_count(), 1);
        assert!(bridge.enabled);
    }

    #[test]
    fn test_ragdoll_bridge_update() {
        let mut world = PhysicsWorld::new(WorldConfig::default());
        let body_id = world.add_body(RigidBody::new_3d(1.0, &Shape::sphere(0.5)));

        // Set body position
        if let Some(body) = world.body_mut(body_id) {
            body.position_3d = Vec3::new(1.0, 2.0, 3.0);
            body.rotation = Quat::from_rotation_y(0.5);
        }

        let config = RagdollBridgeConfig {
            bone_mappings: vec![BoneBodyMap {
                bone_index: 0,
                body_id,
                local_offset: Vec3::ZERO,
                local_rotation: Quat::IDENTITY,
            }],
            physics_drives_animation: true,
        };

        let mut bridge = RagdollBridge::new(config);
        bridge.update(&world);

        let transforms = bridge.physics_transforms();
        assert_eq!(transforms[0].0, Vec3::new(1.0, 2.0, 3.0));
    }
}
