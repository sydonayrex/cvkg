//! Developer-only hot patch registry for render resources.
//!
//! This module is gated behind the `hot-patching` feature. It stores string
//! payloads with monotonic revisions so a renderer can poll for updates
//! without reloading the whole frame graph.

use std::collections::HashMap;

/// One versioned render patch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderPatch {
    pub key: &'static str,
    pub payload: String,
    pub revision: u64,
}

/// Stores the latest patch for each key and hands out new revisions.
#[derive(Debug, Default)]
pub struct PatchRegistry {
    patches: HashMap<&'static str, RenderPatch>,
    revision: u64,
}

impl PatchRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace a payload and return the new revision.
    pub fn put(&mut self, key: &'static str, payload: impl Into<String>) -> u64 {
        self.revision += 1;
        let revision = self.revision;
        self.patches.insert(
            key,
            RenderPatch {
                key,
                payload: payload.into(),
                revision,
            },
        );
        revision
    }

    pub fn get(&self, key: &str) -> Option<&RenderPatch> {
        self.patches.get(key)
    }

    /// Return every patch newer than `since`, ordered by key for determinism.
    pub fn take_updates(&self, since: u64) -> Vec<RenderPatch> {
        let mut updates: Vec<RenderPatch> = self
            .patches
            .values()
            .filter(|patch| patch.revision > since)
            .cloned()
            .collect();
        updates.sort_by_key(|patch| patch.key);
        updates
    }
}
