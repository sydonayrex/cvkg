//! Integration coverage for the gallery registry: every advertised component
//! must render a live preview, not just a description.

use cvkg_core::Rect;
use cvkg_core::View;
use cvkg_core::testing::MockRenderer;
use cvkg_gallery::DocsPanel;
use cvkg_gallery::state::{GalleryState, Registry};

#[test]
fn gallery_writes_go_through_the_entity_handle() {
    let handle = GalleryState::global();
    {
        let mut state = handle.lock();
        state.selected_component = Some("VirtualList".into());
    }
    assert_eq!(
        handle.read().selected_component.as_deref(),
        Some("VirtualList")
    );
}

#[test]
fn canvas_renders_from_the_entity_frame_snapshot() {
    use cvkg_core::Rect;
    use cvkg_core::View;
    use cvkg_core::testing::MockRenderer;
    use cvkg_gallery::GalleryCanvas;

    let mut selected = GalleryState::default();
    selected.selected_component = Some("Button".into());
    GalleryState::set_frame_snapshot(&selected);

    // The global mutex still has no selection. The canvas must use the snapshot.
    let canvas = GalleryCanvas::new(GalleryState::global());
    let mut renderer = MockRenderer::new();
    canvas.render(&mut renderer, Rect::new(0.0, 0.0, 800.0, 600.0));

    assert!(
        renderer
            .calls
            .iter()
            .any(|call| format!("{call:?}").contains("Button")),
        "canvas did not render the snapshot-selected component"
    );
}

#[test]
fn gallery_frame_snapshot_uses_an_entity_handle() {
    let mut state = GalleryState::default();
    state.selected_component = Some("Button".into());
    GalleryState::set_frame_snapshot(&state);

    let snapshot = GalleryState::frame_snapshot();
    assert_eq!(snapshot.selected_component.as_deref(), Some("Button"));
}

#[test]
fn button_variant_matrix_renders_every_state() {
    use cvkg_core::View;
    use cvkg_gallery::variant_matrix::button_variants;

    let variants = button_variants();
    assert!(variants.len() >= 9, "expected at least nine button states");
    for name in [
        "default",
        "hover",
        "focus",
        "active",
        "disabled",
        "loading",
        "invalid",
        "dark",
        "light",
        "small viewport",
    ] {
        assert!(
            variants.iter().any(|variant| variant.name == name),
            "{name} missing"
        );
    }

    let rect = cvkg_core::Rect::new(0.0, 0.0, 200.0, 48.0);
    for variant in variants {
        let mut renderer = cvkg_core::testing::MockRenderer::new();
        (variant.build)().render(&mut renderer, rect);
    }
}

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
    let before = state.lock().inspector_visible;
    let modifiers = KeyModifiers {
        ctrl: true,
        ..KeyModifiers::default()
    };

    assert!(app.on_key_event("i", modifiers));
    assert_ne!(state.lock().inspector_visible, before);

    app.on_key_event("i", modifiers);
    assert_eq!(state.lock().inspector_visible, before);
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
