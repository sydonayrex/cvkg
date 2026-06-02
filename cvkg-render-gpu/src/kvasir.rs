//! Kvasir — Unified Visual Computation Graph.
//!
//! See IMPLEMENTATION-PLAN.md for the full architecture. In short:
//! - Every render operation is a `KvasirNode` with typed resource I/O.
//! - `KvasirGraph` is a DAG of nodes connected by `ResourceId` edges.
//! - `ExecutionPlanner` derives correct order, barriers, and dead-node elimination.
//! - `ResourceRegistry` tracks GPU resource lifetimes.

pub mod graph;
pub mod node;
pub mod planner;
pub mod registry;
pub mod resource;

pub use graph::{GraphBuilder, KvasirGraph, NodeKey};
pub use node::{ExecutionContext, ExecutionHint, KvasirNode};
pub use planner::ExecutionPlan;
pub use registry::ResourceRegistry;
pub use resource::{ResourceDescriptor, ResourceId, ResourceKind, ResourceLifetime};

use std::fmt;

#[derive(Debug)]
pub enum KvasirError {
    CycleDetected(Vec<NodeKey>),
    MissingInput(ResourceId, NodeKey),
    ResourceConflict {
        resource: ResourceId,
        requested: AccessMode,
        existing: AccessMode,
    },
    ExecutionFailed {
        node: &'static str,
        msg: String,
    },
}

impl fmt::Display for KvasirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CycleDetected(nodes) => write!(f, "cycle detected: {:?}", nodes),
            Self::MissingInput(res, node) => write!(f, "missing input {:?} for node {:?}", res, node),
            Self::ResourceConflict { resource, requested, existing } => {
                write!(f, "resource {:?} conflict: requested {:?}, existing {:?}", resource, requested, existing)
            }
            Self::ExecutionFailed { node, msg } => write!(f, "node '{}' failed: {}", node, msg),
        }
    }
}

impl std::error::Error for KvasirError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    Read,
    Write,
    ReadWrite,
}
