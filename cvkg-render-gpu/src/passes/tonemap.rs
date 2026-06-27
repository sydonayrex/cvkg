use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::nodes::{PassId, RES_SCENE};

/// Tone mapping pass node.
/// Converts HDR scene texture to LDR output using ACES filmic tone mapping.
/// When HDR is disabled, this pass is a no-op (scene is already LDR).
pub struct ToneMapNode {
    pub inputs: Vec<crate::kvasir::resource::ResourceId>,
    pub outputs: Vec<crate::kvasir::resource::ResourceId>,
    pub target_view: Option<wgpu::TextureView>,
}

impl ToneMapNode {
    pub fn new() -> Self {
        Self {
            inputs: vec![RES_SCENE],
            outputs: vec![RES_SCENE],
            target_view: None,
        }
    }
}

impl Default for ToneMapNode {
    fn default() -> Self {
        Self::new()
    }
}

impl KvasirNode for ToneMapNode {
    fn label(&self) -> &'static str {
        "ToneMap"
    }

    fn inputs(&self) -> &[crate::kvasir::resource::ResourceId] {
        &self.inputs
    }

    fn outputs(&self) -> &[crate::kvasir::resource::ResourceId] {
        &self.outputs
    }

    fn pass_id(&self) -> PassId {
        PassId::PostProcess {
            pipeline_id: 0x544F4E45, // "TONE"
        }
    }

    fn execute(&self, _ctx: &mut ExecutionContext) {
        // Tone mapping is handled by the dedicated tonemap pipeline in end_frame.
        // This node exists to reserve the PassId slot in the render graph.
        log::trace!("[Kvasir] ToneMap: pass executed (pipeline in end_frame)");
    }
}
