//! Kvasir — Unified Visual Computation Graph.
//!
//! See IMPLEMENTATION-PLAN.md for the full architecture. In short:
//! - Every render operation is a `KvasirNode` with typed resource I/O.
//! - `KvasirGraph` is a DAG of nodes connected by `ResourceId` edges.
//! - `ExecutionPlanner` derives correct order, barriers, and dead-node elimination.
//! - `ResourceRegistry` tracks GPU resource lifetimes.

#![allow(dead_code)]

pub mod effects;
pub mod graph;
pub mod graph_cache;
pub mod node;
pub mod nodes;
pub mod planner;
pub mod registry;
pub mod resource;

use std::fmt;

#[derive(Debug)]
pub enum KvasirError {
    CycleDetected(Vec<String>),
}

impl fmt::Display for KvasirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CycleDetected(nodes) => write!(f, "cycle detected: {:?}", nodes),
        }
    }
}

impl std::error::Error for KvasirError {}
