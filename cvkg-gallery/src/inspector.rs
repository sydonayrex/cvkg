//! Accessibility element inspector for the gallery.
//!
//! The inspector flattens the companion states published by a view into a list
//! of role, label, and rect rows. It reads the same data the VDOM uses, so the
//! overlay cannot drift from the accessibility tree.

use cvkg_core::{A11yCompanion, AnyView, Rect, Renderer, View};

/// One accessible element in the inspector list.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorNode {
    pub role: String,
    pub label: String,
    pub rect: Rect,
}

/// Flatten the accessibility companions of `view` into inspector rows.
pub fn inspect_view(view: &AnyView, rect: Rect) -> Vec<InspectorNode> {
    view.companion_states()
        .into_iter()
        .filter_map(|companion| {
            let a11y = companion.as_any().downcast_ref::<A11yCompanion>()?;
            Some(InspectorNode {
                role: a11y.role.clone(),
                label: a11y.label.clone(),
                rect,
            })
        })
        .collect()
}

/// Lightweight overlay that lists accessible elements.
#[derive(Clone, Debug)]
pub struct InspectorOverlay {
    nodes: Vec<InspectorNode>,
}

impl InspectorOverlay {
    pub fn new(nodes: Vec<InspectorNode>) -> Self {
        Self { nodes }
    }

    pub fn nodes(&self) -> &[InspectorNode] {
        &self.nodes
    }
}

impl View for InspectorOverlay {
    type Body = cvkg_core::Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rect(rect, [0.05, 0.05, 0.08, 0.85]);
        let mut y = rect.y + 8.0;
        for node in &self.nodes {
            let line = format!(
                "{}: {} @ ({:.0}, {:.0}, {:.0}, {:.0})",
                node.role, node.label, node.rect.x, node.rect.y, node.rect.width, node.rect.height
            );
            renderer.draw_text_raw(&line, rect.x + 8.0, y, 12.0, [1.0, 1.0, 1.0, 1.0]);
            y += 16.0;
        }
    }
}
