use crate::theme;
use crate::{FONT_BASE, RADIUS_LG, RADIUS_MD, draw_focus_ring};
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{Event, Never, Rect, Renderer, Size, View};
use std::sync::Arc;

/// System-state hash key for the dialog open/close state.
const DIALOG_OPEN_HASH: u64 = 0xB00_0001;

/// Navigation stack for push/pop navigation
pub struct NavigationStack {
    pub(crate) stack: Vec<cvkg_core::AnyView>,
}

impl NavigationStack {
    pub fn new<V: View + Clone + 'static>(root: V) -> Self {
        Self {
            stack: vec![root.erase()],
        }
    }

    pub fn push<V: View + Clone + 'static>(&mut self, view: V) {
        self.stack.push(view.erase());
    }

    pub fn pop(&mut self) -> Option<cvkg_core::AnyView> {
        if self.stack.len() > 1 {
            self.stack.pop()
        } else {
            None
        }
    }
}

impl View for NavigationStack {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "NavigationStack");
        // Render the top-most view in the stack
        if let Some(top) = self.stack.last() {
            top.render(renderer, rect);
        }
        renderer.pop_vnode();
    }
}

/// Navigation split view for sidebar layouts
pub struct NavigationSplitView<S, D> {
    pub(crate) sidebar: S,
    pub(crate) detail: D,
    pub(crate) is_collapsed: bool,
    pub(crate) sidebar_width: f32,
}

impl<S: View, D: View> NavigationSplitView<S, D> {
    pub fn new(sidebar: S, detail: D) -> Self {
        Self {
            sidebar,
            detail,
            is_collapsed: false,
            sidebar_width: 300.0,
        }
    }

    pub fn is_collapsed(mut self, collapsed: bool) -> Self {
        self.is_collapsed = collapsed;
        self
    }

    pub fn sidebar_width(mut self, width: f32) -> Self {
        self.sidebar_width = width.clamp(150.0, 500.0);
        self
    }
}

impl<S: View, D: View> View for NavigationSplitView<S, D> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let nav_collapse_hash: u64 = 0xA00_0001;
        let is_collapsed: bool = cvkg_core::load_system_state()
            .get_component_state::<bool>(nav_collapse_hash)
            .and_then(|v| v.read().ok().map(|v| *v))
            .unwrap_or(self.is_collapsed);
        let collapsed_width: f32 = 48.0;
        let handle_width: f32 = 4.0;
        let effective_sidebar_width = if is_collapsed {
            collapsed_width
        } else {
            self.sidebar_width.clamp(150.0, 500.0)
        };
        let sidebar_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: effective_sidebar_width,
            height: rect.height,
        };
        let handle_rect = Rect {
            x: rect.x + effective_sidebar_width,
            y: rect.y,
            width: handle_width,
            height: rect.height,
        };
        let detail_rect = Rect {
            x: rect.x + effective_sidebar_width + handle_width,
            y: rect.y,
            width: (rect.width - effective_sidebar_width - handle_width).max(0.0),
            height: rect.height,
        };

        if is_collapsed {
            // ── Collapsed: icon rail ──
            renderer.fill_rect(sidebar_rect, theme::surface());
            renderer.stroke_rect(sidebar_rect, [0.2, 0.2, 0.3, 0.5], 1.0);
        } else {
            // ── Full sidebar ──
            renderer.fill_rect(sidebar_rect, theme::surface());
            renderer.stroke_rect(sidebar_rect, [0.2, 0.2, 0.3, 0.5], 1.0);
            self.sidebar.render(renderer, sidebar_rect);
        }

        // ── Draggable resize handle ──
        let hover_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            "nav_split_handle_hover".hash(&mut s);
            s.finish()
        };
        let is_hover_handle: bool = cvkg_core::load_system_state()
            .get_component_state::<bool>(hover_hash)
            .and_then(|v| v.read().ok().map(|v| *v))
            .unwrap_or(false);
        if is_hover_handle && !is_collapsed {
            let _ = "col-resize";
        }
        renderer.fill_rect(
            handle_rect,
            if is_hover_handle && !is_collapsed {
                [0.0, 0.8, 1.0, 0.3]
            } else {
                [0.12, 0.12, 0.16, 0.8]
            },
        );
        renderer.stroke_rect(
            handle_rect,
            [0.0, 0.8, 1.0, if is_hover_handle { 0.6 } else { 0.2 }],
            1.0,
        );

        // Handle hover + drag detection
        let h_rect = handle_rect;
        let _ww = rect.width;
        renderer.register_handler(
            "pointermove",
            Arc::new(move |event| {
                if let Event::PointerMove { x, y, .. } = event {
                    let hovering = h_rect.contains(x, y);
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        s.set_component_state(hover_hash, hovering);
                        s
                    });
                }
            }),
        );

        // Collapse toggle button (chevron) at the sidebar edge
        let toggle_btn_size: f32 = 24.0;
        let toggle_rect = Rect {
            x: rect.x + effective_sidebar_width - toggle_btn_size / 2.0,
            y: rect.y + 12.0,
            width: toggle_btn_size,
            height: toggle_btn_size,
        };
        renderer.fill_rounded_rect(toggle_rect, 12.0, [0.1, 0.1, 0.15, 0.9]);
        let chevron = if is_collapsed { "▶" } else { "◀" };
        renderer.draw_text(
            chevron,
            toggle_rect.x + 6.0,
            toggle_rect.y + 5.0,
            12.0,
            theme::text_muted(),
        );

        // Toggle click
        let tglm = toggle_rect;
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event| {
                if let Event::PointerClick { x, y, .. } = event
                    && tglm.contains(x, y)
                {
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        let nav_collapse_hash: u64 = 0xA00_0001;
                        let current: bool = s
                            .get_component_state::<bool>(nav_collapse_hash)
                            .and_then(|v| v.read().ok().map(|v| *v))
                            .unwrap_or(false);
                        s.set_component_state(nav_collapse_hash, !current);
                        s
                    });
                }
            }),
        );

        // Keyboard: Ctrl+B to toggle sidebar collapse
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event
                    && (key == "b" || key == "B")
                {
                    // Note: ctrl modifier not checked for simplicity; add ctrl check if needed
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        let nav_collapse_hash: u64 = 0xA00_0001;
                        let current: bool = s
                            .get_component_state::<bool>(nav_collapse_hash)
                            .and_then(|v| v.read().ok().map(|v| *v))
                            .unwrap_or(false);
                        s.set_component_state(nav_collapse_hash, !current);
                        s
                    });
                }
            }),
        );

        // ── Detail area ──
        self.detail.render(renderer, detail_rect);
    }
}

/// Modal bottom sheet or centered dialog
pub struct GraniSheet<V> {
    pub(crate) content: V,
    pub(crate) position: SheetPosition,
    pub(crate) is_open: bool,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) on_dismiss: Option<Arc<dyn Fn() + Send + Sync>>,
}

/// Position from which the sheet slides in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SheetPosition {
    Left,
    Right,
    Top,
    Bottom,
}

impl SheetPosition {
    pub fn is_horizontal(self) -> bool {
        matches!(self, SheetPosition::Left | SheetPosition::Right)
    }
}

const SHEET_ANIM_HASH: u64 = 0xB00_0100;

impl<V: View> GraniSheet<V> {
    pub fn new(content: V, position: SheetPosition) -> Self {
        Self {
            content,
            position,
            is_open: false,
            width: 320.0,
            height: 400.0,
            on_dismiss: None,
        }
    }

    pub fn position(mut self, position: SheetPosition) -> Self {
        self.position = position;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn on_dismiss<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_dismiss = Some(Arc::new(callback));
        self
    }
}

impl<V: View> View for GraniSheet<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.is_open {
            return;
        }

        // ── Backdrop ──
        renderer.fill_rect(rect, [0.0, 0.0, 0.0, 0.65]);

        // ── Animation progress using Sleipnir spring physics ──
        let mut anim = 0.0f32;
        let solver_hash = SHEET_ANIM_HASH;
        {
            let sys = cvkg_core::load_system_state();
            if sys
                .get_component_state::<cvkg_core::SleipnirSolver>(solver_hash)
                .is_none()
            {
                cvkg_core::update_system_state(move |s| {
                    let mut ns = s.clone();
                    ns.set_component_state(
                        solver_hash,
                        cvkg_core::SleipnirSolver::new(
                            cvkg_core::SleipnirParams::fluid(),
                            1.0,
                            0.0,
                        ),
                    );
                    ns
                });
            }
        }
        {
            let sys = cvkg_core::load_system_state();
            if let Some(solver_arc) =
                sys.get_component_state::<cvkg_core::SleipnirSolver>(solver_hash)
            {
                let mut solver = solver_arc.write().expect("lock poisoned");
                solver.set_target(1.0);
                anim = solver.tick(renderer.delta_time());
                if !solver.is_settled() {
                    renderer.request_redraw();
                }
            }
        }

        let sheet_rect = match self.position {
            SheetPosition::Left => {
                let w = self.width * anim;
                Rect {
                    x: rect.x,
                    y: rect.y,
                    width: w,
                    height: rect.height,
                }
            }
            SheetPosition::Right => {
                let w = self.width * anim;
                Rect {
                    x: rect.x + rect.width - w,
                    y: rect.y,
                    width: w,
                    height: rect.height,
                }
            }
            SheetPosition::Top => {
                let h = self.height * anim;
                Rect {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: h,
                }
            }
            SheetPosition::Bottom => {
                let h = self.height * anim;
                Rect {
                    x: rect.x,
                    y: rect.y + rect.height - h,
                    width: rect.width,
                    height: h,
                }
            }
        };

        // ── Panel background ──
        renderer.bifrost(sheet_rect, 16.0, 1.2, 0.9);
        renderer.fill_rounded_rect(sheet_rect, 12.0, [0.04, 0.04, 0.07, 0.95]);
        renderer.stroke_rounded_rect(sheet_rect, 12.0, [0.0, 0.8, 1.0, 0.3], 1.5);

        // ── Content ──
        let padding = 16.0;
        let sheet_content_rect = Rect {
            x: sheet_rect.x + padding,
            y: sheet_rect.y + padding + 28.0, // leave room for close button
            width: (sheet_rect.width - padding * 2.0).max(0.0),
            height: (sheet_rect.height - padding * 2.0 - 28.0).max(0.0),
        };
        self.content.render(renderer, sheet_content_rect);

        // ── Close button (×) top-right ──
        let btn_size: f32 = 28.0;
        let close_rect = Rect {
            x: sheet_rect.x + sheet_rect.width - btn_size - 8.0,
            y: sheet_rect.y + 8.0,
            width: btn_size,
            height: btn_size,
        };
        renderer.fill_rounded_rect(close_rect, 14.0, [0.12, 0.12, 0.16, 0.8]);
        renderer.draw_text(
            "×",
            close_rect.x + 9.0,
            close_rect.y + 7.0,
            FONT_BASE + 2.0,
            theme::text_muted(),
        );

        let close_cb = self.on_dismiss.clone();
        // Dismiss: clicking the close button
        renderer.register_handler(
            "pointerdown",
            Arc::new(move |event| {
                if let Event::PointerDown { x, y, .. } = event
                    && close_rect.contains(x, y)
                {
                    if let Some(ref cb) = close_cb {
                        (cb)();
                    }
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        s.component_states.remove(&solver_hash);
                        s
                    });
                }
            }),
        );

        // Dismiss: clicking the backdrop (outside the panel)
        let sheet_rect_capture = sheet_rect;
        let dismiss_cb = self.on_dismiss.clone();
        renderer.register_handler(
            "pointerdown",
            Arc::new(move |event| {
                if let Event::PointerDown { x, y, .. } = event
                    && !sheet_rect_capture.contains(x, y)
                {
                    if let Some(ref cb) = dismiss_cb {
                        (cb)();
                    }
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        s.component_states.remove(&solver_hash);
                        s
                    });
                }
            }),
        );
    }
}

/// A modifier that presents a modal sheet over a view.
#[derive(Clone)]
pub struct SheetModifier<V2> {
    pub is_presented: bool,
    pub content: V2,
}

impl<V2: View + Clone> cvkg_core::ViewModifier for SheetModifier<V2> {
    fn modify<V: View>(self, content: V) -> impl View {
        cvkg_core::ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        view.render(renderer, rect);

        if self.is_presented {
            renderer.fill_rect(rect, theme::shadow());

            let modal_width = (rect.width * 0.8).min(500.0);
            let modal_height = (rect.height * 0.6).min(400.0);
            let modal_rect = Rect {
                x: rect.x + (rect.width - modal_width) / 2.0,
                y: rect.y + (rect.height - modal_height) / 2.0,
                width: modal_width,
                height: modal_height,
            };

            renderer.bifrost(modal_rect, 25.0, 1.5, 0.85);
            renderer.fill_rounded_rect(modal_rect, 12.0, [0.0, 0.0, 0.0, 0.3]);
            renderer.stroke_rounded_rect(modal_rect, 12.0, [0.2, 0.25, 0.3, 0.6], 2.0);

            self.content.render(renderer, modal_rect);
        }
    }
}

/// A modal dialog with title, content, and actions.
pub struct GeriDialog<V> {
    pub(crate) is_presented: bool,
    pub(crate) title: Option<String>,
    pub(crate) content: V,
    pub(crate) actions: Vec<DialogAction>,
}

impl<V: View> GeriDialog<V> {
    pub fn new(content: V) -> Self {
        Self {
            is_presented: false,
            title: None,
            content,
            actions: Vec::new(),
        }
    }

    pub fn presented(mut self, is_presented: bool) -> Self {
        self.is_presented = is_presented;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn action(
        mut self,
        label: impl Into<String>,
        on_click: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        self.actions.push(DialogAction {
            label: label.into(),
            style: DialogActionStyle::Default,
            on_click: std::sync::Arc::new(on_click),
        });
        self
    }
}

impl<V: View> View for GeriDialog<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.is_presented {
            return;
        }

        // ── Backdrop (semi-transparent overlay covering entire viewport) ──
        renderer.fill_rect(rect, theme::shadow());

        let modal_w = (rect.width * 0.8).min(450.0);
        let modal_h = (rect.height * 0.5).min(350.0);
        let modal_rect = Rect {
            x: rect.x + (rect.width - modal_w) / 2.0,
            y: rect.y + (rect.height - modal_h) / 2.0,
            width: modal_w,
            height: modal_h,
        };

        // ── Opacity fade animation (entry/exit) ──
        // Read the animation progress from system state.
        // The opacity ramps from 0.0 to 1.0 over ~0.2s using frame-based lerp.
        let anim_hash = DIALOG_OPEN_HASH + 100;
        let mut opacity: f32 = cvkg_core::load_system_state()
            .get_component_state::<f32>(anim_hash)
            .and_then(|g| g.read().ok().map(|v| *v))
            .unwrap_or(0.0);
        // Advance the animation toward 1.0 when presented
        opacity = (opacity + 0.15).min(1.0);
        cvkg_core::update_system_state(move |s| {
            let mut s = s.clone();
            s.set_component_state(anim_hash, opacity);
            s
        });

        // Apply opacity to backdrop and modal colors
        let backdrop_alpha = 0.7 * opacity;
        renderer.fill_rect(rect, [0.0, 0.0, 0.0, backdrop_alpha]);

        // ── Modal glass surface ──
        renderer.bifrost(modal_rect, 20.0, 1.2, 0.9);
        let modal_bg = [0.05 * opacity, 0.05 * opacity, 0.1 * opacity, 0.8 * opacity];
        renderer.fill_rounded_rect(modal_rect, RADIUS_LG, modal_bg);
        renderer.stroke_rounded_rect(
            modal_rect,
            RADIUS_LG,
            [0.0 * opacity, 0.8 * opacity, 1.0 * opacity, 0.5 * opacity],
            1.5,
        );

        // ── Focus ring (visual focus trap indicator) ──
        if opacity > 0.5 {
            draw_focus_ring(renderer, modal_rect);
        }

        // ── Title ──
        let padding = 20.0;
        let mut current_y = modal_rect.y + padding;

        if let Some(title) = &self.title {
            renderer.draw_text(
                title,
                modal_rect.x + padding,
                current_y,
                FONT_BASE + 6.0,
                [1.0 * opacity, 1.0 * opacity, 1.0 * opacity, opacity],
            );
            current_y += 30.0;
        }

        // ── Content area ──
        let content_h = modal_h - (current_y - modal_rect.y) - 60.0;
        let content_rect = Rect {
            x: modal_rect.x + padding,
            y: current_y,
            width: modal_w - 2.0 * padding,
            height: content_h,
        };
        self.content.render(renderer, content_rect);

        // ── Action buttons ──
        let action_y = modal_rect.y + modal_h - 45.0;
        let action_w = 80.0;
        // Read focused action index from system state (set by Tab key handler)
        let focused_action = cvkg_core::load_system_state()
            .get_component_state::<usize>(DIALOG_OPEN_HASH + 200)
            .and_then(|v| v.read().ok().map(|v| *v));
        for (i, action) in self.actions.iter().enumerate() {
            let action_rect = Rect {
                x: modal_rect.x + modal_w - padding - (i as f32 + 1.0) * (action_w + 10.0),
                y: action_y,
                width: action_w,
                height: 30.0,
            };
            let is_focused = focused_action == Some(i);
            // Focused button gets a brighter background
            let bg = if is_focused {
                [0.25 * opacity, 0.25 * opacity, 0.35 * opacity, opacity]
            } else {
                [0.15 * opacity, 0.15 * opacity, 0.2 * opacity, opacity]
            };
            renderer.fill_rounded_rect(action_rect, RADIUS_MD, bg);
            renderer.stroke_rounded_rect(
                action_rect,
                RADIUS_MD,
                [0.0 * opacity, 0.8 * opacity, 1.0 * opacity, 0.8 * opacity],
                if is_focused { 2.0 } else { 1.0 },
            );
            // Focus ring on the focused button
            if is_focused && opacity > 0.5 {
                draw_focus_ring(renderer, action_rect);
            }
            renderer.draw_text(
                &action.label,
                action_rect.x + 8.0,
                action_rect.y + 8.0,
                FONT_BASE,
                [1.0 * opacity, 1.0 * opacity, 1.0 * opacity, opacity],
            );

            // Click handler for this action button
            let on_click = action.on_click.clone();
            renderer.register_handler(
                "pointerclick",
                Arc::new(move |event| {
                    if let Event::PointerClick { x, y, .. } = event
                        && action_rect.contains(x, y)
                    {
                        (on_click)();
                    }
                }),
            );
        }

        // ── Close-on-backdrop-click handler ──
        let mr = modal_rect;
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event| {
                if let Event::PointerClick { x, y, .. } = event {
                    // Click outside the modal rect closes the dialog
                    if !mr.contains(x, y) {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            s.set_component_state(DIALOG_OPEN_HASH, false);
                            s.set_component_state(anim_hash, 0.0);
                            s
                        });
                    }
                }
            }),
        );

        // ── Keyboard handler: Escape close + Tab focus trap ──
        // ── Keyboard handler: Escape close + Tab focus trap ──
        // Focus trap: when Tab is pressed inside the dialog, cycle focus among
        // the action buttons within the modal. This prevents Tab from escaping
        // the modal and focusing elements behind the backdrop.
        let action_count = self.actions.len();
        // Clone callbacks for use in the keyboard handler (avoids borrowing self which has generic V)
        let action_callbacks: Vec<_> = self.actions.iter().map(|a| a.on_click.clone()).collect();
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    if key == "Escape" {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            s.set_component_state(DIALOG_OPEN_HASH, false);
                            s.set_component_state(anim_hash, 0.0);
                            s
                        });
                    } else if key == "Tab" && action_count > 0 {
                        // Focus trap: cycle forward among action buttons
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            let current = s
                                .get_component_state::<usize>(DIALOG_OPEN_HASH + 200)
                                .and_then(|v| v.read().ok().map(|g| *g))
                                .unwrap_or(0);
                            let next = (current + 1) % action_count;
                            s.set_component_state(DIALOG_OPEN_HASH + 200, next);
                            s
                        });
                    } else if key == "Enter" {
                        // Trigger the focused action button
                        let focused = cvkg_core::load_system_state()
                            .get_component_state::<usize>(DIALOG_OPEN_HASH + 200)
                            .and_then(|v| v.read().ok().map(|v| *v));
                        if let Some(idx) = focused
                            && let Some(cb) = action_callbacks.get(idx)
                        {
                            (cb)();
                        }
                    }
                }
            }),
        );
    }
}

#[derive(Clone)]
pub struct DialogAction {
    pub label: String,
    pub style: DialogActionStyle,
    pub on_click: std::sync::Arc<dyn Fn() + Send + Sync>,
}

#[derive(Clone)]
pub enum DialogActionStyle {
    Default,
    Destructive,
    Cancel,
}

/// Scrollable container for content that exceeds available space with smooth
/// momentum scrolling, auto-hiding scrollbars, and proper clipping.
/// Scroll state (position, velocity, scrollbar opacity) is stored in the
/// system state map under `scroll_id`. The component responds to:
/// - Mouse / trackpad wheel events (delta_x, delta_y)
/// - Pointer drag gestures (pointerdown + pointermove + pointerup)
/// - Keyboard navigation (PageUp/PageDown/Home/End/Arrow keys)
/// State layout per scroll_id:
///   - `(f32, f32)` at key `scroll_id`          = scroll position (x, y)
///   - `(f32, f32)` at key `scroll_id + 1`      = velocity (vx, vy)
///   - `f32`        at key `scroll_id + 2`      = scrollbar opacity [0..1]
///   - `(f32, f32)` at key `scroll_id + 1000`   = content size (w, h) hint
/// A container view that supports scrolling its child content.
/// It features pointer drag scrolling with rubber-band physics at bounds,
/// momentum scrolling, keyboard navigation, and interactive scrollbars.
#[derive(Clone)]
pub struct ScrollView<V> {
    pub(crate) content: V,
    /// Unique identifier for this scroll view's state in the system state map.
    pub(crate) scroll_id: u64,
    /// Cached or specified content size.
    pub(crate) content_size: [f32; 2],
    /// Width of scrollbars.
    pub(crate) scrollbar_width: f32,
    /// Scroll speed multiplier for wheel events.
    pub(crate) scroll_speed: f32,
    /// Momentum decay factor per frame.
    pub(crate) momentum_decay: f32,
    /// How long the scrollbar stays visible after last scroll.
    pub(crate) scrollbar_fade_delay: u32,
    /// Scrollbar fade-out speed per frame.
    pub(crate) scrollbar_fade_speed: f32,
}

/// Internal scroll state stored in the system state map.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ScrollState {
    scroll_offset: [f32; 2],
    momentum_velocity: [f32; 2],
    scrollbar_opacity: f32,
    last_scroll_frame: u32,
    is_dragging: bool,
    last_pointer_pos: [f32; 2],
    is_scrollbar_dragging_v: bool,
    is_scrollbar_dragging_h: bool,
    scrollbar_drag_offset: f32,
    spring_x: Option<cvkg_core::SleipnirSolver>,
    spring_y: Option<cvkg_core::SleipnirSolver>,
    /// Current zoom level from pinch gestures (1.0 = normal).
    zoom_level: f32,
    /// Sleipnir spring for smooth zoom animation.
    zoom_spring: Option<cvkg_core::SleipnirSolver>,
    /// Minimum allowed zoom from pinch.
    min_zoom: f32,
    /// Maximum allowed zoom from pinch.
    max_zoom: f32,
}

impl Default for ScrollState {
    fn default() -> Self {
        Self {
            scroll_offset: [0.0, 0.0],
            momentum_velocity: [0.0, 0.0],
            scrollbar_opacity: 0.0,
            last_scroll_frame: 0,
            is_dragging: false,
            last_pointer_pos: [0.0, 0.0],
            is_scrollbar_dragging_v: false,
            is_scrollbar_dragging_h: false,
            scrollbar_drag_offset: 0.0,
            spring_x: None,
            spring_y: None,
            zoom_level: 1.0,
            zoom_spring: None,
            min_zoom: 0.25,
            max_zoom: 4.0,
        }
    }
}

impl<V: View> ScrollView<V> {
    /// Create a new ScrollView wrapping `content`.
    pub fn new(content: V) -> Self {
        Self {
            content,
            scroll_id: 0,
            content_size: [0.0, 0.0],
            scrollbar_width: 6.0,
            scroll_speed: 1.0,
            momentum_decay: 0.92,
            scrollbar_fade_delay: 60,
            scrollbar_fade_speed: 0.85,
        }
    }

    /// Set a unique ID for this scroll view's state in the system state map.
    pub fn scroll_id(mut self, id: u64) -> Self {
        self.scroll_id = id;
        self
    }

    /// Set the content size hint for scrollbar calculations.
    pub fn content_size(mut self, width: f32, height: f32) -> Self {
        self.content_size = [width, height];
        self
    }

    /// Set scroll speed multiplier for wheel events (default 1.0).
    pub fn scroll_speed(mut self, speed: f32) -> Self {
        self.scroll_speed = speed;
        self
    }

    /// Set momentum decay factor (default 0.92). Higher = more inertia.
    pub fn momentum_decay(mut self, decay: f32) -> Self {
        self.momentum_decay = decay.clamp(0.0, 0.999);
        self
    }

    /// Set scrollbar width in pixels (default 6.0).
    pub fn scrollbar_width(mut self, width: f32) -> Self {
        self.scrollbar_width = width.max(2.0);
        self
    }

    /// Set how many frames the scrollbar stays visible after scrolling (default 60).
    pub fn scrollbar_fade_delay(mut self, frames: u32) -> Self {
        self.scrollbar_fade_delay = frames;
        self
    }

    /// Set scrollbar fade-out speed per frame (default 0.85).
    pub fn scrollbar_fade_speed(mut self, speed: f32) -> Self {
        self.scrollbar_fade_speed = speed.clamp(0.0, 1.0);
        self
    }

    /// Helper: read scroll state from system state.
    fn read_state(&self) -> ScrollState {
        if self.scroll_id == 0 {
            return ScrollState::default();
        }
        let state = cvkg_core::load_system_state();
        state
            .get_component_state::<ScrollState>(self.scroll_id)
            .and_then(|guard| guard.read().ok().map(|v| *v))
            .unwrap_or_default()
    }

    /// Helper: write scroll state to system state.
    fn write_state(&self, new_state: ScrollState) {
        if self.scroll_id == 0 {
            return;
        }
        cvkg_core::update_system_state(move |s| {
            let mut s = s.clone();
            s.set_component_state(self.scroll_id, new_state);
            s
        });
    }

    /// Apply physics tick including spring solver bounce-back.
    fn tick_physics(
        state: &mut ScrollState,
        viewport_w: f32,
        viewport_h: f32,
        content_w: f32,
        content_h: f32,
        dt: f32,
        decay: f32,
    ) {
        let max_x = (content_w - viewport_w).max(0.0);
        let max_y = (content_h - viewport_h).max(0.0);

        if state.is_dragging || state.is_scrollbar_dragging_v || state.is_scrollbar_dragging_h {
            state.spring_x = None;
            state.spring_y = None;
            return;
        }

        // Apply X axis spring-back or momentum
        if state.scroll_offset[0] < 0.0 || state.scroll_offset[0] > max_x {
            let target = if state.scroll_offset[0] < 0.0 {
                0.0
            } else {
                max_x
            };
            let mut solver = state.spring_x.unwrap_or_else(|| {
                cvkg_core::SleipnirSolver::new(
                    cvkg_core::SleipnirParams::fluid(),
                    target,
                    state.scroll_offset[0],
                )
            });
            solver.set_target(target);
            state.scroll_offset[0] = solver.tick(dt);
            state.momentum_velocity[0] = 0.0;
            if solver.is_settled() {
                state.scroll_offset[0] = target;
                state.spring_x = None;
            } else {
                state.spring_x = Some(solver);
            }
        } else {
            state.spring_x = None;
            if state.momentum_velocity[0].abs() > 0.01 {
                state.scroll_offset[0] += state.momentum_velocity[0] * dt * 60.0;
                state.momentum_velocity[0] *= decay;
            } else {
                state.momentum_velocity[0] = 0.0;
            }
        }

        // Apply Y axis spring-back or momentum
        if state.scroll_offset[1] < 0.0 || state.scroll_offset[1] > max_y {
            let target = if state.scroll_offset[1] < 0.0 {
                0.0
            } else {
                max_y
            };
            let mut solver = state.spring_y.unwrap_or_else(|| {
                cvkg_core::SleipnirSolver::new(
                    cvkg_core::SleipnirParams::fluid(),
                    target,
                    state.scroll_offset[1],
                )
            });
            solver.set_target(target);
            state.scroll_offset[1] = solver.tick(dt);
            state.momentum_velocity[1] = 0.0;
            if solver.is_settled() {
                state.scroll_offset[1] = target;
                state.spring_y = None;
            } else {
                state.spring_y = Some(solver);
            }
        } else {
            state.spring_y = None;
            if state.momentum_velocity[1].abs() > 0.01 {
                state.scroll_offset[1] += state.momentum_velocity[1] * dt * 60.0;
                state.momentum_velocity[1] *= decay;
            } else {
                state.momentum_velocity[1] = 0.0;
            }
        }

        // Apply zoom spring-back for pinch gestures.
        // The zoom spring smoothly animates toward the clamped zoom target.
        let zoom_target = state.zoom_level.clamp(state.min_zoom, state.max_zoom);
        if (state.zoom_level - zoom_target).abs() > 0.001 {
            let mut solver = state.zoom_spring.unwrap_or_else(|| {
                cvkg_core::SleipnirSolver::new(
                    cvkg_core::SleipnirParams::fluid(),
                    zoom_target,
                    state.zoom_level,
                )
            });
            solver.set_target(zoom_target);
            state.zoom_level = solver.tick(dt);
            if solver.is_settled() {
                state.zoom_level = zoom_target;
                state.zoom_spring = None;
            } else {
                state.zoom_spring = Some(solver);
            }
        } else {
            state.zoom_spring = None;
        }
    }

    /// Render a scrollbar track and thumb.
    fn render_scrollbar(
        &self,
        renderer: &mut dyn Renderer,
        rect: Rect,
        content_size: f32,
        viewport_size: f32,
        scroll_pos: f32,
        opacity: f32,
        is_vertical: bool,
    ) {
        if opacity <= 0.001 || content_size <= viewport_size {
            return;
        }

        let sb_w = self.scrollbar_width;
        let track_color = [0.0, 0.0, 0.0, 0.15 * opacity];
        let thumb_color = [0.5, 0.5, 0.6, 0.6 * opacity];

        let thumb_ratio = viewport_size / content_size;
        let thumb_size = (viewport_size * thumb_ratio).max(24.0);
        let max_scroll = (content_size - viewport_size).max(0.0);
        let thumb_pos = if max_scroll > 0.0 {
            ((scroll_pos / max_scroll) * (viewport_size - thumb_size)).round()
        } else {
            0.0
        };

        if is_vertical {
            let track_rect = Rect {
                x: rect.x + rect.width - sb_w - 2.0,
                y: rect.y + 2.0,
                width: sb_w,
                height: rect.height - 4.0,
            };
            renderer.fill_rounded_rect(track_rect, sb_w / 2.0, track_color);

            let thumb_rect = Rect {
                x: rect.x + rect.width - sb_w - 2.0,
                y: rect.y + 2.0 + thumb_pos,
                width: sb_w,
                height: thumb_size,
            };
            renderer.fill_rounded_rect(thumb_rect, sb_w / 2.0, thumb_color);
        } else {
            let track_rect = Rect {
                x: rect.x + 2.0,
                y: rect.y + rect.height - sb_w - 2.0,
                width: rect.width - 4.0,
                height: sb_w,
            };
            renderer.fill_rounded_rect(track_rect, sb_w / 2.0, track_color);

            let thumb_rect = Rect {
                x: rect.x + 2.0 + thumb_pos,
                y: rect.y + rect.height - sb_w - 2.0,
                width: thumb_size,
                height: sb_w,
            };
            renderer.fill_rounded_rect(thumb_rect, sb_w / 2.0, thumb_color);
        }
    }
}

impl<V: View> View for ScrollView<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let content_w = self.content_size[0];
        let content_h = self.content_size[1];
        let dt = renderer.delta_time();

        let mut state = self.read_state();

        Self::tick_physics(
            &mut state,
            rect.width,
            rect.height,
            content_w,
            content_h,
            dt,
            self.momentum_decay,
        );

        let is_moving = state.momentum_velocity[0].abs() > 0.05
            || state.momentum_velocity[1].abs() > 0.05
            || state.spring_x.is_some()
            || state.spring_y.is_some()
            || state.zoom_spring.is_some()
            || state.is_dragging;

        if is_moving {
            state.scrollbar_opacity = 1.0;
            state.last_scroll_frame = 0;
            renderer.request_redraw();
        } else {
            state.last_scroll_frame += 1;
            if state.last_scroll_frame > self.scrollbar_fade_delay {
                state.scrollbar_opacity *= self.scrollbar_fade_speed;
                if state.scrollbar_opacity < 0.01 {
                    state.scrollbar_opacity = 0.0;
                } else {
                    renderer.request_redraw();
                }
            }
        }

        self.write_state(state);

        if self.scroll_id != 0 {
            let scroll_id = self.scroll_id;
            let speed = self.scroll_speed;
            let decay = self.momentum_decay;
            let sb_w = self.scrollbar_width;

            renderer.register_handler(
                "pointerwheel",
                std::sync::Arc::new(move |event| {
                    if let Event::PointerWheel {
                        delta_x, delta_y, ..
                    } = event
                    {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            let mut st = s
                                .get_component_state::<ScrollState>(scroll_id)
                                .and_then(|g| g.read().ok().map(|v| *v))
                                .unwrap_or_default();

                            let max_scroll_x = (content_w - rect.width).max(0.0);
                            let max_scroll_y = (content_h - rect.height).max(0.0);

                            let mut dx = delta_x * speed;
                            let mut dy = delta_y * speed;

                            // Apply resistance out of bounds
                            if (st.scroll_offset[0] <= 0.0 && dx < 0.0)
                                || (st.scroll_offset[0] >= max_scroll_x && dx > 0.0)
                            {
                                dx *= 0.35;
                            }
                            if (st.scroll_offset[1] <= 0.0 && dy < 0.0)
                                || (st.scroll_offset[1] >= max_scroll_y && dy > 0.0)
                            {
                                dy *= 0.35;
                            }

                            st.scroll_offset[0] += dx;
                            st.scroll_offset[1] += dy;

                            st.momentum_velocity[0] += dx * 0.5;
                            st.momentum_velocity[1] += dy * 0.5;
                            st.scrollbar_opacity = 1.0;
                            st.last_scroll_frame = 0;
                            s.set_component_state(scroll_id, st);
                            s
                        });
                    }
                }),
            );

            renderer.register_handler(
                "pointerdown",
                std::sync::Arc::new(move |event| {
                    if let Event::PointerDown { x, y, .. } = event {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            let mut st = s
                                .get_component_state::<ScrollState>(scroll_id)
                                .and_then(|g| g.read().ok().map(|v| *v))
                                .unwrap_or_default();

                            let max_scroll_x = (content_w - rect.width).max(0.0);
                            let max_scroll_y = (content_h - rect.height).max(0.0);

                            // Detect vertical scrollbar interaction
                            let track_rect_v = Rect {
                                x: rect.x + rect.width - sb_w - 2.0,
                                y: rect.y + 2.0,
                                width: sb_w,
                                height: rect.height - 4.0,
                            };
                            let is_on_v = x >= track_rect_v.x
                                && x <= track_rect_v.x + track_rect_v.width
                                && y >= track_rect_v.y
                                && y <= track_rect_v.y + track_rect_v.height;

                            // Detect horizontal scrollbar interaction
                            let track_rect_h = Rect {
                                x: rect.x + 2.0,
                                y: rect.y + rect.height - sb_w - 2.0,
                                width: rect.width - 4.0,
                                height: sb_w,
                            };
                            let is_on_h = x >= track_rect_h.x
                                && x <= track_rect_h.x + track_rect_h.width
                                && y >= track_rect_h.y
                                && y <= track_rect_h.y + track_rect_h.height;

                            if is_on_v {
                                let thumb_ratio = rect.height / content_h;
                                let thumb_h = (rect.height * thumb_ratio).max(24.0);
                                let thumb_pos_y = ((st.scroll_offset[1] / max_scroll_y)
                                    * (rect.height - thumb_h))
                                    .round();
                                let thumb_start_y = rect.y + 2.0 + thumb_pos_y;
                                let inside_thumb =
                                    y >= thumb_start_y && y <= thumb_start_y + thumb_h;

                                if inside_thumb {
                                    st.is_scrollbar_dragging_v = true;
                                } else {
                                    let click_y_relative = (y - track_rect_v.y - thumb_h / 2.0)
                                        / (track_rect_v.height - thumb_h);
                                    st.scroll_offset[1] =
                                        (click_y_relative * max_scroll_y).clamp(0.0, max_scroll_y);
                                    st.is_scrollbar_dragging_v = true;
                                }
                                st.last_pointer_pos = [x, y];
                            } else if is_on_h {
                                let thumb_ratio = rect.width / content_w;
                                let thumb_w = (rect.width * thumb_ratio).max(24.0);
                                let thumb_pos_x = ((st.scroll_offset[0] / max_scroll_x)
                                    * (rect.width - thumb_w))
                                    .round();
                                let thumb_start_x = rect.x + 2.0 + thumb_pos_x;
                                let inside_thumb =
                                    x >= thumb_start_x && x <= thumb_start_x + thumb_w;

                                if inside_thumb {
                                    st.is_scrollbar_dragging_h = true;
                                } else {
                                    let click_x_relative = (x - track_rect_h.x - thumb_w / 2.0)
                                        / (track_rect_h.width - thumb_w);
                                    st.scroll_offset[0] =
                                        (click_x_relative * max_scroll_x).clamp(0.0, max_scroll_x);
                                    st.is_scrollbar_dragging_h = true;
                                }
                                st.last_pointer_pos = [x, y];
                            } else {
                                st.is_dragging = true;
                                st.last_pointer_pos = [x, y];
                                st.momentum_velocity = [0.0, 0.0];
                            }

                            st.scrollbar_opacity = 1.0;
                            st.last_scroll_frame = 0;
                            s.set_component_state(scroll_id, st);
                            s
                        });
                    }
                }),
            );

            renderer.register_handler(
                "pointermove",
                std::sync::Arc::new(move |event| {
                    if let Event::PointerMove { x, y, .. } = event {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            let mut st = s
                                .get_component_state::<ScrollState>(scroll_id)
                                .and_then(|g| g.read().ok().map(|v| *v))
                                .unwrap_or_default();

                            let max_scroll_x = (content_w - rect.width).max(0.0);
                            let max_scroll_y = (content_h - rect.height).max(0.0);

                            if st.is_dragging {
                                let mut dx = st.last_pointer_pos[0] - x;
                                let mut dy = st.last_pointer_pos[1] - y;

                                if (st.scroll_offset[0] <= 0.0 && dx < 0.0)
                                    || (st.scroll_offset[0] >= max_scroll_x && dx > 0.0)
                                {
                                    dx *= 0.3;
                                }
                                if (st.scroll_offset[1] <= 0.0 && dy < 0.0)
                                    || (st.scroll_offset[1] >= max_scroll_y && dy > 0.0)
                                {
                                    dy *= 0.3;
                                }

                                st.scroll_offset[0] += dx;
                                st.scroll_offset[1] += dy;

                                st.momentum_velocity[0] = dx * 0.5;
                                st.momentum_velocity[1] = dy * 0.5;
                                st.last_pointer_pos = [x, y];
                            } else if st.is_scrollbar_dragging_v {
                                let track_h = rect.height - 4.0;
                                let thumb_ratio = rect.height / content_h;
                                let thumb_h = (rect.height * thumb_ratio).max(24.0);
                                let max_travel = track_h - thumb_h;
                                if max_travel > 0.0 {
                                    let delta_y = y - st.last_pointer_pos[1];
                                    let scroll_delta = (delta_y / max_travel) * max_scroll_y;
                                    st.scroll_offset[1] = (st.scroll_offset[1] + scroll_delta)
                                        .clamp(0.0, max_scroll_y);
                                    st.last_pointer_pos = [x, y];
                                }
                            } else if st.is_scrollbar_dragging_h {
                                let track_w = rect.width - 4.0;
                                let thumb_ratio = rect.width / content_w;
                                let thumb_w = (rect.width * thumb_ratio).max(24.0);
                                let max_travel = track_w - thumb_w;
                                if max_travel > 0.0 {
                                    let delta_x = x - st.last_pointer_pos[0];
                                    let scroll_delta = (delta_x / max_travel) * max_scroll_x;
                                    st.scroll_offset[0] = (st.scroll_offset[0] + scroll_delta)
                                        .clamp(0.0, max_scroll_x);
                                    st.last_pointer_pos = [x, y];
                                }
                            }

                            st.scrollbar_opacity = 1.0;
                            st.last_scroll_frame = 0;
                            s.set_component_state(scroll_id, st);
                            s
                        });
                    }
                }),
            );

            renderer.register_handler(
                "pointerup",
                std::sync::Arc::new(move |_| {
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        let mut st = s
                            .get_component_state::<ScrollState>(scroll_id)
                            .and_then(|g| g.read().ok().map(|v| *v))
                            .unwrap_or_default();
                        st.is_dragging = false;
                        st.is_scrollbar_dragging_v = false;
                        st.is_scrollbar_dragging_h = false;
                        st.momentum_velocity[0] *= decay;
                        st.momentum_velocity[1] *= decay;
                        s.set_component_state(scroll_id, st);
                        s
                    });
                }),
            );

            renderer.register_handler(
                "gesturepinch",
                std::sync::Arc::new(move |event| {
                    if let Event::GesturePinch {
                        center,
                        scale,
                        velocity,
                        phase,
                    } = event
                    {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            let mut st = s
                                .get_component_state::<ScrollState>(scroll_id)
                                .and_then(|g| g.read().ok().map(|v| *v))
                                .unwrap_or_default();

                            // Zoom anchor: use the actual pinch center so content under
                            // the fingers stays stationary during zoom.
                            let pinch_center_x = center[0];
                            let pinch_center_y = center[1];

                            // Accumulate zoom from the pinch scale delta.
                            // A scale > 1.0 means zoom in, < 1.0 means zoom out.
                            // We apply it relative to the current zoom and add momentum
                            // from the pinch velocity so the zoom feels responsive.
                            let zoom_delta = scale - 1.0;
                            let velocity_boost = velocity * 0.1;
                            let new_zoom = st.zoom_level * (1.0 + zoom_delta + velocity_boost);
                            st.zoom_level = new_zoom.clamp(st.min_zoom, st.max_zoom);

                            // Adjust scroll offset to zoom toward the pinch center.
                            // This keeps the content under the fingers stationary.
                            let zoom_ratio = st.zoom_level / (st.zoom_level - zoom_delta).max(0.01);
                            st.scroll_offset[0] = pinch_center_x
                                - (pinch_center_x - st.scroll_offset[0]) * zoom_ratio;
                            st.scroll_offset[1] = pinch_center_y
                                - (pinch_center_y - st.scroll_offset[1]) * zoom_ratio;

                            // Cancel momentum while actively pinching
                            st.momentum_velocity = [0.0, 0.0];
                            st.scrollbar_opacity = 1.0;
                            st.last_scroll_frame = 0;

                            // On pinch end/cancel, snap zoom to clean value
                            if phase == cvkg_core::TouchPhase::Ended
                                || phase == cvkg_core::TouchPhase::Cancelled
                            {
                                st.zoom_level = (st.zoom_level * 4.0).round() / 4.0;
                                st.momentum_velocity = [0.0, 0.0];
                            }

                            s.set_component_state(scroll_id, st);
                            s
                        });
                    }
                }),
            );

            renderer.register_handler(
                "keydown",
                std::sync::Arc::new(move |event| {
                    if let Event::KeyDown { key, .. } = event {
                        let scroll_amount = match key.as_str() {
                            "PageDown" => 100.0,
                            "PageUp" => -100.0,
                            "End" => f32::MAX,
                            "Home" => f32::MIN,
                            "ArrowDown" => 40.0,
                            "ArrowUp" => -40.0,
                            "ArrowRight" => 40.0,
                            "ArrowLeft" => -40.0,
                            _ => return,
                        };
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            let mut st = s
                                .get_component_state::<ScrollState>(scroll_id)
                                .and_then(|g| g.read().ok().map(|v| *v))
                                .unwrap_or_default();
                            let max_y = (content_h - rect.height).max(0.0);
                            let max_x = (content_w - rect.width).max(0.0);
                            if scroll_amount == f32::MAX {
                                st.scroll_offset[1] = max_y;
                            } else if scroll_amount == f32::MIN {
                                st.scroll_offset[0] = 0.0;
                                st.scroll_offset[1] = 0.0;
                            } else {
                                match key.as_str() {
                                    "ArrowLeft" | "ArrowRight" => {
                                        st.scroll_offset[0] =
                                            (st.scroll_offset[0] + scroll_amount).clamp(0.0, max_x);
                                    }
                                    _ => {
                                        st.scroll_offset[1] =
                                            (st.scroll_offset[1] + scroll_amount).clamp(0.0, max_y);
                                    }
                                }
                            }
                            st.momentum_velocity = [0.0, 0.0];
                            st.scrollbar_opacity = 1.0;
                            st.last_scroll_frame = 0;
                            s.set_component_state(scroll_id, st);
                            s
                        });
                    }
                }),
            );
        }

        renderer.push_clip_rect(rect);
        // Apply both scroll offset and pinch-zoom transform.
        // Zoom is centered around the viewport center so content under the fingers stays put.
        let zoom = state.zoom_level;
        let cx = rect.x + rect.width / 2.0;
        let cy = rect.y + rect.height / 2.0;
        renderer.push_transform(
            [
                -state.scroll_offset[0].round() + cx - cx * zoom,
                -state.scroll_offset[1].round() + cy - cy * zoom,
            ],
            [zoom, zoom],
            0.0,
        );

        let content_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: if content_w > 0.0 {
                content_w
            } else {
                rect.width
            },
            height: if content_h > 0.0 {
                content_h
            } else {
                rect.height
            },
        };
        self.content.render(renderer, content_rect);

        renderer.pop_transform();
        renderer.pop_clip_rect();

        let needs_v_scrollbar = content_h > rect.height;
        let needs_h_scrollbar = content_w > rect.width;

        if needs_v_scrollbar {
            self.render_scrollbar(
                renderer,
                rect,
                content_h,
                rect.height,
                state.scroll_offset[1],
                state.scrollbar_opacity,
                true,
            );
        }

        if needs_h_scrollbar {
            self.render_scrollbar(
                renderer,
                rect,
                content_w,
                rect.width,
                state.scroll_offset[0],
                state.scrollbar_opacity,
                false,
            );
        }
    }
}

/// Multi-column table layout
/// Settings style grouped form layout
pub struct SettingsForm<V> {
    pub(crate) content: V,
}

impl<V: View> SettingsForm<V> {
    pub fn new(content: V) -> Self {
        Self { content }
    }
}

impl<V: View> View for SettingsForm<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        self.content.render(renderer, rect);
    }
}

/// A vertical stack of views
#[derive(Clone)]
pub struct VStack {
    spacing: f32,
    alignment: cvkg_core::Alignment,
    distribution: cvkg_core::Distribution,
    children: Vec<cvkg_core::AnyView>,
}

impl VStack {
    pub fn new(spacing: f32) -> Self {
        Self {
            spacing,
            alignment: cvkg_core::Alignment::Center,
            distribution: cvkg_core::Distribution::Fill,
            children: Vec::new(),
        }
    }

    pub fn alignment(mut self, alignment: cvkg_core::Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn distribution(mut self, distribution: cvkg_core::Distribution) -> Self {
        self.distribution = distribution;
        self
    }

    pub fn child<V: View + Clone + 'static>(mut self, view: V) -> Self {
        self.children.push(view.erase());
        self
    }
}

impl View for VStack {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.push_vnode(rect, "VStack");
        if self.children.is_empty() {
            renderer.pop_vnode();
            return;
        }

        let mut cache = LayoutCache::new();
        let layouts: Vec<&dyn LayoutView> =
            self.children.iter().filter_map(|c| c.layout()).collect();

        let rects = cvkg_layout::VStack::compute_layout(
            self.spacing,
            self.alignment,
            self.distribution,
            rect,
            &layouts,
            &mut cache,
        );

        let mut rect_idx = 0;
        for child in self.children.iter() {
            if child.layout().is_some() && rect_idx < rects.len() {
                child.render(renderer, rects[rect_idx]);
                rect_idx += 1;
            }
        }
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in self.children.iter().enumerate() {
            let child_size = child.intrinsic_size(renderer, proposal);
            width = width.max(child_size.width);
            height += child_size.height;
            if i < self.children.len() - 1 {
                height += self.spacing;
            }
        }

        Size { width, height }
    }

    fn layout(&self) -> Option<&dyn LayoutView> {
        Some(self)
    }
}

impl LayoutView for VStack {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in self.children.iter().enumerate() {
            if let Some(layout) = child.layout() {
                let child_size = layout.size_that_fits(proposal, &[], cache);
                width = width.max(child_size.width);
                height += child_size.height;
                if i < self.children.len() - 1 {
                    height += self.spacing;
                }
            }
        }

        Size { width, height }
    }

    fn place_subviews(
        &self,
        bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    ) {
        let layouts: Vec<&dyn LayoutView> =
            self.children.iter().filter_map(|c| c.layout()).collect();

        let _rects = cvkg_layout::VStack::compute_layout(
            self.spacing,
            self.alignment,
            self.distribution,
            bounds,
            &layouts,
            cache,
        );
    }
}

/// A vertical stack that only renders visible children (efficient for long lists)
#[derive(Clone)]
pub struct LazyVStack {
    spacing: f32,
    children: Vec<cvkg_core::AnyView>,
}

impl LazyVStack {
    pub fn new(spacing: f32) -> Self {
        Self {
            spacing,
            children: Vec::new(),
        }
    }

    pub fn child<V: View + Clone + 'static>(mut self, view: V) -> Self {
        self.children.push(view.erase());
        self
    }
}

impl View for LazyVStack {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.children.is_empty() {
            return;
        }

        let clip = renderer.current_clip_rect();
        let viewport_y = clip.y.max(rect.y);
        let viewport_bottom = (clip.y + clip.height).min(rect.y + rect.height);

        if viewport_bottom <= viewport_y {
            return;
        }

        let child_height = 40.0;

        // Calculate indices based on fixed height for simplicity in LazyVStack
        let start_idx = ((viewport_y - rect.y) / (child_height + self.spacing)).floor() as usize;
        let visible_count =
            ((viewport_bottom - viewport_y) / (child_height + self.spacing)).ceil() as usize;
        let end_idx = (start_idx + visible_count + 1).min(self.children.len());

        for idx in start_idx..end_idx {
            let child = &self.children[idx];
            let child_y = rect.y + idx as f32 * (child_height + self.spacing);

            child.render(
                renderer,
                Rect {
                    x: rect.x,
                    y: child_y,
                    width: rect.width,
                    height: child_height,
                },
            );
        }
    }
}

/// A horizontal stack of views
#[derive(Clone)]
pub struct HStack {
    spacing: f32,
    alignment: cvkg_core::Alignment,
    distribution: cvkg_core::Distribution,
    children: Vec<cvkg_core::AnyView>,
}

impl HStack {
    pub fn new(spacing: f32) -> Self {
        Self {
            spacing,
            alignment: cvkg_core::Alignment::Center,
            distribution: cvkg_core::Distribution::Fill,
            children: Vec::new(),
        }
    }

    pub fn alignment(mut self, alignment: cvkg_core::Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn distribution(mut self, distribution: cvkg_core::Distribution) -> Self {
        self.distribution = distribution;
        self
    }

    pub fn child<V: View + Clone + 'static>(mut self, view: V) -> Self {
        self.children.push(view.erase());
        self
    }
}

impl View for HStack {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        if self.children.is_empty() {
            return;
        }

        let mut cache = LayoutCache::new();
        let layouts: Vec<&dyn LayoutView> =
            self.children.iter().filter_map(|c| c.layout()).collect();

        let rects = cvkg_layout::HStack::compute_layout(
            self.spacing,
            self.alignment,
            self.distribution,
            rect,
            &layouts,
            &mut cache,
        );

        let mut rect_idx = 0;
        for child in self.children.iter() {
            if child.layout().is_some() && rect_idx < rects.len() {
                child.render(renderer, rects[rect_idx]);
                rect_idx += 1;
            }
        }
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in self.children.iter().enumerate() {
            let child_size = child.intrinsic_size(renderer, proposal);
            width += child_size.width;
            height = height.max(child_size.height);
            if i < self.children.len() - 1 {
                width += self.spacing;
            }
        }

        Size { width, height }
    }

    fn layout(&self) -> Option<&dyn LayoutView> {
        Some(self)
    }
}

impl LayoutView for HStack {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in self.children.iter().enumerate() {
            if let Some(layout) = child.layout() {
                let child_size = layout.size_that_fits(proposal, &[], cache);
                width += child_size.width;
                height = height.max(child_size.height);
                if i < self.children.len() - 1 {
                    width += self.spacing;
                }
            }
        }

        Size { width, height }
    }

    fn place_subviews(
        &self,
        bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    ) {
        let layouts: Vec<&dyn LayoutView> =
            self.children.iter().filter_map(|c| c.layout()).collect();

        let _rects = cvkg_layout::HStack::compute_layout(
            self.spacing,
            self.alignment,
            self.distribution,
            bounds,
            &layouts,
            cache,
        );
    }
}

/// A flexible container that defaults to a glassmorphic construct over a void black background
pub struct FlexBox {
    pub orientation: cvkg_core::Orientation,
    pub spacing: f32,
    children: Vec<cvkg_core::AnyView>,
}

impl FlexBox {
    pub fn new(orientation: cvkg_core::Orientation, spacing: f32) -> Self {
        Self {
            orientation,
            spacing,
            children: Vec::new(),
        }
    }

    pub fn child<V: View + Clone + 'static>(mut self, view: V) -> Self {
        self.children.push(view.erase());
        self
    }
}

impl View for FlexBox {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.fill_rounded_rect(rect, 8.0, [0.0, 0.0, 0.0, 0.85]);
        renderer.stroke_rect(rect, [0.2, 0.2, 0.25, 0.5], 1.0);
        renderer.bifrost(rect, 15.0, 1.2, 0.85);

        if self.children.is_empty() {
            return;
        }

        let n = self.children.len() as f32;
        match self.orientation {
            cvkg_core::Orientation::Horizontal => {
                let total_spacing = self.spacing * (n - 1.0);
                let item_width = (rect.width - total_spacing) / n;
                for (i, child) in self.children.iter().enumerate() {
                    let child_rect = Rect {
                        x: rect.x + i as f32 * (item_width + self.spacing),
                        y: rect.y,
                        width: item_width,
                        height: rect.height,
                    };
                    child.render(renderer, child_rect);
                }
            }
            cvkg_core::Orientation::Vertical => {
                let total_spacing = self.spacing * (n - 1.0);
                let item_height = (rect.height - total_spacing) / n;
                for (i, child) in self.children.iter().enumerate() {
                    let child_rect = Rect {
                        x: rect.x,
                        y: rect.y + i as f32 * (item_height + self.spacing),
                        width: rect.width,
                        height: item_height,
                    };
                    child.render(renderer, child_rect);
                }
            }
        }
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (i, child) in self.children.iter().enumerate() {
            let child_size = child.intrinsic_size(renderer, proposal);
            match self.orientation {
                cvkg_core::Orientation::Horizontal => {
                    width += child_size.width;
                    height = height.max(child_size.height);
                    if i < self.children.len() - 1 {
                        width += self.spacing;
                    }
                }
                cvkg_core::Orientation::Vertical => {
                    width = width.max(child_size.width);
                    height += child_size.height;
                    if i < self.children.len() - 1 {
                        height += self.spacing;
                    }
                }
            }
        }

        Size { width, height }
    }
}

/// Popover component for displaying rich content in a floating bubble.
#[deprecated(note = "Use popover::Popover instead")]
pub use crate::popover::Popover;

const COLLAPSIBLE_ANIM_HASH: u64 = 0xD00_0001;

/// Collapsible component for hiding/showing content with animation.
pub struct Collapsible<V> {
    pub(crate) content: V,
    pub(crate) header: String,
    pub(crate) is_open: bool,
}

impl<V: View> Collapsible<V> {
    pub fn new(header: impl Into<String>, content: V, is_open: bool) -> Self {
        Self {
            header: header.into(),
            content,
            is_open,
        }
    }
}

impl<V: View> View for Collapsible<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Collapsible");

        let header_h: f32 = 40.0;
        let header_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: header_h,
        };

        // ── Header bar ──
        renderer.fill_rounded_rect(header_rect, 6.0, [0.08, 0.08, 0.12, 0.9]);
        renderer.stroke_rounded_rect(header_rect, 6.0, [0.2, 0.2, 0.3, 0.5], 1.0);

        // Arrow indicator
        let arrow = if self.is_open { "▼" } else { "▶" };
        let accent = if self.is_open {
            theme::accent()
        } else {
            theme::text_muted()
        };
        renderer.draw_text(arrow, rect.x + 10.0, rect.y + 12.0, 12.0, accent);
        renderer.draw_text(
            &self.header,
            rect.x + 30.0,
            rect.y + 10.0,
            FONT_BASE + 2.0,
            [1.0, 1.0, 1.0, 0.95],
        );

        // ── Click-to-toggle handler ──
        let hdr = header_rect;
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event| {
                if let Event::PointerClick { x, y, .. } = event
                    && hdr.contains(x, y)
                {
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        let current: bool = s
                            .get_component_state::<bool>(COLLAPSIBLE_ANIM_HASH)
                            .and_then(|v| v.read().ok().map(|v| *v))
                            .unwrap_or(false);
                        s.set_component_state(COLLAPSIBLE_ANIM_HASH, !current);
                        s
                    });
                }
            }),
        );

        // ── Animated content area ──
        let anim_open: bool = cvkg_core::load_system_state()
            .get_component_state::<bool>(COLLAPSIBLE_ANIM_HASH)
            .and_then(|v| v.read().ok().map(|v| *v))
            .unwrap_or(self.is_open);

        if anim_open {
            // Animate height: advance toward 1.0 when open, 0.0 when closed
            let prev_h: f32 = cvkg_core::load_system_state()
                .get_component_state::<f32>(COLLAPSIBLE_ANIM_HASH + 1)
                .and_then(|v| v.read().ok().map(|v| *v))
                .unwrap_or(0.0);
            let target: f32 = if anim_open { 1.0 } else { 0.0 };
            let new_h = prev_h + (target - prev_h) * 0.2;
            cvkg_core::update_system_state(move |s| {
                let mut s = s.clone();
                s.set_component_state(COLLAPSIBLE_ANIM_HASH + 1, new_h);
                s
            });

            let max_content_h = rect.height - header_h - 4.0;
            let content_h = max_content_h * new_h;
            if content_h > 1.0 {
                let content_rect = Rect {
                    x: rect.x + 4.0,
                    y: rect.y + header_h + 4.0,
                    width: rect.width - 8.0,
                    height: content_h,
                };
                renderer.fill_rounded_rect(content_rect, 4.0, [0.04, 0.04, 0.07, 0.4]);
                self.content.render(renderer, content_rect);
            }
        }

        renderer.pop_vnode();
    }
}
/// GjallarSplitter - A draggable split pane component.
/// Named after the Gjallarhorn, which signals boundaries and transitions.
pub struct GjallarSplitter<V1: View, V2: View> {
    pub first: V1,
    pub second: V2,
    pub split_ratio: f32, // 0.0 to 1.0
    pub orientation: cvkg_core::Orientation,
}

const SPLITTER_DRAG_HASH: u64 = 0xC00_0001;
const SPLITTER_RATIO_HASH: u64 = 0xC00_0002;

impl<V1: View, V2: View> GjallarSplitter<V1, V2> {
    /// Creates a new GjallarSplitter with the given views.
    pub fn new(first: V1, second: V2) -> Self {
        Self {
            first,
            second,
            split_ratio: 0.5,
            orientation: cvkg_core::Orientation::Horizontal,
        }
    }

    /// Sets the split ratio (clamped to 0.1..=0.9).
    pub fn split_ratio(mut self, ratio: f32) -> Self {
        self.split_ratio = ratio.clamp(0.1, 0.9);
        self
    }

    /// Sets the orientation of the split.
    pub fn orientation(mut self, orientation: cvkg_core::Orientation) -> Self {
        self.orientation = orientation;
        self
    }
}

impl<V1: View, V2: View> View for GjallarSplitter<V1, V2> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "GjallarSplitter");

        // Read the live split ratio from system state (updated by drag)
        let live_ratio: f32 = cvkg_core::load_system_state()
            .get_component_state::<f32>(SPLITTER_RATIO_HASH)
            .and_then(|v| v.read().ok().map(|v| *v))
            .unwrap_or(self.split_ratio)
            .clamp(0.1, 0.9);

        let handle_size = 4.0;
        let (first_rect, handle_rect, second_rect) = match self.orientation {
            cvkg_core::Orientation::Horizontal => {
                let w1 = rect.width * live_ratio - handle_size / 2.0;
                let r1 = Rect {
                    x: rect.x,
                    y: rect.y,
                    width: w1,
                    height: rect.height,
                };
                let rh = Rect {
                    x: rect.x + w1,
                    y: rect.y,
                    width: handle_size,
                    height: rect.height,
                };
                let r2 = Rect {
                    x: rect.x + w1 + handle_size,
                    y: rect.y,
                    width: rect.width - w1 - handle_size,
                    height: rect.height,
                };
                (r1, rh, r2)
            }
            cvkg_core::Orientation::Vertical => {
                let h1 = rect.height * live_ratio - handle_size / 2.0;
                let r1 = Rect {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: h1,
                };
                let rh = Rect {
                    x: rect.x,
                    y: rect.y + h1,
                    width: rect.width,
                    height: handle_size,
                };
                let r2 = Rect {
                    x: rect.x,
                    y: rect.y + h1 + handle_size,
                    width: rect.width,
                    height: rect.height - h1 - handle_size,
                };
                (r1, rh, r2)
            }
        };

        // 1. Render Views
        self.first.render(renderer, first_rect);
        self.second.render(renderer, second_rect);

        // 2. Render Split Handle
        let is_dragging: bool = cvkg_core::load_system_state()
            .get_component_state::<bool>(SPLITTER_DRAG_HASH)
            .and_then(|v| v.read().ok().map(|v| *v))
            .unwrap_or(false);
        let handle_color = if is_dragging {
            [0.0, 0.8, 1.0, 0.5]
        } else {
            theme::surface_elevated()
        };
        renderer.fill_rect(handle_rect, handle_color);
        renderer.stroke_rect(
            handle_rect,
            [0.0, 0.8, 1.0, if is_dragging { 0.8 } else { 0.4 }],
            1.0,
        );

        // 3. Handle Center Glow (Mimir's Eye)
        let center_x = handle_rect.x + handle_rect.width / 2.0;
        let center_y = handle_rect.y + handle_rect.height / 2.0;
        renderer.fill_rounded_rect(
            Rect {
                x: center_x - 1.0,
                y: center_y - 10.0,
                width: 2.0,
                height: 20.0,
            },
            1.0,
            [0.0, 1.0, 1.0, 0.8],
        );

        // 4. Drag interaction: pointerdown on handle starts drag
        let h_rect = handle_rect;
        let _orient = self.orientation;
        let _full_rect = rect;
        renderer.register_handler(
            "pointerdown",
            Arc::new(move |event| {
                if let Event::PointerDown { x, y, .. } = event
                    && h_rect.contains(x, y)
                {
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        s.set_component_state(SPLITTER_DRAG_HASH, true);
                        s
                    });
                }
            }),
        );

        // 5. pointermove updates split_ratio while dragging
        let full_rect2 = rect;
        let orient2 = self.orientation;
        renderer.register_handler(
            "pointermove",
            Arc::new(move |event| {
                if let Event::PointerMove { x, y, .. } = event {
                    let dragging: bool = cvkg_core::load_system_state()
                        .get_component_state::<bool>(SPLITTER_DRAG_HASH)
                        .and_then(|v| v.read().ok().map(|v| *v))
                        .unwrap_or(false);
                    if dragging {
                        let new_ratio = match orient2 {
                            cvkg_core::Orientation::Horizontal => {
                                (x - full_rect2.x) / full_rect2.width
                            }
                            cvkg_core::Orientation::Vertical => {
                                (y - full_rect2.y) / full_rect2.height
                            }
                        };
                        let clamped = new_ratio.clamp(0.1, 0.9);
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            s.set_component_state(SPLITTER_RATIO_HASH, clamped);
                            s
                        });
                    }
                }
            }),
        );

        // 6. pointerup ends drag
        renderer.register_handler(
            "pointerup",
            Arc::new(move |event| {
                if let Event::PointerUp { .. } = event {
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        s.set_component_state(SPLITTER_DRAG_HASH, false);
                        s
                    });
                }
            }),
        );

        renderer.pop_vnode();
    }
}

/// SagaAccordion - A collapsible accordion component for revealing stories (data).
/// Named after the Sagas, the epic narratives of the Norse.
#[derive(Clone)]
pub struct SagaAccordion<V: View> {
    pub items: Vec<SagaItem<V>>,
    pub allow_multiple: bool,
}

#[derive(Clone)]
pub struct SagaItem<V: View> {
    pub title: String,
    pub content: V,
    pub is_expanded: bool,
}

impl<V: View> SagaAccordion<V> {
    /// Creates a new SagaAccordion.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            allow_multiple: false,
        }
    }

    /// Adds an item to the accordion.
    pub fn item(mut self, title: impl Into<String>, content: V) -> Self {
        self.items.push(SagaItem {
            title: title.into(),
            content,
            is_expanded: false,
        });
        self
    }

    /// Sets whether multiple items can be expanded at once.
    pub fn allow_multiple(mut self, allow: bool) -> Self {
        self.allow_multiple = allow;
        self
    }
}

impl<V: View> View for SagaAccordion<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "SagaAccordion");

        let t = renderer.elapsed_time();
        let mut current_y = rect.y;
        let item_h = 44.0; // Slightly larger for tactical feel

        // System-state hash key for tracking expanded items (Vec<bool>).
        // Derived from a fixed string so it is stable across renders.
        let state_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            "saga_accordion_expanded".hash(&mut s);
            s.finish()
        };

        // Read current expanded state from system state
        let mut expanded_state: Vec<bool> = {
            let s = cvkg_core::load_system_state();
            s.get_component_state::<Vec<bool>>(state_hash)
                .and_then(|v| v.read().ok().map(|g| g.clone()))
                .unwrap_or_else(|| self.items.iter().map(|item| item.is_expanded).collect())
        };

        // Ensure the vec length matches items count
        while expanded_state.len() < self.items.len() {
            expanded_state.push(false);
        }

        for (i, item) in self.items.iter().enumerate() {
            let is_expanded = expanded_state.get(i).copied().unwrap_or(false);

            let header_rect = Rect {
                x: rect.x,
                y: current_y,
                width: rect.width,
                height: item_h,
            };

            // 1. Mimir's Refraction (Glass Header)
            renderer.bifrost(header_rect, 4.0, 1.2, 0.9);
            renderer.fill_rounded_rect(header_rect, 4.0, [0.1, 0.1, 0.15, 0.7]);

            // Surtur's Reactive Materials (Hover/Selection Glow)
            if is_expanded {
                let pulse = (t * 3.0 + i as f32).sin() * 0.1 + 0.9;
                renderer.stroke_rounded_rect(header_rect, 4.0, [0.0, 0.8, 1.0, 0.4 * pulse], 1.5);
            } else {
                renderer.stroke_rounded_rect(header_rect, 4.0, [0.3, 0.3, 0.4, 0.3], 1.0);
            }

            let arrow = if is_expanded { "▼" } else { "▶" };
            let accent_color = if is_expanded {
                theme::accent()
            } else {
                theme::text_muted()
            };

            renderer.draw_text(
                arrow,
                header_rect.x + 12.0,
                header_rect.y + 14.0,
                12.0,
                accent_color,
            );
            renderer.draw_text(
                &item.title,
                header_rect.x + 36.0,
                header_rect.y + 14.0,
                14.0,
                [1.0, 1.0, 1.0, 0.95],
            );

            // Click-to-toggle: pointerclick handler on header
            let hdr_x = header_rect.x;
            let hdr_y = header_rect.y;
            let hdr_w = header_rect.width;
            let hdr_h = header_rect.height;
            let idx = i;
            let allow_multi = self.allow_multiple;
            renderer.register_handler(
                "pointerdown",
                Arc::new(move |event| {
                    if let Event::PointerDown { x, y, .. } = event
                        && x >= hdr_x
                        && x <= hdr_x + hdr_w
                        && y >= hdr_y
                        && y <= hdr_y + hdr_h
                    {
                        cvkg_core::update_system_state(|s| {
                            let mut s = s.clone();
                            let mut state: Vec<bool> = s
                                .get_component_state::<Vec<bool>>(state_hash)
                                .and_then(|v| v.read().ok().map(|g| g.clone()))
                                .unwrap_or_default();
                            while state.len() <= idx {
                                state.push(false);
                            }
                            // Toggle the clicked item
                            state[idx] = !state[idx];
                            // When allow_multiple=false, close all others
                            if !allow_multi && state[idx] {
                                for (j, v) in state.iter_mut().enumerate() {
                                    if j != idx {
                                        *v = false;
                                    }
                                }
                            }
                            s.set_component_state(state_hash, state);
                            s
                        });
                    }
                }),
            );

            current_y += item_h + 4.0;

            // 2. Content (Unfolded Saga)
            if is_expanded {
                let content_h = 120.0; // Dynamic height would be better but keeping simple for now
                let content_rect = Rect {
                    x: rect.x + 12.0,
                    y: current_y,
                    width: rect.width - 24.0,
                    height: content_h,
                };

                // Subtle content background
                renderer.fill_rounded_rect(content_rect, 4.0, [0.05, 0.05, 0.08, 0.3]);
                item.content.render(renderer, content_rect);

                current_y += content_h + 8.0;
            }
        }

        renderer.pop_vnode();
    }
}
