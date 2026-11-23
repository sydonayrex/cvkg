use cvkg_core::{
    ActionId, ActionRegistry, FrameRenderer, KeyBinding, KeyContextStack, KeyModifiers,
    KeyShortcut, Never, Rect, Renderer, View, dispatch_key_event,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone)]
struct JourneyView {
    clicks: Arc<AtomicUsize>,
}

impl View for JourneyView {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "JourneyView");
        renderer.pop_vnode();
    }

    fn on_key_event(&self, key: &str, _modifiers: KeyModifiers) -> bool {
        if key == "Enter" {
            self.clicks.fetch_add(1, Ordering::SeqCst);
            true
        } else {
            false
        }
    }
}

#[test]
fn click_action_reaches_focused_view_and_changes_state() {
    let clicks = Arc::new(AtomicUsize::new(0));
    let view = JourneyView {
        clicks: clicks.clone(),
    };
    let rect = Rect::new(0.0, 0.0, 120.0, 40.0);

    let vdom = cvkg_vdom::VDom::build(&view, rect);
    assert!(!vdom.nodes.is_empty(), "view must produce a VDom");

    let mut renderer = cvkg_scene::test_renderer::TestRenderer::new();
    renderer.begin_frame();
    view.render(&mut renderer, rect);
    renderer.end_frame(());
    assert!(
        !renderer.commands.is_empty(),
        "view must reach the renderer"
    );

    let activate = ActionId::new("activate");
    let registry = ActionRegistry::new(vec![KeyBinding::new(
        KeyShortcut::new("Enter", "Activate"),
        activate,
        Some("canvas"),
    )]);
    let mut contexts = KeyContextStack::new();
    contexts.push("canvas");

    let dispatch = dispatch_key_event(
        &registry,
        &contexts,
        "Enter",
        KeyModifiers::default(),
        |key, modifiers| view.on_key_event(key, modifiers),
    );

    assert_eq!(dispatch.action, Some(activate));
    assert!(dispatch.consumed);
    assert_eq!(clicks.load(Ordering::SeqCst), 1);
}
