use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::nodes::{PassId, RES_SCENE};
use crate::kvasir::resource::ResourceId;

pub struct FlowRenderNode {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
}

impl FlowRenderNode {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            inputs: vec![RES_SCENE],
            outputs: vec![RES_SCENE],
        }
    }
}

impl KvasirNode for FlowRenderNode {
    fn label(&self) -> &'static str {
        "FlowRender"
    }
    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }
    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }
    fn pass_id(&self) -> PassId {
        PassId::Flow
    }
    fn execute(&self, _ctx: &mut ExecutionContext) {
        // Flow rendering requires ribbon batch data from the cvkg-flow crate.
        // When flow data is available, this pass binds the flow_pipeline and
        // draws ribbon meshes with animated pulse effects.
        //
        // For now, this is a no-op placeholder. The flow.wgsl shader is
        // production-ready and will be wired in when the flow compositor
        // integration is complete.
    }
}