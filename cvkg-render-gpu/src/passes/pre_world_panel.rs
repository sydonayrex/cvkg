use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::resource::ResourceId;

/// PreWorldPanelNode renders each WorldSpacePanel's VDOM subtree to its
/// offscreen texture. The panel geometries are then submitted as textured
/// quads by the Geometry pass.
pub struct PreWorldPanelNode {
    pub panel_textures: Vec<ResourceId>,
    pub panel_ids: Vec<u64>,
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
}

impl PreWorldPanelNode {
    pub fn new(panel_textures: Vec<ResourceId>, panel_ids: Vec<u64>) -> Self {
        Self {
            panel_textures: panel_textures.clone(),
            panel_ids,
            inputs: vec![],
            outputs: panel_textures,
        }
    }
}

impl KvasirNode for PreWorldPanelNode {
    fn label(&self) -> &'static str {
        "PreWorldPanel"
    }
    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }
    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }
    fn pass_id(&self) -> crate::kvasir::nodes::PassId {
        crate::kvasir::nodes::PassId::PreWorldPanel
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        // For each panel texture, render the corresponding VDOM subtree.
        // This is a simplified implementation - the actual implementation
        // would iterate over panels, bind each offscreen texture as the
        // render target, and run the UI rendering pipeline for that panel's
        // VDOM subtree.

        for (i, &panel_tex) in self.panel_textures.iter().enumerate() {
            let panel_id = self.panel_ids[i];
            let view = match ctx.registry.get_texture_view(panel_tex) {
                Some(v) => v,
                None => {
                    tracing::error!(
                        "PreWorldPanel: missing texture view for panel {} ({})",
                        i,
                        panel_tex.0
                    );
                    continue;
                }
            };

            // Clear the offscreen texture
            let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Surtr PreWorldPanel {}", i)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // Bind UI bind groups
            pass.set_bind_group(1, &ctx.renderer.dummy_env_bind_group, &[]);
            pass.set_bind_group(2, &ctx.renderer.berserker_bind_group, &[]);
            pass.set_bind_group(3, &ctx.renderer.gradient_bind_group, &[]);

            // Filter draw_calls to only those belonging to this panel's VDOM subtree.
            if !ctx.renderer.draw_calls.is_empty() {
                pass.set_vertex_buffer(0, ctx.renderer.geometry_buffers.vertex_buffer.slice(..));
                pass.set_vertex_buffer(1, ctx.renderer.geometry_buffers.instance_buffer.slice(..));
                pass.set_index_buffer(
                    ctx.renderer.geometry_buffers.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );

                for call in &ctx.renderer.draw_calls {
                    if call.panel_id == Some(panel_id) {
                        pass.draw_indexed(
                            call.index_start..call.index_start + call.index_count,
                            0,
                            call.instance_start..call.instance_start + call.instance_count,
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod pre_world_panel_tests {
    use super::*;

    #[test]
    fn test_pre_world_panel_node_label() {
        let node = PreWorldPanelNode::new(vec![], vec![]);
        assert_eq!(node.label(), "PreWorldPanel");
    }

    #[test]
    fn test_pre_world_panel_node_empty() {
        let node = PreWorldPanelNode::new(vec![], vec![]);
        assert!(node.inputs().is_empty());
        assert!(node.outputs().is_empty());
        assert!(node.panel_textures.is_empty());
    }

    #[test]
    fn test_pre_world_panel_node_single_panel() {
        let tex = ResourceId(2000);
        let node = PreWorldPanelNode::new(vec![tex], vec![123]);
        assert_eq!(node.inputs().len(), 0);
        assert_eq!(node.outputs().len(), 1);
        assert_eq!(node.outputs()[0], tex);
        assert_eq!(node.panel_textures.len(), 1);
    }

    #[test]
    fn test_pre_world_panel_node_multiple_panels() {
        let texes = vec![ResourceId(2000), ResourceId(2001), ResourceId(2002)];
        let node = PreWorldPanelNode::new(texes.clone(), vec![1, 2, 3]);
        assert_eq!(node.outputs().len(), 3);
        assert_eq!(node.panel_textures.len(), 3);
        for (i, tex) in texes.iter().enumerate() {
            assert_eq!(node.panel_textures[i], *tex);
        }
    }

    #[test]
    fn test_pre_world_panel_node_pass_id() {
        let node = PreWorldPanelNode::new(vec![], vec![]);
        assert!(matches!(
            node.pass_id(),
            crate::kvasir::nodes::PassId::PreWorldPanel
        ));
    }
}