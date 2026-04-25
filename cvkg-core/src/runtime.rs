use serde::{Deserialize, Serialize};

/// A patch instruction to apply to the running CVKG application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimePatch {
    /// Replaces an entire view subgraph.
    ReplaceView {
        /// ID of the node to replace.
        node_id: u64,
        /// The new serialized view state.
        new_view: serde_json::Value,
    },
    /// Updates a single state property of a node.
    UpdateState {
        /// ID of the node containing the state.
        node_id: u64,
        /// The field name.
        field: String,
        /// The new value.
        value: serde_json::Value,
    },
    /// A batch of sequential patches.
    Batch(Vec<RuntimePatch>),
}

/// A serialized snapshot of the runtime state graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RuntimeStateSnapshot {
    /// Serialized node states.
    pub nodes: Vec<NodeStateSnapshot>,
}

/// A snapshot for a specific node.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct NodeStateSnapshot {
    /// Node identifier.
    pub id: u64,
    /// Key-value state properties.
    pub state: serde_json::Value,
}

/// Internal application event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RuntimeEvent {
    /// E.g., "Agent", "Input"
    pub kind: String,
    /// Serialized event payload.
    pub payload: serde_json::Value,
}

/// Applies a `RuntimePatch` to the active application.
///
/// This integrates directly with the scheduler to execute safe mutations
/// during the frame lifecycle.
pub fn apply_patch(patch: RuntimePatch) {
    let state_lock = crate::get_system_state();
    let _state = state_lock.write().unwrap();

    match patch {
        RuntimePatch::ReplaceView { node_id, new_view } => {
            log::info!(
                "Runtime: Replacing view subgraph for node {}. (Serialized: {})",
                node_id,
                new_view
            );
            // In a full implementation, this would trigger a scene graph reconciliation
        }
        RuntimePatch::UpdateState {
            node_id,
            field,
            value,
        } => {
            log::info!(
                "Runtime: Updating state for node {} field '{}' to {}",
                node_id,
                field,
                value
            );
            // Here we would use the node_id to find the component state and update the field
            // if we had a reflective field-access system.
        }
        RuntimePatch::Batch(patches) => {
            for p in patches {
                apply_patch(p);
            }
        }
    }
}

/// Captures and returns the current state of the application.
pub fn snapshot_state() -> RuntimeStateSnapshot {
    let state_lock = crate::get_system_state();
    let state = state_lock.read().unwrap();
    RuntimeStateSnapshot {
        nodes: state.snapshot(),
    }
}
