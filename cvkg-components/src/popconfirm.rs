//! Popconfirm component for lightweight inline confirmation overlays.
//!
//! Provides quick Yes/No actions relative to an anchor target.

use crate::theme;
use crate::{RADIUS_MD, SPACE_SM};
use crate::interactive::Button;
use cvkg_core::{Never, Rect, Renderer, View, SizeProposal, Size};
use std::sync::Arc;

/// A lightweight popover confirmation box featuring Yes and No actions.
#[derive(Clone)]
pub struct Popconfirm<V> {
    /// The trigger element that anchors the confirmation box.
    pub(crate) content: V,
    /// The message/question shown in the confirmation box.
    pub(crate) message: String,
    /// Callback triggered if the user confirms.
    pub(crate) on_confirm: Arc<dyn Fn() + Send + Sync>,
    /// Unique hash to manage popover open state.
    pub(crate) state_id: u64,
}

impl<V: View> Popconfirm<V> {
    /// Create a new Popconfirm with a trigger and confirmation message.
    pub fn new(content: V, message: impl Into<String>) -> Self {
        use std::hash::{Hash, Hasher};
        let msg_string = message.into();
        let mut s = std::collections::hash_map::DefaultHasher::new();
        "popconfirm".hash(&mut s);
        msg_string.hash(&mut s);
        let state_id = s.finish();

        Self {
            content,
            message: msg_string,
            on_confirm: Arc::new(|| {}),
            state_id,
        }
    }

    /// Set the callback for when the action is confirmed.
    pub fn on_confirm(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_confirm = Arc::new(callback);
        self
    }
}

impl<V: View> View for Popconfirm<V> {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Popconfirm");

        // Render the anchor trigger view
        self.content.render(renderer, rect);

        // Read active open state from system state
        let is_open = cvkg_core::load_system_state()
            .get_component_state::<bool>(self.state_id)
            .and_then(|v| v.read().ok().map(|g| *g))
            .unwrap_or(false);

        if is_open {
            let overlay_rect = Rect {
                x: rect.x - 20.0,
                y: rect.y + rect.height + 6.0,
                width: 180.0,
                height: 80.0,
            };

            renderer.set_z_index(200.0);
            renderer.fill_rounded_rect(overlay_rect, RADIUS_MD, theme::surface_elevated());
            renderer.stroke_rounded_rect(overlay_rect, RADIUS_MD, theme::border(), 1.0);

            // Draw message
            renderer.draw_text(
                &self.message,
                overlay_rect.x + SPACE_SM,
                overlay_rect.y + 8.0,
                12.0,
                theme::text(),
            );

            // Draw "Yes" Button
            let yes_rect = Rect {
                x: overlay_rect.x + SPACE_SM,
                y: overlay_rect.y + 40.0,
                width: 70.0,
                height: 28.0,
            };
            let on_confirm = self.on_confirm.clone();
            let state_id = self.state_id;
            let yes_btn = Button::new("Yes", move || {
                (on_confirm)();
                cvkg_core::update_system_state(move |s| {
                    let mut s = s.clone();
                    s.set_component_state(state_id, false);
                    s
                });
            });
            yes_btn.render(renderer, yes_rect);

            // Draw "No" Button
            let no_rect = Rect {
                x: overlay_rect.x + overlay_rect.width - 70.0 - SPACE_SM,
                y: overlay_rect.y + 40.0,
                width: 70.0,
                height: 28.0,
            };
            let no_btn = Button::new("No", move || {
                cvkg_core::update_system_state(move |s| {
                    let mut s = s.clone();
                    s.set_component_state(state_id, false);
                    s
                });
            });
            no_btn.render(renderer, no_rect);

            renderer.set_z_index(0.0);
        }

        // Click on trigger to toggle popover
        let state_id = self.state_id;
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event| {
                if let cvkg_core::Event::PointerClick { x, y, .. } = event {
                    if rect.contains(x, y) {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            let current = s
                                .get_component_state::<bool>(state_id)
                                .and_then(|v| v.read().ok().map(|g| *g))
                                .unwrap_or(false);
                            s.set_component_state(state_id, !current);
                            s
                        });
                    }
                }
            }),
        );

        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        self.content.intrinsic_size(renderer, proposal)
    }
}
