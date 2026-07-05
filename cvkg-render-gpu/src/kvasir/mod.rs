//! Kvasir render graph module.
//!
//! Provides a retained-mode render graph with hazard detection and
//! persistent resource management.

pub mod graph;
pub mod graph_cache;
pub mod node;
pub mod nodes;
pub mod pass_registry;
pub mod planner;
pub mod registry;
pub mod resource;

pub use graph::GraphBuilder;
pub use graph_cache::CachedGraphPlan;
pub use node::{ExecutionContext, GraphId, KvasirError, KvasirNode};
pub use nodes::PassId;
pub use pass_registry::{DynKvasirNode, ErasedKvasirNode, PassRegistration, PassRegistry};
pub use registry::ResourceRegistry;
pub use resource::ResourceId;
