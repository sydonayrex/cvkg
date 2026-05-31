//! Scene bridge: maps physics bodies to cvkg-scene nodes and writes transforms back.
//!
//! Supports both 2D and 3D physics bodies. For 3D bodies, the bridge writes
//! position_3d, rotation_3d, and scale_3d into the corresponding VNode's
//! 3D transform fields.

use glam::Vec2;
use glam::Vec3;

use cvkg_scene::{NodeId, SceneGraph};

use crate::BodyId;

/// Data for a 3D body to be synced to the scene graph.
#[derive(Debug, Clone, Copy)]
pub struct Body3DTransform {
    pub position: Vec3,
    pub rotation: glam::Quat,
}

/// Maps physics bodies to scene graph nodes and syncs transforms.
pub struct SceneBridge {
    /// Mapping from BodyId to the NodeId it controls.
    mappings: Vec<(BodyId, NodeId)>, // Using Vec instead of HashMap for cache-friendly iteration
}

impl Default for SceneBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneBridge {
    /// Create a new empty scene bridge.
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }

    /// Register a body-to-node mapping.
    pub fn bind(&mut self, body_id: BodyId, node_id: NodeId) {
        self.mappings.push((body_id, node_id));
    }

    /// Remove a binding by BodyId.
    pub fn unbind(&mut self, body_id: BodyId) {
        self.mappings.retain(|(b, _)| *b != body_id);
    }

    /// Write 2D physics transforms into the scene graph.
    #[allow(clippy::collapsible_if)]
    pub fn sync_to_scene(
        &self,
        body_positions: &std::collections::HashMap<BodyId, (Vec2, f32)>,
        scene: &mut SceneGraph,
    ) {
        for (body_id, node_id) in &self.mappings {
            if let Some((pos, angle)) = body_positions.get(body_id) {
                if let Some(node) = scene.nodes.get_mut(node_id) {
                    let new_x = pos.x - node.local_rect.width * 0.5;
                    let new_y = pos.y - node.local_rect.height * 0.5;

                    // Only mark dirty if actually changed
                    if (new_x - node.local_rect.x).abs() > 0.01
                        || (new_y - node.local_rect.y).abs() > 0.01
                    {
                        node.local_rect.x = new_x;
                        node.local_rect.y = new_y;
                        node.z_index = *angle;
                        node.is_dirty = true;
                    }
                }
            }
        }
        scene.update_transforms();
    }

    /// Write 3D physics transforms into the scene graph.
    ///
    /// For each bound 3D body, update the position_3d and rotation_3d
    /// fields of the corresponding scene graph node. The renderer will
    /// use these to construct model matrices for draw_mesh_3d calls.
    pub fn sync_3d_to_scene(
        &self,
        body_transforms: &std::collections::HashMap<BodyId, Body3DTransform>,
        scene: &mut SceneGraph,
    ) {
        for (body_id, node_id) in &self.mappings {
            if let Some(xform) = body_transforms.get(body_id) {
                if let Some(node) = scene.nodes.get_mut(node_id) {
                    let new_pos = [xform.position.x, xform.position.y, xform.position.z];
                    let new_rot = [xform.rotation.x, xform.rotation.y, xform.rotation.z, xform.rotation.w];

                    // Only mark dirty if actually changed
                    let pos_changed = (0..3).any(|i| (new_pos[i] - node.position_3d[i]).abs() > 0.001);
                    let rot_changed = (0..4).any(|i| (new_rot[i] - node.rotation_3d[i]).abs() > 0.001);

                    if pos_changed || rot_changed {
                        node.position_3d = new_pos;
                        node.rotation_3d = new_rot;
                        node.is_3d = true;
                        // Derive 2D fallback from 3D position for compatibility
                        node.local_rect.x = new_pos[0] - node.local_rect.width * 0.5;
                        node.local_rect.y = new_pos[1] - node.local_rect.height * 0.5;
                        node.is_dirty = true;
                    }
                }
            }
        }
        scene.update_transforms();
    }

    /// Get all node IDs this bridge controls.
    pub fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.mappings.iter().map(|(_, n)| *n)
    }

    /// Get the number of bindings.
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// Returns true if there are no bindings.
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_bind() {
        let mut bridge = SceneBridge::new();
        bridge.bind(BodyId(1), NodeId(100));
        assert_eq!(bridge.len(), 1);

        bridge.bind(BodyId(2), NodeId(200));
        assert_eq!(bridge.len(), 2);

        bridge.unbind(BodyId(1));
        assert_eq!(bridge.len(), 1);
    }

    #[test]
    fn test_sync_3d_to_scene() {
        let mut bridge = SceneBridge::new();
        let mut scene = SceneGraph::new();

        // Create a node with 3D transform
        let node_id = NodeId(42);
        scene.nodes.insert(node_id, cvkg_scene::VNode::new(
            node_id,
            "Cube",
            cvkg_core::Rect::new(-0.5, -0.5, 1.0, 1.0),
        ));
        let body_id = BodyId(1);
        bridge.bind(body_id, node_id);

        // Sync a 3D transform
        let mut transforms = std::collections::HashMap::new();
        transforms.insert(body_id, Body3DTransform {
            position: Vec3::new(10.0, 20.0, 5.0),
            rotation: glam::Quat::from_rotation_z(0.5),
        });
        bridge.sync_3d_to_scene(&transforms, &mut scene);

        let node = scene.nodes.get(&node_id).unwrap();
        assert!(node.is_3d);
        assert_eq!(node.position_3d, [10.0, 20.0, 5.0]);
        assert!((node.rotation_3d[3] - 0.9689).abs() < 0.01); // w component of quat
        assert!(node.is_dirty);
    }
}