use crate::kvasir::{ExecutionContext, KvasirNode, ResourceId};
use crate::passes::accessibility::AccessibilityNode;
use crate::passes::backdrop_region::BackdropRegionNode;
use crate::passes::bloom::{BloomBlurNode, BloomExtractNode};
use crate::passes::composite::CompositeNode;
use crate::passes::geometry::GeometryNode;
use crate::passes::glass::{BackdropBlurNode, BackdropCopyNode, GlassNode};
use crate::passes::gbuffer::GBufferNode;
use crate::passes::ssao::SsaoNode;
use crate::passes::deferred_lighting::DeferredLightingNode;
use crate::passes::taa::TaaNode;
use crate::passes::gi::GlobalIlluminationNode;
use crate::passes::opaque3d::Opaque3dNode;
use crate::passes::pre_world_panel::PreWorldPanelNode;
use crate::passes::shadow::{DirectionalLight, GpuMesh3d, ShadowNode};
use crate::passes::skinning::SkinningNode;
use crate::passes::transparent::TransparentNode;
use crate::passes::ui::UINode;
use crate::passes::volumetric::VolumetricNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassId {
    PreWorldPanel,
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
    /// 3D shadow pass rendering depth maps.
    Shadow,
    /// 3D opaque pass rendering meshes with PBR.
    Opaque3d,
    /// Transparent 3D pass with back-to-front sorting.
    Transparent3d,
    ComputeSkinning,
    /// G-Buffer pass for deferred rendering.
    GBuffer,
    /// SSAO pass for deferred rendering.
    Ssao,
    /// Deferred lighting resolve pass.
    DeferredLighting,
    /// Temporal Anti-Aliasing pass.
    Taa,
    /// Global Illumination probe sampling pass.
    GlobalIllumination,
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

// G-Buffer resources (deferred rendering)
pub const RES_GBUFFER_ALBEDO: ResourceId = ResourceId(400);
pub const RES_GBUFFER_NORMAL: ResourceId = ResourceId(401);
pub const RES_GBUFFER_MOTION: ResourceId = ResourceId(402);

/// SSAO output resource.
pub const RES_SSAO_OUT: ResourceId = ResourceId(403);

/// Render graph configuration parameters.
pub struct RenderGraphConfig<'a> {
    pub has_glass: bool,
    pub has_bloom: bool,
    pub has_accessibility: bool,
    pub has_ibl: bool,
    /// Whether volumetric raymarching pass is active for fog/light shaft effects.
    pub has_volumetric: bool,
    /// Whether deferred rendering path is active (uses G-Buffer + lighting resolve).
    pub has_deferred: bool,
    pub active_offscreens: &'a [crate::types::OffscreenEffectConfig],
    pub portal_regions: &'a [cvkg_core::Rect],
    /// World-space UI panels that render to offscreen textures for 3D compositing.
    pub world_space_panels: &'a [(u64, cvkg_vdom::WorldSpacePanel)],
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    /// Active directional light for shadow pass (if set, shadow map is allocated).
    pub directional_light: Option<DirectionalLight>,
    /// GPU-ready 3D mesh instances for shadow map and opaque pass rendering.
    pub mesh_instances_3d: Vec<GpuMesh3d>,
    /// Transparent 3D mesh instances (sorted by view_depth for back-to-front rendering).
    pub transparent_meshes_3d: Vec<GpuMesh3d>,
    /// Cascade splits for shadow frustum division.
    pub cascade_splits: [f32; 4],
    /// Camera view projection matrix.
    pub camera_view_proj: glam::Mat4,
    /// Camera position for view_depth calculation.
    pub camera_pos: glam::Vec3,
    /// Current frame index for temporal effects.
    pub frame_index: u32,
}

/// Build the dynamic RenderGraph (KvasirGraph)
pub fn build_render_graph(config: &RenderGraphConfig<'_>) -> super::graph::KvasirGraph {
    let mut builder = super::graph::GraphBuilder::new();

    // PreWorldPanel pass: render WorldSpacePanel subtrees to offscreen textures.
    // These textures will be sampled by Geometry pass for 3D quad compositing.
    let mut panel_outputs = Vec::new();
    let mut panel_ids = Vec::new();
    for (i, panel) in config.world_space_panels.iter().enumerate() {
        let size = panel.1.texture_resolution();
        let tex_id = ResourceId(2000 + i as u32);
        panel_outputs.push(tex_id);
        panel_ids.push(panel.0);
    }

    if !panel_outputs.is_empty() {
        let pre_panel =
            builder.add_node(Box::new(PreWorldPanelNode::new(panel_outputs, panel_ids)));
        // No output connection needed - panels write to their allocated offscreen textures.
        // Geometry pass will sample them via their ResourceIds.
    }

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
            // Use resource IDs in a separate range from PreWorldPanel (which uses 2000 + i)
            // so simultaneous pre-world panels AND portal regions cannot collide on
            // the same ResourceId (would cause one to overwrite the other).
            let region_id = ResourceId(3000 + i as u32);
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

    // 3D Skinning compute pass (runs before shadow, dispatches GPU skinning for all skinned meshes)
    let skinning_node = builder.add_node(Box::new(SkinningNode {
        inputs: vec![],
        outputs: vec![],
    }));
    // SkinningNode writes to per-mesh dst_buffers; no resource connections needed.
    // It must run before shadow/opaque3d passes that read the skinned vertex data.

    // 3D Shadow pass (runs after skinning, outputs shadow map)
    if let Some(light) = &config.directional_light
        && (!config.mesh_instances_3d.is_empty() || !config.transparent_meshes_3d.is_empty())
    {
        let shadow_rid = ResourceId(10000); // dedicated shadow map resource
        let shadow_node = builder.add_node(Box::new(ShadowNode {
            light: *light,
            shadow_map: shadow_rid,
            mesh_instances: config.mesh_instances_3d.clone(),
            cascade_splits: config.cascade_splits,
            camera_view_proj: config.camera_view_proj,
        }));
        // Shadow runs after skinning — skinning writes to per-mesh dst_buffers.
        // Connect skinning → shadow to enforce deterministic ordering in topological
        // sort. Without this edge both nodes have indegree 0 and their relative order
        // depends on HashMap iteration (non-deterministic across runs), which would
        // cause shadow to sometimes read stale skinned vertex data.
        builder.connect(skinning_node, shadow_rid, shadow_node);
        // Shadow runs before scene — scene reads the shadow map.

        if config.has_deferred {
            // G-Buffer pass: writes albedo, normal, motion vectors, depth
            let gbuffer_node = builder.add_node(Box::new(GBufferNode {
                mesh_instances: config.mesh_instances_3d.clone(),
            }));
            // G-Buffer reads skinned vertex buffers — connect via a synthetic
            // gbuffer-staging resource so the topological sort deterministically
            // orders skinning before gbuffer.
            builder.connect(skinning_node, shadow_rid, gbuffer_node);

            // SSAO pass: samples depth/normal + outputs occlusion
            let ssao_node = builder.add_node(Box::new(SsaoNode {
                depth_buffer: ResourceId(0), // placeholder - depth sampled from ctx.depth_view
                normal_buffer: RES_GBUFFER_NORMAL,
                inputs: [RES_GBUFFER_NORMAL, RES_SCENE], // scene for depth view access
            }));

            // Deferred lighting: reads G-Buffer + SSAO + shadow map
            let deferred_lighting_node = builder.add_node(Box::new(DeferredLightingNode {
                ssao_occlusion: RES_SSAO_OUT,
                shadow_atlas: shadow_rid,
                scene_output: RES_SCENE,
                inputs: [RES_GBUFFER_ALBEDO, RES_GBUFFER_NORMAL],
            }));
            builder.connect(gbuffer_node, RES_GBUFFER_ALBEDO, deferred_lighting_node);
            builder.connect(gbuffer_node, RES_GBUFFER_NORMAL, deferred_lighting_node);
            builder.connect(ssao_node, RES_SSAO_OUT, deferred_lighting_node);
            builder.connect(shadow_node, shadow_rid, deferred_lighting_node);
            // Deferred lighting outputs to RES_SCENE for downstream passes
            builder.connect(deferred_lighting_node, RES_SCENE, last_scene_node);

            // Global Illumination (baked GI sampling) - runs after deferred lighting
            let _gi_node = builder.add_node(Box::new(GlobalIlluminationNode {
                volume: crate::passes::gi::IrradianceVolume {
                    origin: glam::Vec3::ZERO,
                    spacing: glam::Vec3::new(8.0, 8.0, 8.0),
                    dimensions: [8, 8, 8],
                },
                environment_probes: Vec::new(),
                lightmap: None,
                gi_uniforms: cvkg_core::GiUniforms::default(),
            }));
            // GI node has no resource connections currently (stub implementation)
            last_scene_node = deferred_lighting_node;
        } else {
            // 3D Opaque pass (runs after shadow, reads shadow map)
            let opaque_3d_node = builder.add_node(Box::new(Opaque3dNode {
                mesh_instances: config.mesh_instances_3d.clone(),
                light: *light,
                shadow_map: shadow_rid,
            }));
            builder.connect(shadow_node, shadow_rid, opaque_3d_node);
            builder.connect(opaque_3d_node, RES_SCENE, last_scene_node);
            // Opaque 3d writes to scene — update last_scene_node to chain off it.
            last_scene_node = opaque_3d_node;

            // 3D Transparent pass (runs after opaque, reads shadow map)
            // Transparent meshes must be sorted by view_depth (back-to-front)
            if !config.transparent_meshes_3d.is_empty() {
                let mut transparent_meshes = config.transparent_meshes_3d.clone();
                // Sort by view_depth descending (farthest first for back-to-front)
                transparent_meshes.sort_by(|a, b| {
                    b.view_depth
                        .partial_cmp(&a.view_depth)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                let transparent_node = builder.add_node(Box::new(TransparentNode {
                    mesh_instances: transparent_meshes,
                    shadow_map: shadow_rid,
                    camera_pos: config.camera_pos,
                }));
                builder.connect(last_scene_node, RES_SCENE, transparent_node);
                last_scene_node = transparent_node;
            }
        }
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
    // Connect from composite, not from last_scene_node, so we wait for bloom blur
    let present = builder.add_node(Box::new(PresentNode {
        inputs: vec![RES_SCENE],
        outputs: vec![],
    }));
    builder.connect(composite, RES_SCENE, present);

    builder.build()
}
