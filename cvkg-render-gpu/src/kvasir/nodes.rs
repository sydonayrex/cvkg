use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::resource::ResourceId;
use crate::passes::accessibility::AccessibilityNode;
use crate::passes::backdrop_region::BackdropRegionNode;
use crate::passes::bloom::{BloomBlurNode, BloomExtractNode};
use crate::passes::composite::CompositeNode;
use crate::passes::geometry::GeometryNode;
use crate::passes::glass::{BackdropBlurNode, BackdropCopyNode, GlassNode};
use crate::passes::ui::UINode;
use crate::passes::volumetric::VolumetricNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassId {
    Geometry,
    BackdropCopy,
    BackdropBlur,
    Volumetric,
    Glass,
    UI,
    Flow,
    ComputeParticle,
    BloomExtract,
    BloomBlur,
    Composite,
    Accessibility,
    Present,
    PostProcess {
        pipeline_id: u64,
    },
    /// Per-element isolated backdrop region blur.
    BackdropRegion,
}

pub struct PresentNode {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
}

impl KvasirNode for PresentNode {
    fn label(&self) -> &'static str {
        "Present"
    }
    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }
    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }
    fn pass_id(&self) -> PassId {
        PassId::Present
    }
    fn execute(&self, _ctx: &mut ExecutionContext) {
        // Presentation is handled implicitly when submitting the command buffer
    }
}

// Built-in resource constants to wire the graph
pub const RES_SCENE: ResourceId = ResourceId(1);
pub const RES_SCENE_MSAA: ResourceId = ResourceId(5);
pub const RES_BLUR_A: ResourceId = ResourceId(2);
pub const RES_BLOOM_A: ResourceId = ResourceId(3);
pub const RES_SWAPCHAIN: ResourceId = ResourceId(4);

/// Render graph configuration parameters.
pub struct RenderGraphConfig<'a> {
    pub has_glass: bool,
    pub has_bloom: bool,
    pub has_accessibility: bool,
    /// Whether volumetric raymarching pass is active for fog/light shaft effects.
    pub has_volumetric: bool,
    pub active_offscreens: &'a [crate::types::OffscreenEffectConfig],
    pub portal_regions: &'a [cvkg_core::Rect],
    pub width: u32,
    pub height: u32,
    pub scale: f32,
}

/// Build the dynamic RenderGraph (KvasirGraph)
pub fn build_render_graph(config: &RenderGraphConfig<'_>) -> super::graph::KvasirGraph {
    let mut builder = super::graph::GraphBuilder::new();

    let geometry = builder.add_node(Box::new(GeometryNode::new()));
    let mut last_scene_node = geometry;

    for offscreen in config.active_offscreens {
        let tex_id = ResourceId(1000 + (offscreen.target_id as u32));
        debug_assert!(offscreen.target_id <= u32::MAX as u64, "target_id overflow");

        let off_geom = builder.add_node(Box::new(
            crate::passes::effects::OffscreenGeometryNode::new(offscreen.target_id, tex_id),
        ));

        let composite =
            builder.add_node(Box::new(crate::passes::effects::EffectCompositeNode::new(
                offscreen.target_id,
                tex_id,
                offscreen.effect.clone(),
                offscreen.blend_mode,
                offscreen.effect_args,
            )));

        builder.connect(off_geom, tex_id, composite);
        builder.connect(last_scene_node, RES_SCENE, composite);
        last_scene_node = composite;
    }

    if config.has_glass {
        let copy = builder.add_node(Box::new(BackdropCopyNode::new()));
        builder.connect(last_scene_node, RES_SCENE, copy);

        let blur = builder.add_node(Box::new(BackdropBlurNode::new(
            config.width / 2,
            config.height / 2,
        )));
        builder.connect(copy, RES_BLUR_A, blur);

        // Per-element backdrop blur for portal-aware glass elements
        for (i, region) in config.portal_regions.iter().enumerate() {
            let region_id = ResourceId(2000 + i as u32);
            let region_node =
                builder.add_node(Box::new(BackdropRegionNode::new(*region, region_id)));
            builder.connect(last_scene_node, RES_SCENE, region_node);
        }

        let glass = builder.add_node(Box::new(GlassNode::new(config.scale)));
        builder.connect(blur, RES_BLUR_A, glass);
        builder.connect(last_scene_node, RES_SCENE, glass);
        last_scene_node = glass;
    }

    let ui = builder.add_node(Box::new(UINode::new()));
    builder.connect(last_scene_node, RES_SCENE, ui);
    last_scene_node = ui;

    // Volumetric raymarching (conditional, for fog/light shaft effects)
    let has_volumetric = config.has_volumetric;
    if has_volumetric {
        let volumetric = builder.add_node(Box::new(VolumetricNode::new()));
        builder.connect(last_scene_node, RES_SCENE, volumetric);
        last_scene_node = volumetric;
    }

    // Bloom extraction and blur (conditional)
    let mut last_bloom_node = None;
    if config.has_bloom {
        let extract = builder.add_node(Box::new(BloomExtractNode::new()));
        builder.connect(last_scene_node, RES_SCENE, extract);

        let blur = builder.add_node(Box::new(BloomBlurNode::new(
            config.width / 2,
            config.height / 2,
        )));
        builder.connect(extract, RES_BLOOM_A, blur);
        last_bloom_node = Some(blur);
    }

    // Accessibility transform (conditional, runs before final composite)
    if config.has_accessibility {
        let a11y = builder.add_node(Box::new(AccessibilityNode::new()));
        builder.connect(last_scene_node, RES_SCENE, a11y);
        // Accessibility writes back to RES_SCENE for the composite to consume
        last_scene_node = a11y;
    }

    // Final composite: blends scene + bloom onto the swapchain target.
    // If accessibility ran, it already cleared the swapchain, so we load.
    // If accessibility did NOT run, we need to clear first.
    let composite = builder.add_node(Box::new(CompositeNode::new(
        config.has_bloom,
        !config.has_accessibility,
    )));
    builder.connect(last_scene_node, RES_SCENE, composite);
    if let Some(bloom_node) = last_bloom_node {
        builder.connect(bloom_node, RES_BLOOM_A, composite);
    }

    // Present node marks the graph endpoint (presentation is handled by Surface::present)
    let present = builder.add_node(Box::new(PresentNode {
        inputs: vec![RES_SCENE],
        outputs: vec![],
    }));
    builder.connect(last_scene_node, RES_SCENE, present);

    builder.build()
}
