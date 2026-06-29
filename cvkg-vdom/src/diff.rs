use crate::VDom;
use crate::vnode::{AriaProps, EventHandlerMap, LayoutRect, NodeId, VNode};
use serde::Deserialize;
use std::collections::HashMap;

/// A discrete mutation to the Virtual DOM tree.
#[derive(Clone)]
pub enum VDomPatch {
    /// Create and append a new node
    Create(VNode),
    /// Update properties of an existing node
    Update {
        /// ID of the node to update
        id: NodeId,
        /// Updated properties map
        props: Option<HashMap<String, serde_json::Value>>,
        /// Updated layout
        layout: Option<LayoutRect>,
        /// Updated ARIA properties
        aria_props: Option<AriaProps>,
        /// Updated ARIA role
        aria_role: Option<String>,
        /// Updated children list
        children: Option<Vec<NodeId>>,
        /// Updated event handlers
        handlers: Option<EventHandlerMap>,
        /// Updated SDF shape
        sdf_shape: Option<cvkg_core::layout::SdfShape>,
    },
    /// Remove an existing node
    Remove(NodeId),
    /// Replace an existing node completely with a new one
    Replace {
        /// ID of the node being replaced
        id: NodeId,
        /// The new node to substitute
        node: VNode,
    },
    /// Move a keyed node to a new position within its parent
    Move {
        /// ID of the node being moved
        id: NodeId,
        /// The new index position
        new_index: usize,
    },
    /// Update the root node ID
    SetRoot(Option<NodeId>),
    /// Clear all event handlers attached to a node.
    ///
    /// Without this variant, `Update { handlers: None }` cannot remove
    /// handlers -- the apply step interprets `None` as "leave handlers
    /// unchanged". Emitted when the new tree has no handlers for a node
    /// but the old tree did.
    ClearHandlers {
        /// ID of the node whose handlers should be cleared.
        id: NodeId,
    },
}

impl std::fmt::Debug for VDomPatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Create(node) => f.debug_tuple("Create").field(node).finish(),
            Self::Update {
                id,
                props,
                layout,
                aria_props,
                aria_role,
                children,
                handlers,
                sdf_shape,
            } => f
                .debug_struct("Update")
                .field("id", id)
                .field("props", props)
                .field("layout", layout)
                .field("aria_props", aria_props)
                .field("aria_role", aria_role)
                .field("children", children)
                .field("handlers_count", &handlers.as_ref().map(|h| h.len()))
                .field("sdf_shape", sdf_shape)
                .finish(),
            Self::Remove(id) => f.debug_tuple("Remove").field(id).finish(),
            Self::Replace { id, node } => f
                .debug_struct("Replace")
                .field("id", id)
                .field("node", node)
                .finish(),
            Self::Move { id, new_index } => f
                .debug_struct("Move")
                .field("id", id)
                .field("new_index", new_index)
                .finish(),
            Self::SetRoot(id) => f.debug_tuple("SetRoot").field(id).finish(),
            Self::ClearHandlers { id } => f.debug_struct("ClearHandlers").field("id", id).finish(),
        }
    }
}

impl serde::Serialize for VDomPatch {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStructVariant;
        match self {
            Self::Create(node) => {
                serializer.serialize_newtype_variant("VDomPatch", 0, "Create", node)
            }
            Self::Update {
                id,
                props,
                layout,
                aria_props,
                aria_role,
                children,
                handlers,
                sdf_shape,
            } => {
                let mut state = serializer.serialize_struct_variant("VDomPatch", 1, "Update", 8)?;
                state.serialize_field("id", id)?;
                state.serialize_field("props", props)?;
                state.serialize_field("layout", layout)?;
                state.serialize_field("aria_props", aria_props)?;
                state.serialize_field("aria_role", aria_role)?;
                state.serialize_field("children", children)?;
                state.serialize_field(
                    "handlers",
                    &handlers
                        .as_ref()
                        .map(|h| h.keys().cloned().collect::<Vec<String>>()),
                )?;
                state.serialize_field("sdf_shape", sdf_shape)?;
                state.end()
            }
            Self::Remove(id) => serializer.serialize_newtype_variant("VDomPatch", 2, "Remove", id),
            Self::Replace { id, node } => {
                let mut state =
                    serializer.serialize_struct_variant("VDomPatch", 3, "Replace", 2)?;
                state.serialize_field("id", id)?;
                state.serialize_field("node", node)?;
                state.end()
            }
            Self::Move { id, new_index } => {
                let mut state = serializer.serialize_struct_variant("VDomPatch", 4, "Move", 2)?;
                state.serialize_field("id", id)?;
                state.serialize_field("new_index", new_index)?;
                state.end()
            }
            Self::SetRoot(id) => {
                serializer.serialize_newtype_variant("VDomPatch", 5, "SetRoot", id)
            }
            Self::ClearHandlers { id } => {
                serializer.serialize_newtype_variant("VDomPatch", 6, "ClearHandlers", id)
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for VDomPatch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        enum VDomPatchInternal {
            Create(VNode),
            Update {
                id: NodeId,
                props: Option<HashMap<String, serde_json::Value>>,
                layout: Option<LayoutRect>,
                aria_props: Option<AriaProps>,
                aria_role: Option<String>,
                children: Option<Vec<NodeId>>,
                handlers: Option<Vec<String>>,
                sdf_shape: Option<cvkg_core::layout::SdfShape>,
            },
            Remove(NodeId),
            Replace {
                id: NodeId,
                node: VNode,
            },
            Move {
                id: NodeId,
                new_index: usize,
            },
            SetRoot(Option<NodeId>),
        }

        let internal = VDomPatchInternal::deserialize(deserializer)?;
        Ok(match internal {
            VDomPatchInternal::Create(n) => VDomPatch::Create(n),
            VDomPatchInternal::Update {
                id,
                props,
                layout,
                aria_props,
                aria_role,
                children,
                handlers,
                sdf_shape,
            } => VDomPatch::Update {
                id,
                props,
                layout,
                aria_props,
                aria_role,
                children,
                handlers: handlers.map(|keys| {
                    let mut map: EventHandlerMap = HashMap::new();
                    for key in keys {
                        // Handlers are serialized as key names only;
                        // on deserialization we create placeholder entries.
                        // The actual handler closures cannot be serialized.
                        map.insert(
                            key,
                            std::sync::Arc::new(|_| tracing::warn!("Cannot invoke serialized handler")),
                        );
                    }
                    map
                }),
                sdf_shape,
            },
            VDomPatchInternal::Remove(id) => VDomPatch::Remove(id),
            VDomPatchInternal::Replace { id, node } => VDomPatch::Replace { id, node },
            VDomPatchInternal::Move { id, new_index } => VDomPatch::Move { id, new_index },
            VDomPatchInternal::SetRoot(id) => VDomPatch::SetRoot(id),
        })
    }
}

impl VDom {
    /// Compute the difference between this VDom and another.
    ///
    /// Generates a minimal sequence of `VDomPatch` instructions to transition
    /// the host accessibility DOM from `self` to `other`.
    pub fn diff(&self, other: &VDom) -> Vec<VDomPatch> {
        let _span = tracing::info_span!("vdom_diff").entered();
        let mut patches = Vec::new();

        // Handle root changes
        match (self.root.as_ref(), other.root.as_ref()) {
            (None, None) => return patches,
            (None, Some(new_root_id)) => {
                if let Some(new_node) = other.nodes.get(new_root_id) {
                    patches.push(VDomPatch::Create(new_node.clone()));
                    patches.push(VDomPatch::SetRoot(Some(*new_root_id)));
                }
            }
            (Some(old_root_id), None) => {
                patches.push(VDomPatch::Remove(*old_root_id));
                patches.push(VDomPatch::SetRoot(None));
            }
            (Some(old_root_id), Some(new_root_id)) => {
                if old_root_id != new_root_id {
                    if let Some(new_node) = other.nodes.get(new_root_id) {
                        patches.push(VDomPatch::Replace {
                            id: *old_root_id,
                            node: new_node.clone(),
                        });
                        patches.push(VDomPatch::SetRoot(Some(*new_root_id)));
                    }
                } else {
                    self.diff_node(*old_root_id, *new_root_id, other, &mut patches);
                }
            }
        }

        patches
    }

    /// Internal node-level diff helper.
    fn diff_node(
        &self,
        old_id: NodeId,
        new_id: NodeId,
        other: &VDom,
        patches: &mut Vec<VDomPatch>,
    ) {
        let old_node = match self.nodes.get(&old_id) {
            Some(n) => n,
            None => return,
        };
        let new_node = match other.nodes.get(&new_id) {
            Some(n) => n,
            None => return,
        };

        // If components are completely different types or have different keys, replace.
        if old_node.component_type != new_node.component_type || old_node.key != new_node.key {
            patches.push(VDomPatch::Replace {
                id: old_id,
                node: new_node.clone(),
            });
            return;
        }

        // If props, layout, aria_props, or children changed, emit an Update
        let props_changed = old_node.props != new_node.props;
        let layout_changed = old_node.layout != new_node.layout;
        let aria_props_changed = old_node.aria_props != new_node.aria_props;
        let aria_role_changed = old_node.aria_role != new_node.aria_role;
        let children_changed = old_node.children != new_node.children;
        let sdf_shape_changed = old_node.sdf_shape != new_node.sdf_shape;

        // P0-7 fix: compare old vs new handler maps directly.
        let old_handlers = self.event_handlers.get(&new_id);
        let new_handlers = other.event_handlers.get(&new_id);
        let handlers_changed = match (old_handlers, new_handlers) {
            (None, None) => false,
            (Some(_), None) | (None, Some(_)) => true,
            (Some(a), Some(b)) => {
                a.len() != b.len()
                    || a.keys().any(|k| {
                        b.get(k)
                            .is_none_or(|bv| !std::sync::Arc::ptr_eq(a.get(k).unwrap(), bv))
                    })
            }
        };
        let handlers_removed = old_handlers.is_some() && new_handlers.is_none();

        if props_changed
            || layout_changed
            || aria_props_changed
            || aria_role_changed
            || children_changed
            || sdf_shape_changed
            || handlers_changed
        {
            patches.push(VDomPatch::Update {
                id: old_id,
                props: if props_changed {
                    Some(new_node.props.clone())
                } else {
                    None
                },
                layout: if layout_changed {
                    Some(new_node.layout)
                } else {
                    None
                },
                aria_props: if aria_props_changed {
                    Some(new_node.aria_props.clone())
                } else {
                    None
                },
                aria_role: if aria_role_changed {
                    Some(new_node.aria_role.clone())
                } else {
                    None
                },
                children: if children_changed {
                    Some(new_node.children.clone())
                } else {
                    None
                },
                handlers: other.event_handlers.get(&new_id).cloned(),
                sdf_shape: if sdf_shape_changed {
                    new_node.sdf_shape
                } else {
                    None
                },
            });
        }

        // P0-6 fix: emit ClearHandlers when handlers were removed.
        if handlers_removed {
            patches.push(VDomPatch::ClearHandlers { id: old_id });
        }

        // High-fidelity Keyed Child Diffing
        let old_children = &old_node.children;
        let new_children = &new_node.children;

        // 1. Map old children by key for fast lookup
        let mut old_keyed: HashMap<String, (usize, NodeId)> = HashMap::new();
        for (i, id) in old_children.iter().enumerate() {
            if let Some(node) = self.nodes.get(id)
                && let Some(key) = &node.key
            {
                old_keyed.insert(key.clone(), (i, *id));
            }
        }

        // 2. Identify moves and updates
        let mut last_index = 0;
        let mut source_indices = vec![-1; new_children.len()];
        let mut moved = false;

        for (i, new_child_id) in new_children.iter().enumerate() {
            let new_child = match other.nodes.get(new_child_id) {
                Some(n) => n,
                None => continue,
            };

            if let Some(key) = &new_child.key {
                if let Some((old_idx, old_child_id)) = old_keyed.remove(key) {
                    source_indices[i] = old_idx as i32;
                    self.diff_node(old_child_id, *new_child_id, other, patches);
                    if old_idx < last_index {
                        moved = true;
                    } else {
                        last_index = old_idx;
                    }
                } else {
                    patches.push(VDomPatch::Create(new_child.clone()));
                }
            } else if i < old_children.len() {
                self.diff_node(old_children[i], *new_child_id, other, patches);
            } else {
                patches.push(VDomPatch::Create(new_child.clone()));
            }
        }

        // 3. Apply moves using LIS to minimize mutations
        if moved {
            let lis = self.calculate_lis(&source_indices);
            let mut lis_idx = lis.len() as i32 - 1;
            for i in (0..new_children.len()).rev() {
                if source_indices[i] != -1 {
                    if lis_idx >= 0 && lis[lis_idx as usize] == i as i32 {
                        lis_idx -= 1;
                    } else {
                        patches.push(VDomPatch::Move {
                            id: new_children[i],
                            new_index: i,
                        });
                    }
                }
            }
        }

        // 4. Cleanup remaining old keyed nodes
        for (_, (_, id)) in old_keyed {
            patches.push(VDomPatch::Remove(id));
        }

        // 5. Cleanup excess unkeyed old children
        if old_children.len() > new_children.len() {
            for id in old_children.iter().skip(new_children.len()) {
                if self.nodes.get(id).is_some_and(|n| n.key.is_none()) {
                    patches.push(VDomPatch::Remove(*id));
                }
            }
        }
    }

    /// Calculate the Longest Increasing Subsequence indices
    fn calculate_lis(&self, arr: &[i32]) -> Vec<i32> {
        let n = arr.len();
        if n == 0 {
            return Vec::new();
        }

        let mut p = vec![0; n];
        let mut m = vec![0; n + 1];
        let mut l = 0;

        for i in 0..n {
            if arr[i] == -1 {
                continue;
            }

            let mut low = 1;
            let mut high = l;
            while low <= high {
                let mid = (low + high) / 2;
                if arr[m[mid] as usize] < arr[i] {
                    low = mid + 1;
                } else {
                    high = mid - 1;
                }
            }

            let new_l = low;
            p[i] = m[new_l - 1];
            m[new_l] = i as i32;

            if new_l > l {
                l = new_l;
            }
        }

        let mut res = vec![0; l];
        let mut k = m[l];
        for i in (0..l).rev() {
            res[i] = k;
            k = p[k as usize];
        }
        res
    }
}
