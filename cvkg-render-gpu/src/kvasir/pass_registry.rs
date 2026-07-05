//! Render Pass Self-Registration (Bevy-inspired plugin-pattern).
//!
//! Each crate declares its render passes here instead of hard-coding them in
//! the umbrella `cvkg` crate. Populated at startup; each crate calls
//! `register()` from a well-known entry point.
//!
//! ## Relationship to FrameManifest (§14)
//! The compile-time `FrameManifest::merge()` is the preferred path for
//! production code. This registry is a stepping stone — useful for testing
//! and for crates that cannot use the `const fn` merge.

use crate::kvasir::node::ExecutionContext;
use crate::kvasir::resource::ResourceId;
use std::collections::HashMap;

/// Boxed dyn erased-node used by `PassRegistration::constructor`.
pub type DynKvasirNode = Box<dyn ErasedKvasirNode>;

/// Object-safe erased node trait for use through the registry.
///
/// We do not require `Send + Sync` here so the constructor's `Box<...>` can
/// be `static`. The runtime `KvasirNode` (cfg-gated) carries those bounds.
pub trait ErasedKvasirNode {
    /// Human-readable debug label.
    fn label(&self) -> &'static str;
    /// Input resource ids.
    fn inputs(&self) -> &[ResourceId];
    /// Output resource ids.
    fn outputs(&self) -> &[ResourceId];
    /// Pass id for graph traversal.
    fn pass_id(&self) -> super::nodes::PassId;
    /// Execute the pass against an `ExecutionContext`.
    fn execute(&self, ctx: &mut ExecutionContext);
}

/// A self-registering render pass descriptor.
///
/// Each crate defines its passes here instead of hard-coding them in the
/// umbrella `cvkg` crate.
pub struct PassRegistration {
    /// Stable string identifier (e.g. "particle_trail").
    pub id: &'static str,
    /// Human-readable label (e.g. "Particle Trail").
    pub label: &'static str,
    /// Resource ids read by this pass.
    pub inputs: &'static [&'static str],
    /// Resource ids produced by this pass.
    pub outputs: &'static [&'static str],
    /// Pass ids this pass must run after, for ordering.
    pub after: &'static [&'static str],
    /// Constructor returning an erased node.
    pub constructor: fn() -> DynKvasirNode,
}

/// Registry that collects pass registrations from all crates.
pub struct PassRegistry {
    passes: Vec<PassRegistration>,
}

impl Default for PassRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PassRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    /// Register a pass. Duplicate ids panic — caught at startup, not in production.
    pub fn register(&mut self, pass: PassRegistration) {
        if self.passes.iter().any(|p| p.id == pass.id) {
            panic!("PassRegistry: duplicate pass id `{}`", pass.id);
        }
        self.passes.push(pass);
    }

    /// Number of registered passes.
    pub fn len(&self) -> usize {
        self.passes.len()
    }

    /// Returns whether the registry has no passes.
    pub fn is_empty(&self) -> bool {
        self.passes.is_empty()
    }

    /// Iterate over registrations.
    pub fn iter(&self) -> impl Iterator<Item = &PassRegistration> {
        self.passes.iter()
    }

    /// Look up a pass by id.
    pub fn get(&self, id: &str) -> Option<&PassRegistration> {
        self.passes.iter().find(|p| p.id == id)
    }

    /// Topological ordering based on `.after` constraints.
    ///
    /// Falls back to insertion order when no constraints apply. Returns
    /// `Err(_)` if a cycle is detected or an unknown dependency is referenced.
    pub fn topo_sort(&self) -> Result<Vec<&'static str>, String> {
        let ids: Vec<&'static str> = self.passes.iter().map(|p| p.id).collect();
        let mut indegree: HashMap<&'static str, usize> = ids.iter().map(|&id| (id, 0)).collect();
        let mut graph: HashMap<&'static str, Vec<&'static str>> = HashMap::new();

        for pass in &self.passes {
            for after in pass.after {
                if !indegree.contains_key(after) {
                    return Err(format!(
                        "PassRegistry: unknown dependency `{}` referenced by `{}`",
                        after, pass.id
                    ));
                }
                graph.entry(after).or_default().push(pass.id);
                *indegree.entry(pass.id).or_insert(0) += 1;
            }
        }

        let mut queue: std::collections::VecDeque<&'static str> = std::collections::VecDeque::new();
        for &id in &ids {
            if indegree.get(id).copied().unwrap_or(0) == 0 {
                queue.push_back(id);
            }
        }
        let mut result = Vec::new();

        while let Some(node) = queue.pop_front() {
            result.push(node);
            if let Some(edges) = graph.get(node) {
                let mut ordered_edges: Vec<&'static str> = edges.clone();
                ordered_edges.sort_by_key(|id| ids.iter().position(|x| x == id).unwrap_or(0));
                for &next in &ordered_edges {
                    if let Some(d) = indegree.get_mut(next) {
                        *d -= 1;
                        if *d == 0 {
                            queue.push_back(next);
                        }
                    }
                }
            }
        }

        if result.len() == ids.len() {
            Ok(result)
        } else {
            Err("PassRegistry: cycle detected".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kvasir::nodes::PassId;

    struct DummyNode(&'static str);
    impl crate::kvasir::node::KvasirNode for DummyNode {
        fn label(&self) -> &'static str {
            self.0
        }
        fn inputs(&self) -> &[ResourceId] {
            &[]
        }
        fn outputs(&self) -> &[ResourceId] {
            &[]
        }
        fn pass_id(&self) -> PassId {
            PassId::Composite
        }
        fn execute(&self, _ctx: &mut ExecutionContext) {}
    }
    impl ErasedKvasirNode for DummyNode {
        fn label(&self) -> &'static str {
            self.0
        }
        fn inputs(&self) -> &[ResourceId] {
            &[]
        }
        fn outputs(&self) -> &[ResourceId] {
            &[]
        }
        fn pass_id(&self) -> PassId {
            PassId::Composite
        }
        fn execute(&self, _ctx: &mut ExecutionContext) {}
    }

    fn make_pass(id: &'static str, after: &'static [&'static str]) -> PassRegistration {
        PassRegistration {
            id,
            label: id,
            inputs: &[],
            outputs: &[],
            after,
            constructor: || Box::new(DummyNode("dummy")) as DynKvasirNode,
        }
    }

    #[test]
    fn test_empty_registry() {
        let reg = PassRegistry::new();
        assert_eq!(reg.len(), 0);
        assert!(reg.is_empty());
    }

    #[test]
    fn test_register_passes() {
        let mut reg = PassRegistry::new();
        reg.register(make_pass("a", &[]));
        reg.register(make_pass("b", &[]));
        assert_eq!(reg.len(), 2);
    }

    #[test]
    fn test_lookup_by_id() {
        let mut reg = PassRegistry::new();
        reg.register(make_pass("geometry", &[]));
        reg.register(make_pass("ui", &[]));
        assert!(reg.get("geometry").is_some());
        assert!(reg.get("missing").is_none());
    }

    #[test]
    fn test_iter() {
        let mut reg = PassRegistry::new();
        reg.register(make_pass("a", &[]));
        reg.register(make_pass("b", &[]));
        let ids: Vec<&str> = reg.iter().map(|p| p.id).collect();
        assert_eq!(ids, vec!["a", "b"]);
    }

    #[test]
    fn test_topological_order() {
        let mut reg = PassRegistry::new();
        reg.register(make_pass("ui", &["geometry"]));
        reg.register(make_pass("composite", &["ui"]));
        reg.register(make_pass("geometry", &[]));

        let order = reg.topo_sort().expect("no cycle");
        let pos = |id: &str| order.iter().position(|&x| x == id).unwrap();
        assert!(pos("geometry") < pos("ui"));
        assert!(pos("ui") < pos("composite"));
    }

    #[test]
    fn test_topological_no_constraints_preserves_insertion_order() {
        let mut reg = PassRegistry::new();
        reg.register(make_pass("a", &[]));
        reg.register(make_pass("b", &[]));
        reg.register(make_pass("c", &[]));

        let order = reg.topo_sort().unwrap();
        assert_eq!(order, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_topological_unknown_dependency_errors() {
        let mut reg = PassRegistry::new();
        reg.register(make_pass("ui", &["ghost"]));
        let err = reg.topo_sort().unwrap_err();
        assert!(err.contains("ghost"));
    }

    #[test]
    #[should_panic]
    fn test_duplicate_registration_panics() {
        let mut reg = PassRegistry::new();
        reg.register(make_pass("dup", &[]));
        reg.register(make_pass("dup", &[]));
    }
}
