//! Integration coverage for the gallery registry: every advertised component
//! must render a live preview, not just a description.

use cvkg_core::Rect;
use cvkg_core::View;
use cvkg_core::testing::MockRenderer;
use cvkg_gallery::DocsPanel;
use cvkg_gallery::state::{GalleryState, Registry};

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
