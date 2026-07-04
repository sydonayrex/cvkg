pub mod canvas;
pub mod edge;
pub mod graph;
pub mod interaction;
pub mod layout;
pub mod node;
pub mod port;
pub mod ribbon;
pub mod types;

pub use canvas::{Camera, FlowCanvas};
pub use cvkg_core::KvasirId;
pub use edge::{EdgeInteraction, FlowEdge, SplineEasing};
pub use graph::FlowGraph;
pub use layout::apply_force_directed_layout;
pub use node::{FlowNode, GlassNodeMaterial, NodeShadow, OklchColor};
pub use ribbon::{RibbonBatch, RibbonVertex, build_ribbon_batch, tessellate_bezier};

use cvkg_core::{FrameManifest, FramePhase, PassNodeDescriptor, TimeBudgetRequest};

/// Frame manifest for the flow crate.
/// Contributes: Layout phase (force-directed layout) + Render phase (ribbon tessellation).
/// Budget: 1ms Layout, 2ms Render.
pub const MANIFEST: FrameManifest = FrameManifest {
    phase_contributions: &[FramePhase::Layout, FramePhase::Render],
    pass_nodes: &[
        PassNodeDescriptor {
            id: "particle_trail",
            label: "Particle Trail Render",
            inputs: &["scene_color"],
            outputs: &["scene_color"],
            after: &["ui"],
            constructor: || -> Box<dyn cvkg_core::PassNode> {
                Box::new(ParticleTrailPass)
            },
        },
    ],
    time_budget_requests: &[
        TimeBudgetRequest {
            phase: FramePhase::Layout,
            time_slice_us: 1000,
            skippable: true,
            name: "flow_layout",
        },
        TimeBudgetRequest {
            phase: FramePhase::Render,
            time_slice_us: 2000,
            skippable: true,
            name: "flow_render",
        },
    ],
};

// Placeholder pass for particle trail render (to be implemented)
struct ParticleTrailPass;
impl cvkg_core::PassNode for ParticleTrailPass {}