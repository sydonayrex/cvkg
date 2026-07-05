use std::collections::HashMap;

use cvkg_core::NodeId;
use cvkg_core::Transform3D;
use glam::Mat4;

/// A node in a 3D transform hierarchy.
///
/// Each node stores its local transform (relative to parent) and
/// a cached global (world-space) matrix that is filled by
/// [`propagate_transforms`].
#[derive(Debug, Clone)]
pub struct TransformNode3D {
    /// Unique identifier for this node.
    pub id: NodeId,
    /// Parent node id, `None` for roots.
    pub parent: Option<NodeId>,
    /// Children of this node (order matters for depth-first traversal).
    pub children: Vec<NodeId>,
    /// Local-space transform relative to the parent.
    pub local: Transform3D,
    /// Computed world-space matrix. Populated by [`propagate_transforms`].
    pub global: Mat4,
}

/// Walk the `nodes` slice in index order and compute each node's `global`
/// matrix from its parent's `global` and its own `local` transform.
///
/// # Preconditions
/// * Every `parent` id that is `Some` must refer to a node earlier in the
///   slice (i.e. parents appear before children).
/// * The `nodes` slice must not contain duplicate `id` values.
///
/// # Panics
/// Panics (in debug) or produces undefined results if a parent id is not
/// found in the index map.
pub fn propagate_transforms(nodes: &mut [TransformNode3D]) {
    let index_map: HashMap<NodeId, usize> =
        nodes.iter().enumerate().map(|(i, n)| (n.id, i)).collect();

    for i in 0..nodes.len() {
        let local_mat = nodes[i].local.to_mat4();
        let global = match nodes[i].parent {
            None => local_mat,
            Some(parent_id) => {
                let parent_idx = *index_map
                    .get(&parent_id)
                    .expect("propagate_transforms: parent id not found in node list");
                nodes[parent_idx].global * local_mat
            }
        };
        nodes[i].global = global;
    }
}
