use std::collections::{HashMap, VecDeque};

use cvkg_core::NodeId;
use cvkg_core::Transform3D;
use glam::Mat4;

/// Errors that can occur during hierarchy validation or transform propagation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HierarchyError {
    /// Two or more nodes share the same `NodeId`.
    DuplicateId(NodeId),
    /// A node references itself as its own parent.
    SelfReference(NodeId),
    /// A node references a `parent` id that does not exist in the node list.
    ParentNotFound {
        child: NodeId,
        parent: NodeId,
    },
    /// A cycle exists among the parent-child relationships.
    CycleDetected(Vec<NodeId>),
}

impl std::fmt::Display for HierarchyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HierarchyError::DuplicateId(id) => write!(f, "duplicate node id: {}", id.0),
            HierarchyError::SelfReference(id) => {
                write!(f, "node {} references itself as parent", id.0)
            }
            HierarchyError::ParentNotFound { child, parent } => {
                write!(
                    f,
                    "node {} references non-existent parent {}",
                    child.0, parent.0
                )
            }
            HierarchyError::CycleDetected(ids) => {
                write!(
                    f,
                    "cycle detected among nodes: {}",
                    ids.iter()
                        .map(|id| id.0.to_string())
                        .collect::<Vec<_>>()
                        .join(" -> ")
                )
            }
        }
    }
}

impl std::error::Error for HierarchyError {}

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

/// Walk the `nodes` slice and compute each node's `global` matrix from its
/// parent's `global` and its own `local` transform.
///
/// # Topological Order
/// The function validates that the hierarchy is a valid DAG (no cycles,
/// no self-references) and sorts nodes in topological order (parents
/// before children) before computing transforms. This means the caller
/// does **not** need to pre-sort the slice.
///
/// # Errors
/// Returns an error if:
/// - Any two nodes share the same `id` ([`HierarchyError::DuplicateId`])
/// - A node references itself as parent ([`HierarchyError::SelfReference`])
/// - A `parent` id does not exist in the node list ([`HierarchyError::ParentNotFound`])
/// - A cycle exists among parent-child relationships ([`HierarchyError::CycleDetected`])
pub fn propagate_transforms(nodes: &mut [TransformNode3D]) -> Result<(), HierarchyError> {
    if nodes.is_empty() {
        return Ok(());
    }

    // 1. Build index map and detect duplicate IDs.
    let mut index_map: HashMap<NodeId, usize> = HashMap::with_capacity(nodes.len());
    for (i, node) in nodes.iter().enumerate() {
        if index_map.insert(node.id, i).is_some() {
            return Err(HierarchyError::DuplicateId(node.id));
        }
    }

    // 2. Validate: no self-references, all parent IDs exist.
    for node in nodes.iter() {
        if let Some(parent_id) = node.parent {
            if parent_id == node.id {
                return Err(HierarchyError::SelfReference(node.id));
            }
            if !index_map.contains_key(&parent_id) {
                return Err(HierarchyError::ParentNotFound {
                    child: node.id,
                    parent: parent_id,
                });
            }
        }
    }

    // 3. Topological sort (Kahn's algorithm) — detects cycles and produces
    //    a valid parent-before-child ordering.
    let n = nodes.len();
    let mut in_degree = vec![0u32; n];
    let mut children_of: Vec<Vec<usize>> = vec![Vec::new(); n];

    for (i, node) in nodes.iter().enumerate() {
        if let Some(parent_id) = node.parent {
            let parent_idx = index_map[&parent_id];
            children_of[parent_idx].push(i);
            in_degree[i] += 1;
        }
    }

    let mut queue: VecDeque<usize> = VecDeque::new();
    for (i, &deg) in in_degree.iter().enumerate() {
        if deg == 0 {
            queue.push_back(i);
        }
    }

    let mut topo_order: Vec<usize> = Vec::with_capacity(n);
    while let Some(idx) = queue.pop_front() {
        topo_order.push(idx);
        for &child_idx in &children_of[idx] {
            in_degree[child_idx] -= 1;
            if in_degree[child_idx] == 0 {
                queue.push_back(child_idx);
            }
        }
    }

    if topo_order.len() != n {
        // Collect the cycle for diagnostics.
        let cycle_nodes: Vec<NodeId> = (0..n)
            .filter(|&i| in_degree[i] > 0)
            .map(|i| nodes[i].id)
            .collect();
        return Err(HierarchyError::CycleDetected(cycle_nodes));
    }

    // 4. Compute transforms in topological order.
    //    Parents are guaranteed to appear before children in topo_order,
    //    so their `global` fields are already computed when we process
    //    each child. The slice order is not changed — callers can use
    //    the index_map if they need topological ordering.
    for &idx in &topo_order {
        let local_mat = nodes[idx].local.to_mat4();
        let global = match nodes[idx].parent {
            None => local_mat,
            Some(parent_id) => {
                let parent_idx = index_map[&parent_id];
                nodes[parent_idx].global * local_mat
            }
        };
        nodes[idx].global = global;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cvkg_core::KvasirId;
    use glam::Vec3;

    #[test]
    fn test_empty() {
        let mut nodes: Vec<TransformNode3D> = vec![];
        assert_eq!(propagate_transforms(&mut nodes), Ok(()));
    }

    #[test]
    fn test_root() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let mut nodes = vec![TransformNode3D {
            id: KvasirId(1),
            parent: None,
            children: vec![],
            local: Transform3D {
                position: pos,
                ..Transform3D::default()
            },
            global: Mat4::IDENTITY,
        }];
        propagate_transforms(&mut nodes).unwrap();
        let expected = Transform3D {
            position: pos,
            ..Transform3D::default()
        }
        .to_mat4();
        assert_eq!(nodes[0].global, expected);
    }

    #[test]
    fn test_duplicate_id() {
        let mut nodes = vec![
            TransformNode3D {
                id: KvasirId(1),
                parent: None,
                children: vec![],
                local: Transform3D::default(),
                global: Mat4::IDENTITY,
            },
            TransformNode3D {
                id: KvasirId(1), // duplicate
                parent: None,
                children: vec![],
                local: Transform3D::default(),
                global: Mat4::IDENTITY,
            },
        ];
        assert_eq!(
            propagate_transforms(&mut nodes),
            Err(HierarchyError::DuplicateId(KvasirId(1)))
        );
    }

    #[test]
    fn test_self_reference() {
        let mut nodes = vec![TransformNode3D {
            id: KvasirId(1),
            parent: Some(KvasirId(1)), // self
            children: vec![],
            local: Transform3D::default(),
            global: Mat4::IDENTITY,
        }];
        assert_eq!(
            propagate_transforms(&mut nodes),
            Err(HierarchyError::SelfReference(KvasirId(1)))
        );
    }

    #[test]
    fn test_parent_not_found() {
        let mut nodes = vec![TransformNode3D {
            id: KvasirId(1),
            parent: Some(KvasirId(99)), // doesn't exist
            children: vec![],
            local: Transform3D::default(),
            global: Mat4::IDENTITY,
        }];
        assert_eq!(
            propagate_transforms(&mut nodes),
            Err(HierarchyError::ParentNotFound {
                child: KvasirId(1),
                parent: KvasirId(99),
            })
        );
    }

    #[test]
    fn test_cycle_two_nodes() {
        let mut nodes = vec![
            TransformNode3D {
                id: KvasirId(1),
                parent: Some(KvasirId(2)),
                children: vec![],
                local: Transform3D::default(),
                global: Mat4::IDENTITY,
            },
            TransformNode3D {
                id: KvasirId(2),
                parent: Some(KvasirId(1)),
                children: vec![],
                local: Transform3D::default(),
                global: Mat4::IDENTITY,
            },
        ];
        let result = propagate_transforms(&mut nodes);
        assert!(matches!(result, Err(HierarchyError::CycleDetected(_))));
    }

    #[test]
    fn test_parent_after_child_is_correct() {
        // Child appears before parent in the slice — should still compute correctly.
        let mut nodes = vec![
            TransformNode3D {
                id: KvasirId(2),
                parent: Some(KvasirId(1)),
                children: vec![],
                local: Transform3D {
                    position: Vec3::new(0.0, 5.0, 0.0),
                    ..Transform3D::default()
                },
                global: Mat4::IDENTITY,
            },
            TransformNode3D {
                id: KvasirId(1),
                parent: None,
                children: vec![KvasirId(2)],
                local: Transform3D {
                    position: Vec3::new(10.0, 0.0, 0.0),
                    ..Transform3D::default()
                },
                global: Mat4::IDENTITY,
            },
        ];
        propagate_transforms(&mut nodes).unwrap();

        let root_mat = Transform3D {
            position: Vec3::new(10.0, 0.0, 0.0),
            ..Transform3D::default()
        }
        .to_mat4();
        let child_local = Transform3D {
            position: Vec3::new(0.0, 5.0, 0.0),
            ..Transform3D::default()
        }
        .to_mat4();

        // Slice order is unchanged, but transforms are correct.
        let root = nodes.iter().find(|n| n.id == KvasirId(1)).unwrap();
        let child = nodes.iter().find(|n| n.id == KvasirId(2)).unwrap();
        assert_eq!(root.global, root_mat);
        assert_eq!(child.global, root_mat * child_local);
    }

    #[test]
    fn test_multi_level_chain() {
        let mut nodes = vec![
            TransformNode3D {
                id: KvasirId(1),
                parent: None,
                children: vec![KvasirId(2)],
                local: Transform3D {
                    position: Vec3::new(1.0, 0.0, 0.0),
                    ..Transform3D::default()
                },
                global: Mat4::IDENTITY,
            },
            TransformNode3D {
                id: KvasirId(2),
                parent: Some(KvasirId(1)),
                children: vec![KvasirId(3)],
                local: Transform3D {
                    position: Vec3::new(0.0, 2.0, 0.0),
                    ..Transform3D::default()
                },
                global: Mat4::IDENTITY,
            },
            TransformNode3D {
                id: KvasirId(3),
                parent: Some(KvasirId(2)),
                children: vec![],
                local: Transform3D {
                    position: Vec3::new(0.0, 0.0, 3.0),
                    ..Transform3D::default()
                },
                global: Mat4::IDENTITY,
            },
        ];
        propagate_transforms(&mut nodes).unwrap();

        let root = Transform3D {
            position: Vec3::new(1.0, 0.0, 0.0),
            ..Transform3D::default()
        }
        .to_mat4();
        let child = Transform3D {
            position: Vec3::new(0.0, 2.0, 0.0),
            ..Transform3D::default()
        }
        .to_mat4();
        let grandchild = Transform3D {
            position: Vec3::new(0.0, 0.0, 3.0),
            ..Transform3D::default()
        }
        .to_mat4();

        assert_eq!(nodes[0].global, root);
        assert_eq!(nodes[1].global, root * child);
        assert_eq!(nodes[2].global, root * child * grandchild);
    }
}
