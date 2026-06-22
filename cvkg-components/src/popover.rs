use crate::theme;
use cvkg_core::{Event, Never, Rect, Renderer, View, load_system_state, update_system_state};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for generating unique popover instance IDs.
static POPOVER_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Popover position relative to the trigger element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopoverPosition {
    /// Popover appears above the trigger.
    Top,
    /// Popover appears below the trigger.
    Bottom,
    /// Popover appears to the left of the trigger.
    Left,
    /// Popover appears to the right of the trigger.
    Right,
    /// Automatically choose the best position based on available space.
    #[default]
    Auto,
}

/// Popover - A floating content container anchored to a trigger element.
/// Renders the trigger view inline. When the trigger is clicked, the content
/// view is displayed in a positioned, glassmorphic floating panel with an
/// arrow pointing at the trigger. Clicking the trigger again or clicking
/// outside closes it.
/// # Type Parameters
/// * `V` - The trigger view type.
/// * `C` - The content view type.
/// # Examples
/// ```
/// use cvkg_components::Text;
/// use cvkg_components::popover::{Popover, PopoverPosition};
/// let popover = Popover::new(
///     Text::new("Click me"),
///     Text::new("Hello from popover!"),
/// )
/// .position(PopoverPosition::Bottom)
/// .open(true);
/// ```
pub struct Popover<V: View, C: View> {
    /// The trigger element that toggles the popover.
    trigger: V,
    /// The floating content displayed when the popover is open.
    content: C,
    /// Whether the popover is currently displayed.
    is_open: bool,
    /// Preferred position of the popover relative to the trigger.
    position: PopoverPosition,
    /// Stable per-instance hash used to identify this component in the system state store.
    id_hash: u64,
}

impl<V: View, C: View> Popover<V, C> {
    /// Create a new Popover with the given trigger and content views.
    ///
    /// The popover defaults to closed (`is_open: false`) and `Auto` positioning.
    pub fn new(trigger: V, content: C) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let counter = POPOVER_COUNTER.fetch_add(1, Ordering::SeqCst);
        counter.hash(&mut hasher);
        std::any::type_name::<V>().hash(&mut hasher);
        std::any::type_name::<C>().hash(&mut hasher);
        let id_hash = hasher.finish();

        Self {
            trigger,
            content,
            is_open: false,
            position: PopoverPosition::Auto,
            id_hash,
        }
    }

    /// Set the preferred position of the popover relative to the trigger.
    ///
    /// When set to `Auto`, the framework will choose the side with the most
    /// available space at render time.
    pub fn position(mut self, pos: PopoverPosition) -> Self {
        self.position = pos;
        self
    }

    /// Set the initial open state of the popover.
    ///
    /// Use this for static open/close control. For interactive toggling,
    /// the trigger click handler manages the state via component state.
    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    /// Read the current open state from the system component state.
    fn is_open_state(&self) -> bool {
        let s = load_system_state();
        s.get_component_state::<bool>(self.id_hash)
            .and_then(|v| v.read().ok().map(|g| *g))
            .unwrap_or(false)
    }

    /// Write the open state into the system component state.
    pub fn set_open_state(&self, open: bool) {
        let id = self.id_hash;
        update_system_state(move |s| {
            let mut s = s.clone();
            s.set_component_state(id, open);
            s
        });
    }

    /// Resolve the effective position, considering `Auto` placement logic.
    fn resolve_position(&self, trigger_rect: Rect, render_rect: Rect) -> PopoverPosition {
        if self.position != PopoverPosition::Auto {
            return self.position;
        }

        let space_top = trigger_rect.y - render_rect.y;
        let space_bottom =
            (render_rect.y + render_rect.height) - (trigger_rect.y + trigger_rect.height);
        let space_left = trigger_rect.x - render_rect.x;
        let space_right =
            (render_rect.x + render_rect.width) - (trigger_rect.x + trigger_rect.width);

        let mut best = PopoverPosition::Bottom;
        let mut best_space = space_bottom;

        if space_top > best_space {
            best = PopoverPosition::Top;
            best_space = space_top;
        }
        if space_left > best_space {
            best = PopoverPosition::Left;
            best_space = space_left;
        }
        if space_right > best_space {
            best = PopoverPosition::Right;
        }

        best
    }

    /// Compute the popover content rect for the given trigger bounds and position.
    fn popover_rect(&self, trigger_rect: Rect, position: PopoverPosition) -> Rect {
        let gap = 6.0;
        let arrow = 8.0;
        let pop_w = (trigger_rect.width * 1.8).max(160.0).min(320.0);
        let pop_h = 120.0;

        match position {
            PopoverPosition::Top => Rect {
                x: trigger_rect.x + (trigger_rect.width - pop_w) / 2.0,
                y: trigger_rect.y - pop_h - gap - arrow,
                width: pop_w,
                height: pop_h,
            },
            PopoverPosition::Bottom => Rect {
                x: trigger_rect.x + (trigger_rect.width - pop_w) / 2.0,
                y: trigger_rect.y + trigger_rect.height + gap + arrow,
                width: pop_w,
                height: pop_h,
            },
            PopoverPosition::Left => Rect {
                x: trigger_rect.x - pop_w - gap - arrow,
                y: trigger_rect.y + (trigger_rect.height - pop_h) / 2.0,
                width: pop_w,
                height: pop_h,
            },
            PopoverPosition::Right => Rect {
                x: trigger_rect.x + trigger_rect.width + gap + arrow,
                y: trigger_rect.y + (trigger_rect.height - pop_h) / 2.0,
                width: pop_w,
                height: pop_h,
            },
            PopoverPosition::Auto => Rect {
                x: trigger_rect.x + (trigger_rect.width - pop_w) / 2.0,
                y: trigger_rect.y + trigger_rect.height + gap + arrow,
                width: pop_w,
                height: pop_h,
            },
        }
    }

    /// Draw a small arrow triangle pointing from the popover toward the trigger.
    fn draw_arrow(&self, renderer: &mut dyn Renderer, pop_rect: Rect, pos: PopoverPosition) {
        let arrow_color = theme::with_alpha(theme::surface_elevated(), 0.88);
        let size = 5.0;

        // Compute the tip point (on the popover edge) and two base points.
        let (tip, base_a, base_b) = match pos {
            PopoverPosition::Top => {
                // Popover above trigger; arrow points up from popover bottom-center.
                let cx = pop_rect.x + pop_rect.width / 2.0;
                (
                    [cx, pop_rect.y + pop_rect.height],
                    [cx - size, pop_rect.y + pop_rect.height - size],
                    [cx + size, pop_rect.y + pop_rect.height - size],
                )
            }
            PopoverPosition::Bottom => {
                // Popover below trigger; arrow points down from popover top-center.
                let cx = pop_rect.x + pop_rect.width / 2.0;
                (
                    [cx, pop_rect.y],
                    [cx - size, pop_rect.y + size],
                    [cx + size, pop_rect.y + size],
                )
            }
            PopoverPosition::Left => {
                // Popover to the left; arrow points left from popover right-center.
                let cy = pop_rect.y + pop_rect.height / 2.0;
                (
                    [pop_rect.x + pop_rect.width, cy],
                    [pop_rect.x + pop_rect.width - size, cy - size],
                    [pop_rect.x + pop_rect.width - size, cy + size],
                )
            }
            PopoverPosition::Right => {
                // Popover to the right; arrow points right from popover left-center.
                let cy = pop_rect.y + pop_rect.height / 2.0;
                (
                    [pop_rect.x, cy],
                    [pop_rect.x + size, cy - size],
                    [pop_rect.x + size, cy + size],
                )
            }
            PopoverPosition::Auto => {
                let cx = pop_rect.x + pop_rect.width / 2.0;
                (
                    [cx, pop_rect.y],
                    [cx - size, pop_rect.y + size],
                    [cx + size, pop_rect.y + size],
                )
            }
        };

        // Draw three lines forming the triangle.
        let lw = 1.0;
        renderer.draw_line(base_a[0], base_a[1], tip[0], tip[1], arrow_color, lw);
        renderer.draw_line(base_b[0], base_b[1], tip[0], tip[1], arrow_color, lw);
        renderer.draw_line(base_a[0], base_a[1], base_b[0], base_b[1], arrow_color, lw);
    }
}

impl<V: View + Clone + 'static, C: View + Clone + 'static> View for Popover<V, C> {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Popover");

        let id = self.id_hash;
        let is_open = self.is_open_state() || self.is_open;
        let trigger_rect = rect;

        // 1. Render the trigger element.
        renderer.push_vnode(trigger_rect, "PopoverTrigger");
        self.trigger.render(renderer, trigger_rect);
        renderer.pop_vnode();

        // 2. Register trigger click handler to toggle the popover.
        let tr = trigger_rect;
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event: Event| {
                if let Event::PointerClick { x, y, .. } = event
                    && tr.contains(x, y)
                {
                    let current = {
                        let s = load_system_state();
                        s.get_component_state::<bool>(id)
                            .and_then(|v| v.read().ok().map(|g| *g))
                            .unwrap_or(false)
                    };
                    update_system_state(move |s| {
                        let mut s = s.clone();
                        s.set_component_state(id, !current);
                        s
                    });
                }
            }),
        );

        // 3. If open, render the floating popover with backdrop.
        if is_open {
            // Semi-transparent backdrop.
            renderer.fill_rect(rect, theme::with_alpha(theme::bg(), 0.35));

            let position = self.resolve_position(trigger_rect, rect);
            let pop_rect = self.popover_rect(trigger_rect, position);

            // Raise the popover above the backdrop.
            renderer.set_z_index(500.0);

            // Bifrost (frosted glass) effect.
            if crate::theme::glassmorphism_enabled() {
                renderer.bifrost(pop_rect, 20.0, 1.2, 0.92);
            }
            // Semi-transparent dark fill.
            renderer.fill_rounded_rect(pop_rect, 10.0, theme::with_alpha(theme::surface_elevated(), 0.88));
            // Subtle neon border.
            renderer.stroke_rounded_rect(pop_rect, 10.0, theme::with_alpha(theme::border(), 0.7), 1.5);

            // Draw the arrow triangle.
            self.draw_arrow(renderer, pop_rect, position);

            // Render the content inside the popover with padding.
            let pad = 12.0;
            let content_rect = Rect {
                x: pop_rect.x + pad,
                y: pop_rect.y + pad,
                width: (pop_rect.width - pad * 2.0).max(0.0),
                height: (pop_rect.height - pad * 2.0).max(0.0),
            };
            renderer.push_vnode(content_rect, "PopoverContent");
            self.content.render(renderer, content_rect);
            renderer.pop_vnode();

            // Register backdrop click handler (click outside popover and trigger closes it).
            let pr = pop_rect;
            let tr2 = trigger_rect;
            renderer.register_handler(
                "pointerclick",
                Arc::new(move |event: Event| {
                    if let Event::PointerClick { x, y, .. } = event
                        && !pr.contains(x, y)
                        && !tr2.contains(x, y)
                    {
                        update_system_state(move |s| {
                            let mut s = s.clone();
                            s.set_component_state(id, false);
                            s
                        });
                    }
                }),
            );

            renderer.set_z_index(0.0);
        }

        renderer.pop_vnode();
    }
}
