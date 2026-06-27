//! # Scene Graph and View Identity
//!
//! Manages the persistent identity of views across render frames, enabling
//! advanced features like shared-element transitions (Bifrost Bridge).

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

pub type NodeId = super::KvasirId;

/// Registry for mapping Bifrost Bridge IDs to persistent scene nodes.
pub struct BifrostRegistry {
    /// Maps Bridge ID -> Persistent Node ID
    bridges: HashMap<String, NodeId>,
    /// Maps Node ID -> Last known geometry (Rect)
    geometry_cache: HashMap<NodeId, crate::Rect>,
}

impl Default for BifrostRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl BifrostRegistry {
    pub fn new() -> Self {
        Self {
            bridges: HashMap::new(),
            geometry_cache: HashMap::new(),
        }
    }

    /// Get or create a persistent ID for a bridge name.
    pub fn get_or_create_bridge(&mut self, bridge_id: &str) -> NodeId {
        #[allow(clippy::unwrap_or_default)]
        *self
            .bridges
            .entry(bridge_id.to_string())
            .or_insert_with(super::KvasirId::new)
    }

    /// Store the geometry of a node for interpolation in the next frame.
    pub fn update_geometry(&mut self, node_id: NodeId, rect: crate::Rect) {
        self.geometry_cache.insert(node_id, rect);
    }

    /// Retrieve the last known geometry for a node.
    pub fn get_geometry(&self, node_id: NodeId) -> Option<crate::Rect> {
        self.geometry_cache.get(&node_id).copied()
    }
}

/// Get the global Bifrost registry instance.
pub fn bifrost_registry() -> std::sync::MutexGuard<'static, BifrostRegistry> {
    static REGISTRY: Lazy<Mutex<BifrostRegistry>> =
        Lazy::new(|| Mutex::new(BifrostRegistry::new()));
    REGISTRY.lock().unwrap()
}
