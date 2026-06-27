use crate::theme;
use crate::{FONT_BASE, RADIUS_MD, RADIUS_SM};
use cvkg_core::{Event, Never, Rect, Renderer, View};
use std::sync::Arc;

/// System-state hash key for the dropdown open/close state.
const DROPDOWN_OPEN_HASH: u64 = 0xD00_0001;

/// An item in a dropdown menu.
#[derive(Clone)]
pub struct DropdownItem {
    /// The label text displayed for this item.
    pub label: String,
    /// Callback fired when the item is clicked.
    pub on_click: Arc<dyn Fn() + Send + Sync>,
    /// Optional icon text rendered to the left of the label.
    pub icon: Option<String>,
    /// Optional shortcut text rendered to the right (e.g. "⌘C").
    pub shortcut: Option<String>,
    /// If true, renders a divider line instead of a clickable row.
    pub is_divider: bool,
}

impl DropdownItem {
    /// Creates a new clickable dropdown item with the given label and callback.
    pub fn new(label: impl Into<String>, on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            label: label.into(),
            on_click: Arc::new(on_click),
            icon: None,
            shortcut: None,
            is_divider: false,
        }
    }

    /// Sets an optional icon (rendered to the left of the label).
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets optional shortcut text (rendered to the right, e.g. "⌘C").
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Marks this item as a divider line.
    pub fn divider(mut self) -> Self {
        self.is_divider = true;
        self
    }
}

/// A dropdown menu that opens below a trigger view.
pub struct DropdownMenu<T: View> {
    /// The trigger view. Clicking it toggles the dropdown.
    pub trigger: T,
    /// The items to display in the dropdown.
    pub items: Vec<DropdownItem>,
    /// Whether the dropdown is currently open.
    pub is_open: bool,
}

impl<T: View> DropdownMenu<T> {
    /// Creates a new DropdownMenu with the given trigger and items.
    pub fn new(trigger: T, items: Vec<DropdownItem>) -> Self {
        Self {
            trigger,
            items,
            is_open: false,
        }
    }

    /// Sets the open state.
    pub fn is_open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }
}

impl<T: View> View for DropdownMenu<T> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Render the trigger inline at the given rect
        self.trigger.render(renderer, rect);

        // Toggle open/close on trigger click
        let trigger_rect = rect;
        renderer.register_handler(
            "pointerdown",
            Arc::new(move |event| {
                if let Event::PointerDown { x, y, .. } = event
                    && trigger_rect.contains(x, y)
                {
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        let current: bool = s
                            .get_component_state::<bool>(DROPDOWN_OPEN_HASH)
                            .and_then(|v| v.read().ok().map(|g| *g))
                            .unwrap_or(false);
                        s.set_component_state(DROPDOWN_OPEN_HASH, !current);
                        s
                    });
                }
            }),
        );

        // Read open state from system state
        let open: bool = cvkg_core::load_system_state()
            .get_component_state::<bool>(DROPDOWN_OPEN_HASH)
            .and_then(|v| v.read().ok().map(|v| *v))
            .unwrap_or(self.is_open);

        if !open {
            return;
        }

        // ── Dropdown panel positioned below the trigger ──
        let item_height = 32.0;
        let divider_height = 8.0;
        let panel_width: f32 = 200.0;
        let panel_height: f32 = self
            .items
            .iter()
            .map(|item| {
                if item.is_divider {
                    divider_height
                } else {
                    item_height
                }
            })
            .sum::<f32>()
            + 16.0; // padding

        let panel_rect = Rect {
            x: rect.x,
            y: rect.y + rect.height + 4.0,
            width: panel_width,
            height: panel_height,
        };

        // Panel background
        if crate::theme::glassmorphism_enabled() {
            renderer.bifrost(panel_rect, 12.0, 1.2, 0.9);
        }
        renderer.fill_rounded_rect(
            panel_rect,
            RADIUS_MD,
            theme::with_alpha(theme::surface_elevated(), 0.95),
        );
        renderer.stroke_rounded_rect(
            panel_rect,
            RADIUS_MD,
            theme::with_alpha(theme::accent(), 0.3),
            1.0,
        );

        // System-state hash for hovered item index
        let hover_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            "dropdown_hover_idx".hash(&mut s);
            s.finish()
        };

        // Render items
        let mut current_y = panel_rect.y + 8.0;
        for (idx, item) in self.items.iter().enumerate() {
            if item.is_divider {
                // Divider line
                let divider_y = current_y + divider_height / 2.0;
                renderer.draw_line(
                    panel_rect.x + 8.0,
                    divider_y,
                    panel_rect.x + panel_rect.width - 8.0,
                    divider_y,
                    [0.2, 0.2, 0.25, 0.6],
                    1.0,
                );
                current_y += divider_height;
                continue;
            }

            let item_rect = Rect {
                x: panel_rect.x + 4.0,
                y: current_y,
                width: panel_rect.width - 8.0,
                height: item_height,
            };

            // Read hover state
            let hovered: bool = cvkg_core::load_system_state()
                .get_component_state::<usize>(hover_hash)
                .and_then(|v| v.read().ok().map(|v| *v))
                .map(|h| h == idx)
                .unwrap_or(false);

            // Hover background
            if hovered {
                renderer.fill_rounded_rect(
                    item_rect,
                    RADIUS_SM,
                    theme::with_alpha(theme::primary(), 0.15),
                );
            }

            let mut text_x = item_rect.x + 8.0;

            // Icon
            if let Some(ref icon) = item.icon {
                renderer.draw_text(icon, text_x, item_rect.y + 8.0, 12.0, theme::text_muted());
                text_x += 20.0;
            }

            // Label
            renderer.draw_text(
                &item.label,
                text_x,
                item_rect.y + 8.0,
                FONT_BASE,
                theme::text(),
            );

            // Shortcut text (right-aligned)
            if let Some(ref shortcut) = item.shortcut {
                renderer.draw_text(
                    shortcut,
                    item_rect.x + item_rect.width - 50.0,
                    item_rect.y + 8.0,
                    12.0,
                    theme::text_muted(),
                );
            }

            // Hover detection
            let item_rect_capture = item_rect;
            let item_idx = idx;
            renderer.register_handler(
                "pointermove",
                Arc::new(move |event| {
                    if let Event::PointerMove { x, y, .. } = event
                        && item_rect_capture.contains(x, y)
                    {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            s.set_component_state(hover_hash, item_idx);
                            s
                        });
                    }
                }),
            );

            // Click handler: fires on_click and closes dropdown
            let on_click = item.on_click.clone();
            let item_rect_click = item_rect;
            renderer.register_handler(
                "pointerdown",
                Arc::new(move |event| {
                    if let Event::PointerDown { x, y, .. } = event
                        && item_rect_click.contains(x, y)
                    {
                        (on_click)();
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            s.set_component_state(DROPDOWN_OPEN_HASH, false);
                            s
                        });
                    }
                }),
            );

            current_y += item_height;
        }

        // Click outside closes the dropdown
        let panel = panel_rect;
        let trigger = rect;
        renderer.register_handler(
            "pointerdown",
            Arc::new(move |event| {
                if let Event::PointerDown { x, y, .. } = event
                    && !panel.contains(x, y)
                    && !trigger.contains(x, y)
                {
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        s.set_component_state(DROPDOWN_OPEN_HASH, false);
                        s
                    });
                }
            }),
        );
    }
}
