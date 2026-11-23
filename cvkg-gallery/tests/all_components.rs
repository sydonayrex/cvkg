//! Integration coverage for the gallery registry: every advertised component
//! must render a live preview, not just a description.

use cvkg_core::Rect;
use cvkg_core::View;
use cvkg_core::testing::MockRenderer;
use cvkg_gallery::DocsPanel;
use cvkg_gallery::state::{GalleryState, Registry};

#[test]
fn gallery_persists_docking_layout() {
    use cvkg_core::docking::{Floating, Layout, Node};

    let mut state = GalleryState::default();
    state.docking_layout = Layout {
        root: Node::Floating(Floating {
            panel_id: "logs".into(),
            x: 4.0,
            y: 8.0,
            width: 160.0,
            height: 90.0,
        }),
    };
    state.save_persisted();

    let mut restored = GalleryState::default();
    restored.persisted_docking_layout = state.persisted_docking_layout.clone();
    restored.load_persisted();

    assert_eq!(restored.docking_layout.node_count(), 1);
    assert_eq!(restored.docking_layout, state.docking_layout);
}

#[test]
fn inspector_toggles_from_the_gallery_key_dispatcher() {
    use cvkg_core::{KeyModifiers, View};
    use cvkg_gallery::GalleryApp;

    let app = GalleryApp::new();
    let state = GalleryState::global();
    let before = state.lock().unwrap().inspector_visible;
    let modifiers = KeyModifiers {
        ctrl: true,
        ..KeyModifiers::default()
    };

    assert!(app.on_key_event("i", modifiers));
    assert_ne!(state.lock().unwrap().inspector_visible, before);

    app.on_key_event("i", modifiers);
    assert_eq!(state.lock().unwrap().inspector_visible, before);
}

#[test]
fn inspector_returns_role_label_and_rect() {
    use cvkg_components::Button;
    use cvkg_core::{AnyView, Rect};
    use cvkg_gallery::inspector::{InspectorNode, inspect_view};

    let view = AnyView::new(Button::new("Inspect", || {}));
    let rect = Rect::new(4.0, 8.0, 120.0, 32.0);
    let nodes: Vec<InspectorNode> = inspect_view(&view, rect);

    assert!(
        nodes
            .iter()
            .any(|node| node.role == "button" && node.label == "Inspect" && node.rect == rect),
        "expected button node with role, label and rect"
    );
}

#[test]
fn every_registered_component_has_generated_docs_summary() {
    for meta in Registry::all() {
        assert!(
            DocsPanel::has_generated_summary(meta.name),
            "{} missing generated docs summary",
            meta.name
        );
    }
}

#[test]
fn every_registered_component_renders_a_live_preview() {
    let components = Registry::all();
    assert!(
        components.len() >= 29,
        "gallery registry shrank to {} components",
        components.len()
    );

    let state = GalleryState::default();
    let rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    let mut empty = Vec::new();

    for meta in components {
        let view = (meta.factory)(&state);
        let mut renderer = MockRenderer::new();
        view.render(&mut renderer, rect);
        // A description-only placeholder produces no draw calls. Every live
        // preview must issue at least one primitive plus its vnode boundary.
        if renderer.calls.is_empty() {
            empty.push(meta.name);
        }
    }

    assert!(
        empty.is_empty(),
        "components with no live preview draw calls: {empty:?}"
    );

    let undocumented: Vec<_> = Registry::all()
        .iter()
        .map(|meta| meta.name)
        .filter(|name| !DocsPanel::has_docs(name))
        .collect();
    assert!(
        undocumented.is_empty(),
        "components with no props/usage docs: {undocumented:?}"
    );
}
