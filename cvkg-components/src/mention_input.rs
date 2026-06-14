//! MentionInput component featuring autocomplete suggestions for '@' and '#' tags.
//!
//! Monitored input text to render inline tag popovers.

use crate::interactive::Input;
use crate::theme;
use crate::{FONT_BASE, RADIUS_MD, SPACE_SM};
use cvkg_core::{Never, Rect, Renderer, Size, SizeProposal, View};
use std::sync::Arc;

/// A text input component that triggers autocomplete overlays when '@' or '#' are typed.
#[derive(Clone)]
pub struct MentionInput {
    /// The current input text.
    pub(crate) text: String,
    /// List of user suggestions to match against '@'.
    pub(crate) users: Vec<String>,
    /// List of topic suggestions to match against '#'.
    pub(crate) topics: Vec<String>,
    /// Callback when value updates.
    pub(crate) on_change: Arc<dyn Fn(String) + Send + Sync>,
}

impl MentionInput {
    /// Create a new MentionInput.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            users: vec![
                "alice".to_string(),
                "bob".to_string(),
                "charlie".to_string(),
            ],
            topics: vec!["rust".to_string(), "ui".to_string(), "gpu".to_string()],
            on_change: Arc::new(|_| {}),
        }
    }

    /// Set input text value.
    pub fn value(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Set list of user suggestions.
    pub fn users(mut self, users: Vec<String>) -> Self {
        self.users = users;
        self
    }

    /// Set list of topic suggestions.
    pub fn topics(mut self, topics: Vec<String>) -> Self {
        self.topics = topics;
        self
    }

    /// Set the callback for value updates.
    pub fn on_change(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_change = Arc::new(callback);
        self
    }
}

impl View for MentionInput {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "MentionInput");

        // We check if the text ends with @ or # to trigger suggestion box
        let show_users = self.text.ends_with('@');
        let show_topics = self.text.ends_with('#');
        let suggestions_open = show_users || show_topics;

        // Render main text input
        let on_change = self.on_change.clone();
        let input = Input::new("Type here... Use @name or #topic")
            .value(self.text.clone())
            .on_change(move |val| {
                (on_change)(val);
            });

        input.render(renderer, rect);

        // Render suggestion overlay if open
        if suggestions_open {
            let items = if show_users {
                &self.users
            } else {
                &self.topics
            };
            let item_h = 32.0;
            let dropdown_h = items.len() as f32 * item_h;
            let dropdown_rect = Rect {
                x: rect.x + 8.0,
                y: rect.y + rect.height + 4.0,
                width: 180.0,
                height: dropdown_h,
            };

            renderer.set_z_index(120.0);
            renderer.fill_rounded_rect(dropdown_rect, RADIUS_MD, theme::surface_elevated());
            renderer.stroke_rounded_rect(dropdown_rect, RADIUS_MD, theme::border(), 1.0);

            for (idx, item) in items.iter().enumerate() {
                let item_rect = Rect {
                    x: dropdown_rect.x,
                    y: dropdown_rect.y + idx as f32 * item_h,
                    width: dropdown_rect.width,
                    height: item_h,
                };

                renderer.push_vnode(item_rect, "MentionOption");
                renderer.draw_text(
                    item,
                    item_rect.x + SPACE_SM,
                    item_rect.y + (item_h - FONT_BASE) / 2.0,
                    FONT_BASE,
                    theme::text(),
                );

                // On selection, append suggestion to the text
                let on_select = self.on_change.clone();
                let current_text = self.text.clone();
                let selected_val = item.clone();
                renderer.register_handler(
                    "pointerclick",
                    Arc::new(move |_| {
                        let base_text = &current_text[..current_text.len() - 1]; // strip '@' or '#'
                        let new_text = format!("{}{} ", base_text, selected_val);
                        (on_select)(new_text);
                    }),
                );

                renderer.pop_vnode();
            }
            renderer.set_z_index(0.0);
        }

        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: proposal.width.unwrap_or(240.0),
            height: 44.0,
        }
    }
}
