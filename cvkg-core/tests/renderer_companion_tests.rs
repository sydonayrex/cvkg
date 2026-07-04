use std::any::Any;
use cvkg_core::{Renderer, Companion, Rect, testing::mock_renderer::MockRenderer};

struct SinkCompanion;
impl Companion for SinkCompanion {
    fn type_name(&self) -> &'static str {
        "Sink"
    }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[test]
fn test_renderer_default_push_vnode_with_companions_ignores_companions() {
    // The default implementation must NOT panic — it delegates to push_vnode.
    let mut renderer = MockRenderer::new();
    let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    let companions: Vec<Box<dyn Companion>> = vec![Box::new(SinkCompanion)];

    // Must not panic. Companions are silently ignored by default.
    renderer.push_vnode_with_companions(rect, "test", companions);

    // Verify the vnode was still created by checking draw calls weren't affected.
    // MockRenderer doesn't track vnodes, so we just verify no panic.
}