//! Kvasir — Unified Visual Computation Graph
//!
//! Kvasir replaces the hardcoded pass orchestration in `end_frame()` with a
//! dependency-driven render graph. Every rendering operation is a typed node
//! with declared input/output resources. The execution planner derives correct
//! barrier insertion, dead-node elimination, and pass ordering automatically.

// ── Submodules ──────────────────────────────────────────────────────────────

pub mod graph;
pub mod node;
pub mod planner;
pub mod registry;
pub mod resource;

// ── Re-exports ──────────────────────────────────────────────────────────────

pub use graph::{ExecutionPlan, KvasirGraph, NodeKey};
pub use node::{ExecutionContext, ExecutionHint, KvasirNode};
pub use planner::ExecutionPlanner;
pub use registry::ResourceRegistry;
pub use resource::{ResourceDescriptor, ResourceId, ResourceKind, ResourceLifetime};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvasirError {
    #[error("cycle detected in render graph: {0:?}")]
    CycleDetected(Vec<NodeKey>),
    #[error("missing input resource {0:?} for node {1:?}")]
    MissingInput(ResourceId, NodeKey),
    #[error("resource {0:?} conflict: node requires {1} but existing access is {2}")]
    ResourceConflict {
        resource: ResourceId,
        requested: AccessMode,
        existing: AccessMode,
    },
    #[error("node '{node}' execution failed: {source}")]
    ExecutionFailed {
        node: &'static str,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    Read,
    Write,
    ReadWrite,
}
