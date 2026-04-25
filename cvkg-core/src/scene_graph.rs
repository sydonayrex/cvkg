//! # Scene Graph and View Identity
//!
//! Manages the persistent identity of views across render frames, enabling
//! advanced features like shared-element transitions (Bifrost Bridge).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

/// Global registry for tracking shared elements across views.
pub static BIFROST_REGISTRY: OnceLock<Arc<Mutex<BifrostRegistry>>> = OnceLock::new();

/// Get or initialize the global Bifrost registry.
pub fn bifrost_registry() -> Arc<Mutex<BifrostRegistry>> {
    BIFROST_REGISTRY
        .get_or_init(|| Arc::new(Mutex::new(BifrostRegistry::new())))
        .clone()
}

/// Unique identifier for a view node in the scene graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct NodeId(u64);

static NEXT_NODE_ID: AtomicU64 = AtomicU64::new(1);

impl NodeId {
    /// Generate a new, unique node ID.
    pub fn generate() -> Self {
        Self(NEXT_NODE_ID.fetch_add(1, Ordering::SeqCst))
    }
}

/// Registry for mapping Bifrost Bridge IDs to persistent scene nodes.
#[allow(dead_code)]
pub struct BifrostRegistry {
    /// Maps Bridge ID -> Persistent Node ID
    bridges: HashMap<String, NodeId>,
    /// Maps Node ID -> Last known geometry (Rect)
    geometry_cache: HashMap<NodeId, crate::Rect>,
}

impl BifrostRegistry {
    pub fn new() -> Self {
        Self {
            bridges: HashMap::new(),
            geometry_cache: HashMap::new(),
        }
    }

    /// Register or retrieve a persistent ID for a bridge name.
    pub fn get_or_create_bridge(&mut self, bridge_id: &str) -> NodeId {
        *self
            .bridges
            .entry(bridge_id.to_string())
            .or_insert_with(NodeId::generate)
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
