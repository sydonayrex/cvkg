//! Editable component for inline text editing.
//!
//! Provides a toggleable label that mutates into a text input field on double-click.

use crate::interactive::Input;
use crate::theme;
use cvkg_core::{Never, Rect, Renderer, Size, SizeProposal, View};
use std::sync::Arc;

/// A component that renders static text and converts into an input field on click/double-click.
#[derive(Clone)]
pub struct Editable {
    /// Static label text.
    pub(crate) text: String,
    /// Callback invoked when editing is committed/saved.
    pub(crate) on_commit: Arc<dyn Fn(String) + Send + Sync>,
    /// Unique hash for state management.
    pub(crate) state_id: u64,
}

impl Editable {
    /// Create a new Editable component.
    pub fn new(text: impl Into<String>) -> Self {
        use std::hash::{Hash, Hasher};
        let text_string = text.into();
        let mut s = std::collections::hash_map::DefaultHasher::new();
        "editable".hash(&mut s);
        text_string.hash(&mut s);
        let state_id = s.finish();

        Self {
            text: text_string,
            on_commit: Arc::new(|_| {}),
            state_id,
        }
    }

    /// Set the callback for commit/save actions.
    pub fn on_commit(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_commit = Arc::new(callback);
        self
    }
}

impl View for Editable {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Editable");

        // Read active editing state from system state
        let is_editing = cvkg_core::load_system_state()
            .get_component_state::<bool>(self.state_id)
            .and_then(|v| v.read().ok().map(|g| *g))
            .unwrap_or(false);

        if is_editing {
            let on_commit = self.on_commit.clone();
            let state_id = self.state_id;
            let input = Input::new("Edit...")
                .value(self.text.clone())
                .focused(true)
                .on_commit(move |val| {
                    (on_commit)(val);
                    // Disable edit mode on commit
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        s.set_component_state(state_id, false);
                        s
                    });
                });

            input.render(renderer, rect);
        } else {
            // Render label text with edit-hover cue
            renderer.fill_rounded_rect(rect, 4.0, [0.0, 0.0, 0.0, 0.0]); // Transparent default
            renderer.draw_text(
                &self.text,
                rect.x + 4.0,
                rect.y + (rect.height - 14.0) / 2.0,
                14.0,
                theme::text(),
            );

            // Double click handler to enter edit mode
            let state_id = self.state_id;
            renderer.register_handler(
                "pointerclick",
                Arc::new(move |_| {
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        s.set_component_state(state_id, true);
                        s
                    });
                }),
            );
        }

        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let (w, h) = renderer.measure_text(&self.text, 14.0);
        Size {
            width: proposal.width.unwrap_or(w + 16.0),
            height: h + 16.0,
        }
    }
}
