use cvkg_core::{
Event, Never, Rect, Renderer, View};
use crate::theme;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Tooltip position relative to the content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TooltipPosition {
    Top,
    Bottom,
    Left,
    Right,
    #[default]
    Auto,
}

/// Tooltip - A contextual popup that reveals information on hover.
/// Wraps any View and displays a glassmorphic tooltip after a brief
/// delay when the pointer enters the content bounds. The tooltip
/// automatically hides when the pointer leaves.
pub struct Tooltip<V: View> {
    /// The content view that the tooltip is attached to.
    content: V,
    /// The text displayed inside the tooltip popup.
    text: String,
    /// Where the tooltip should appear relative to the content.
    position: TooltipPosition,
    /// Stable hash used to identify this component in the system state store.
    id_hash: u64,
}

/// Internal hover state stored in the system state component map.
#[derive(Clone, Copy)]
struct HoverState {
    /// Whether the pointer is currently inside the content bounds.
    is_hovered: bool,
    /// The elapsed_time value when the pointer entered, used to compute the show delay.
    hover_start_time: f32,
}

impl<V: View> Tooltip<V> {
    /// Create a new Tooltip wrapping the given content.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::Text;
    /// use cvkg_components::tooltip::Tooltip;
    /// let tooltip = Tooltip::new(Text::new("Hover me"), "Helpful information");
    /// ```
    pub fn new(content: V, text: impl Into<String>) -> Self {
        let text_str: String = text.into();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        // Hash the text to produce a stable per-instance ID.
        // In production you would also incorporate a caller-supplied ID.
        text_str.hash(&mut hasher);
        let id_hash = hasher.finish();

        Self {
            content,
            text: text_str,
            position: TooltipPosition::Auto,
            id_hash,
        }
    }

    /// Set the tooltip position relative to the content.
    ///
    /// Defaults to `TooltipPosition::Auto`, which picks the best
    /// position based on available screen space.
    pub fn position(mut self, pos: TooltipPosition) -> Self {
        self.position = pos;
        self
    }

    /// Compute the resolved tooltip position, falling back to Auto
    /// heuristics when the caller chose Auto.
    fn resolved_position(&self, content_rect: Rect) -> TooltipPosition {
        match self.position {
            TooltipPosition::Auto => {
                // Prefer Top; if not enough space above, use Bottom.
                // Heuristic: if content is in the top half of a 900px-tall viewport, show below.
                if content_rect.y > 450.0 {
                    TooltipPosition::Top
                } else {
                    TooltipPosition::Bottom
                }
            }
            other => other,
        }
    }

    /// Calculate the tooltip rectangle given the content bounds and resolved position.
    fn tooltip_rect(&self, content_rect: Rect, renderer: &mut dyn Renderer) -> Rect {
        let padding = 8.0;
        let (tw, th) = renderer.measure_text(&self.text, 12.0);
        let tip_w = tw + padding * 2.0;
        let tip_h = th + padding * 2.0;
        let arrow_gap = 6.0; // gap between content and tooltip arrow tip

        match self.resolved_position(content_rect) {
            TooltipPosition::Top => Rect {
                x: content_rect.x + (content_rect.width - tip_w) / 2.0,
                y: content_rect.y - tip_h - arrow_gap,
                width: tip_w,
                height: tip_h,
            },
            TooltipPosition::Bottom => Rect {
                x: content_rect.x + (content_rect.width - tip_w) / 2.0,
                y: content_rect.y + content_rect.height + arrow_gap,
                width: tip_w,
                height: tip_h,
            },
            TooltipPosition::Left => Rect {
                x: content_rect.x - tip_w - arrow_gap,
                y: content_rect.y + (content_rect.height - tip_h) / 2.0,
                width: tip_w,
                height: tip_h,
            },
            TooltipPosition::Right => Rect {
                x: content_rect.x + content_rect.width + arrow_gap,
                y: content_rect.y + (content_rect.height - tip_h) / 2.0,
                width: tip_w,
                height: tip_h,
            },
            // Auto is resolved above, but handle it anyway.
            TooltipPosition::Auto => Rect {
                x: content_rect.x + (content_rect.width - tip_w) / 2.0,
                y: content_rect.y - tip_h - arrow_gap,
                width: tip_w,
                height: tip_h,
            },
        }
    }

    /// Draw a small arrow triangle pointing from the tooltip toward the content.
    fn draw_arrow(
        &self,
        renderer: &mut dyn Renderer,
        tip_rect: Rect,
        _content_rect: Rect,
        pos: TooltipPosition,
    ) {
        let arrow_color = [0.05, 0.05, 0.1, 0.9];
        let arrow_size = 5.0;

        // Compute the anchor point on the tooltip edge closest to the content,
        // and the two base points that form the triangle.
        let (tip, base_a, base_b) = match pos {
            TooltipPosition::Top => {
                // Tooltip is above content; arrow points upward from tooltip bottom-center.
                let cx = tip_rect.x + tip_rect.width / 2.0;
                let tip_pt = [cx, tip_rect.y + tip_rect.height];
                (
                    tip_pt,
                    [cx - arrow_size, tip_rect.y + tip_rect.height - arrow_size],
                    [cx + arrow_size, tip_rect.y + tip_rect.height - arrow_size],
                )
            }
            TooltipPosition::Bottom => {
                // Tooltip is below content; arrow points downward from tooltip top-center.
                let cx = tip_rect.x + tip_rect.width / 2.0;
                let tip_pt = [cx, tip_rect.y];
                (
                    tip_pt,
                    [cx - arrow_size, tip_rect.y + arrow_size],
                    [cx + arrow_size, tip_rect.y + arrow_size],
                )
            }
            TooltipPosition::Left => {
                // Tooltip is to the left; arrow points leftward from tooltip right-center.
                let cy = tip_rect.y + tip_rect.height / 2.0;
                let tip_pt = [tip_rect.x + tip_rect.width, cy];
                (
                    tip_pt,
                    [tip_rect.x + tip_rect.width - arrow_size, cy - arrow_size],
                    [tip_rect.x + tip_rect.width - arrow_size, cy + arrow_size],
                )
            }
            TooltipPosition::Right => {
                // Tooltip is to the right; arrow points rightward from tooltip left-center.
                let cy = tip_rect.y + tip_rect.height / 2.0;
                let tip_pt = [tip_rect.x, cy];
                (
                    tip_pt,
                    [tip_rect.x + arrow_size, cy - arrow_size],
                    [tip_rect.x + arrow_size, cy + arrow_size],
                )
            }
            TooltipPosition::Auto => {
                // Should not reach here after resolution, but draw a downward arrow.
                let cx = tip_rect.x + tip_rect.width / 2.0;
                let tip_pt = [cx, tip_rect.y];
                (
                    tip_pt,
                    [cx - arrow_size, tip_rect.y + arrow_size],
                    [cx + arrow_size, tip_rect.y + arrow_size],
                )
            }
        };

        // Draw the triangle as two filled half-rects (approximated with lines for a simple arrow).
        // We draw three lines forming the triangle outline.
        let line_width = 1.0;
        renderer.draw_line(
            base_a[0],
            base_a[1],
            tip[0],
            tip[1],
            arrow_color,
            line_width,
        );
        renderer.draw_line(
            base_b[0],
            base_b[1],
            tip[0],
            tip[1],
            arrow_color,
            line_width,
        );
        renderer.draw_line(
            base_a[0],
            base_a[1],
            base_b[0],
            base_b[1],
            arrow_color,
            line_width,
        );
    }
}

impl<V: View> View for Tooltip<V> {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Tooltip");

        // Render the wrapped content first.
        self.content.render(renderer, rect);

        // Read hover state from the system state store.
        let hover_state = {
            let state = cvkg_core::load_system_state();
            state
                .get_component_state::<HoverState>(self.id_hash)
                .map(|lock| *lock.read().unwrap())
                .unwrap_or(HoverState {
                    is_hovered: false,
                    hover_start_time: 0.0,
                })
        };

        let now = renderer.elapsed_time();
        let mut hover_start = hover_state.hover_start_time;
        if hover_state.is_hovered && hover_start == -1.0 {
            hover_start = now;
            cvkg_core::update_system_state(|s| {
                let mut next = s.clone();
                next.set_component_state(self.id_hash, HoverState {
                    is_hovered: true,
                    hover_start_time: now,
                });
                next
            });
        }

        let show_delay = 0.3;
        let should_show = hover_state.is_hovered && (now - hover_start) >= show_delay && hover_start > 0.0;
        let target = if should_show { 1.0 } else { 0.0 };

        let anim_hash = self.id_hash.wrapping_add(12345);
        let mut t_val = 0.0;
        {
            let s = cvkg_core::load_system_state();
            if s.get_component_state::<cvkg_anim::SleipnirSolver>(anim_hash).is_none() {
                cvkg_core::update_system_state(|st| {
                    let mut new_st = st.clone();
                    new_st.set_component_state(anim_hash, cvkg_anim::SleipnirSolver::new(cvkg_anim::SleipnirParams::snappy(), target, 0.0));
                    new_st
                });
            }
        }
        {
            let s = cvkg_core::load_system_state();
            if let Some(solver_arc) = s.get_component_state::<cvkg_anim::SleipnirSolver>(anim_hash) {
                let mut solver = solver_arc.write().unwrap();
                solver.set_target(target);
                t_val = solver.tick(renderer.delta_time());
            }
        }

        if t_val > 0.01 {
            let tip_rect = self.tooltip_rect(rect, renderer);
            let pos = self.resolved_position(rect);

            // Raise the tooltip above other content.
            renderer.set_z_index(1000.0);
            
            renderer.push_opacity(t_val);
            let scale = 0.9 + 0.1 * t_val;
            
            // Calculate center for scaling
            let cx = tip_rect.x + tip_rect.width / 2.0;
            let cy = tip_rect.y + tip_rect.height / 2.0;
            
            renderer.push_transform(
                [cx - cx * scale, cy - cy * scale],
                [scale, scale],
                0.0,
            );

            // Dark glassmorphic background.
            renderer.fill_rounded_rect(tip_rect, 6.0, [0.05, 0.05, 0.1, 0.9]);
            // Subtle border.
            renderer.stroke_rounded_rect(tip_rect, 6.0, [0.2, 0.2, 0.3, 0.5], 1.0);

            // Draw the small arrow pointing toward the content.
            self.draw_arrow(renderer, tip_rect, rect, pos);

            // Render the tooltip text.
            let text_x = tip_rect.x + 8.0;
            let text_y = tip_rect.y + 6.0;
            renderer.draw_text(&self.text, text_x, text_y, 12.0, theme::text());

            renderer.pop_transform();
            renderer.pop_opacity();
            renderer.set_z_index(0.0);
        }

        // Register pointer enter handler to start tracking hover.
        let id_hash_enter = self.id_hash;
        renderer.register_handler(
            "pointerenter",
            Arc::new(move |_: Event| {
                cvkg_core::update_system_state(|s| {
                    let mut next = s.clone();
                    // We need the current elapsed time; since we can't access the renderer
                    // inside the handler, we use a monotonic approximation.
                    // The hover_start_time will be set to 0.0 as a sentinel; the actual
                    // delay check uses the renderer's elapsed_time at render time.
                    // For simplicity, we store 0.0 and rely on the first frame after
                    // hover to begin the delay countdown.
                    next.set_component_state(
                        id_hash_enter,
                        HoverState {
                            is_hovered: true,
                            hover_start_time: 0.0,
                        },
                    );
                    next
                });
            }),
        );

        // Register pointer leave handler to stop tracking hover.
        let id_hash_leave = self.id_hash;
        renderer.register_handler(
            "pointerleave",
            Arc::new(move |_: Event| {
                cvkg_core::update_system_state(|s| {
                    let mut next = s.clone();
                    next.set_component_state(
                        id_hash_leave,
                        HoverState {
                            is_hovered: false,
                            hover_start_time: 0.0,
                        },
                    );
                    next
                });
            }),
        );

        // Register a pointer move handler to capture the elapsed_time at hover start.
        // This allows us to measure the 0.3s delay accurately.
        let id_hash_move = self.id_hash;
        renderer.register_handler(
            "pointermove",
            Arc::new(move |_: Event| {
                cvkg_core::update_system_state(|s| {
                    let mut next = s.clone();
                    // Only update if currently hovered; use a placeholder time.
                    // The actual elapsed_time is captured during render.
                    if let Some(lock) = next.get_component_state::<HoverState>(id_hash_move)
                        && let Ok(state) = lock.read()
                        && state.is_hovered
                        && state.hover_start_time == 0.0
                    {
                        // Mark that we've seen movement while hovered.
                        // We use a negative sentinel to indicate "waiting for render time".
                        next.set_component_state(
                            id_hash_move,
                            HoverState {
                                is_hovered: true,
                                hover_start_time: -1.0,
                            },
                        );
                    }
                    next
                });
            }),
        );

        renderer.pop_vnode();
    }
}
