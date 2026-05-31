//! Scene bridge: maps physics bodies to cvkg-scene nodes and writes transforms back.
//!
//! This is the key architectural module that connects the physics world to
//! the visual rendering pipeline. After each simulation step, the bridge
//! writes computed transforms back into the cvkg-scene `SceneGraph`, and
//! the renderer picks them up naturally.

use glam::Vec2;

use cvkg_scene::{NodeId, SceneGraph};

use crate::BodyId;

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

    /// Write physics transforms into the scene graph.
    ///
    /// For each bound body, update the local_rect and position
    /// of the corresponding scene graph node.
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
                        // Store rotation in the z-index field as a convention
                        // (proper transform support would need scene graph extensions)
                        node.z_index = *angle;
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
}
