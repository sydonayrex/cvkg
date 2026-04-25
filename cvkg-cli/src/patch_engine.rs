//! Patch Engine
//! Responsible for generating patches from compiled artifacts

use serde::{Deserialize, Serialize};

/// Compiled artifact from the build process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledArtifact {
    /// The root node ID of the view
    pub root_id: u64,
    /// The serialized view
    pub view: SerializedView,
}

/// Serialized view representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedView {
    /// The view type (e.g., "Text", "Button")
    pub view_type: String,
    /// The view properties
    pub props: serde_json::Value,
    /// The child views
    pub children: Vec<SerializedView>,
}

/// Runtime patch types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimePatch {
    /// Replace a view at the specified node ID
    ReplaceView {
        /// The node ID to replace
        node_id: u64,
        /// The new view to insert
        new_view: SerializedView,
    },
    /// Update state at the specified node ID
    UpdateState {
        /// The node ID to update
        node_id: u64,
        /// The field to update
        field: String,
        /// The new value
        value: serde_json::Value,
    },
    /// Batch multiple patches together
    Batch(Vec<RuntimePatch>),
}

/// Patch Engine implementation
/// PatchEngine — Responsible for generating atomic updates between build artifacts.
///
/// The PatchEngine diffs serialized view trees from the Muspelheim build pipeline
/// to produce minimal patches for runtime hot-reloading.
pub struct PatchEngine {
    previous_view: Option<SerializedView>,
}

impl PatchEngine {
    /// Create a new PatchEngine
    pub fn new() -> Self {
        Self {
            previous_view: None,
        }
    }

    /// Generate a patch from a compiled artifact
    pub fn generate_patch(&mut self, artifact: CompiledArtifact) -> RuntimePatch {
        let mut patches = Vec::new();

        if let Some(prev) = &self.previous_view {
            self.diff_recursive(artifact.root_id, prev, &artifact.view, &mut patches);
        } else {
            // First run, replace everything
            patches.push(RuntimePatch::ReplaceView {
                node_id: artifact.root_id,
                new_view: artifact.view.clone(),
            });
        }

        self.previous_view = Some(artifact.view);

        if patches.len() == 1 {
            patches.remove(0)
        } else {
            RuntimePatch::Batch(patches)
        }
    }

    fn diff_recursive(
        &self,
        node_id: u64,
        old: &SerializedView,
        new: &SerializedView,
        patches: &mut Vec<RuntimePatch>,
    ) {
        // If types are different, we must replace the whole subtree
        if old.view_type != new.view_type {
            patches.push(RuntimePatch::ReplaceView {
                node_id,
                new_view: new.clone(),
            });
            return;
        }

        // If props changed, we might generate UpdateState or just ReplaceView
        // For simplicity in this "real" version, we'll replace the node if anything changed
        if old.props != new.props || old.children.len() != new.children.len() {
            patches.push(RuntimePatch::ReplaceView {
                node_id,
                new_view: new.clone(),
            });
            return;
        }

        // Recursively diff children if they exist
        // Note: Without stable IDs for children in SerializedView, we use index-based matching
        for (i, (old_child, new_child)) in old.children.iter().zip(new.children.iter()).enumerate()
        {
            // We need a way to address child nodes.
            // In CVKG, we assume a deterministic ID generation based on path for dev-server patches.
            let child_id = node_id * 100 + (i as u64 + 1);
            self.diff_recursive(child_id, old_child, new_child, patches);
        }
    }
}
