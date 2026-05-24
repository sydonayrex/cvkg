use crate::{draw_focus_ring, FONT_BASE, RADIUS_LG, RADIUS_MD};
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{Event, Never, Rect, Renderer, Size, View};
use crate::theme;
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
        Self { sidebar, detail, is_collapsed: false, sidebar_width: 300.0 }
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
        renderer.stroke_rect(handle_rect, [0.0, 0.8, 1.0, if is_hover_handle { 0.6 } else { 0.2 }], 1.0);

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
                if let Event::PointerClick { x, y, .. } = event {
                    if tglm.contains(x, y) {
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
                }
            }),
        );

        // Keyboard: Ctrl+B to toggle sidebar collapse
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    if key == "b" || key == "B" {
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
                }
            }),
        );

        // ── Detail area ──
        self.detail.render(renderer, detail_rect);
    }
}

/// Tab bar navigation view
pub struct TabView<V> {
    pub(crate) content: V,
}

impl<V: View> TabView<V> {
    pub fn new(content: V) -> Self {
        Self { content }
    }
}

impl<V: View> View for TabView<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let tab_bar_height = 50.0;
        let content_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height - tab_bar_height,
        };
        let tab_bar_rect = Rect {
            x: rect.x,
            y: rect.y + rect.height - tab_bar_height,
            width: rect.width,
            height: tab_bar_height,
        };

        // Render content
        self.content.render(renderer, content_rect);

        // Render tab bar background
        renderer.bifrost(tab_bar_rect, 10.0, 1.2, 0.9);
        renderer.fill_rect(tab_bar_rect, theme::shadow());
        renderer.draw_line(
            tab_bar_rect.x,
            tab_bar_rect.y,
            tab_bar_rect.x + tab_bar_rect.width,
            tab_bar_rect.y,
            theme::text_dim(),
            1.0,
        );
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

        // ── Animation progress from system state ──
        let mut anim: f32 = cvkg_core::load_system_state()
            .get_component_state::<f32>(SHEET_ANIM_HASH)
            .and_then(|g| g.read().ok().map(|v| *v))
            .unwrap_or(0.0);
        anim = (anim + 0.08).min(1.0);
        cvkg_core::update_system_state({
            let anim = anim;
            move |s| {
                let mut s = s.clone();
                s.set_component_state(SHEET_ANIM_HASH, anim);
                s
            }
        });

        let sheet_rect = match self.position {
            SheetPosition::Left => {
                let w = self.width * anim;
                Rect { x: rect.x, y: rect.y, width: w, height: rect.height }
            }
            SheetPosition::Right => {
                let w = self.width * anim;
                Rect { x: rect.x + rect.width - w, y: rect.y, width: w, height: rect.height }
            }
            SheetPosition::Top => {
                let h = self.height * anim;
                Rect { x: rect.x, y: rect.y, width: rect.width, height: h }
            }
            SheetPosition::Bottom => {
                let h = self.height * anim;
                Rect { x: rect.x, y: rect.y + rect.height - h, width: rect.width, height: h }
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
        renderer.draw_text("×", close_rect.x + 9.0, close_rect.y + 7.0, FONT_BASE + 2.0, theme::text_muted());

        let close_cb = self.on_dismiss.clone();
        // Dismiss: clicking the close button
        renderer.register_handler(
            "pointerdown",
            Arc::new(move |event| {
                if let Event::PointerDown { x, y, .. } = event {
                    if close_rect.contains(x, y) {
                        if let Some(ref cb) = close_cb {
                            (cb)();
                        }
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            s.set_component_state(SHEET_ANIM_HASH, 0.0);
                            s
                        });
                    }
                }
            }),
        );

        // Dismiss: clicking the backdrop (outside the panel)
        let sheet_rect_capture = sheet_rect;
        let dismiss_cb = self.on_dismiss.clone();
        renderer.register_handler(
            "pointerdown",
            Arc::new(move |event| {
                if let Event::PointerDown { x, y, .. } = event {
                    if !sheet_rect_capture.contains(x, y) {
                        if let Some(ref cb) = dismiss_cb {
                            (cb)();
                        }
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            s.set_component_state(SHEET_ANIM_HASH, 0.0);
                            s
                        });
                    }
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
                    if let Event::PointerClick { x, y, .. } = event {
                        if action_rect.contains(x, y) {
                            (on_click)();
                        }
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
                                .map(|v| *v.read().unwrap())
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
                        if let Some(idx) = focused {
                            if let Some(cb) = action_callbacks.get(idx) {
                                (cb)();
                            }
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

pub struct GarmAlert {
    pub(crate) is_presented: bool,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) on_confirm: std::sync::Arc<dyn Fn() + Send + Sync>,
    pub(crate) on_cancel: std::sync::Arc<dyn Fn() + Send + Sync>,
}

impl GarmAlert {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            is_presented: false,
            title: title.into(),
            description: description.into(),
            on_confirm: std::sync::Arc::new(|| {}),
            on_cancel: std::sync::Arc::new(|| {}),
        }
    }

    pub fn presented(mut self, is_presented: bool) -> Self {
        self.is_presented = is_presented;
        self
    }

    pub fn on_confirm(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_confirm = std::sync::Arc::new(callback);
        self
    }

    pub fn on_cancel(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_cancel = std::sync::Arc::new(callback);
        self
    }
}

impl View for GarmAlert {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.is_presented {
            return;
        }

        renderer.fill_rect(rect, theme::shadow());
        let modal_w = 400.0;
        let modal_h = 200.0;
        let modal_rect = Rect {
            x: rect.x + (rect.width - modal_w) / 2.0,
            y: rect.y + (rect.height - modal_h) / 2.0,
            width: modal_w,
            height: modal_h,
        };

        renderer.bifrost(modal_rect, 20.0, 1.2, 0.9);
        renderer.fill_rounded_rect(modal_rect, 12.0, [0.08, 0.08, 0.1, 0.9]);
        renderer.stroke_rounded_rect(modal_rect, 12.0, [1.0, 0.2, 0.2, 0.6], 2.0);

        renderer.draw_text(
            &self.title,
            modal_rect.x + 20.0,
            modal_rect.y + 20.0,
            22.0,
            theme::text(),
        );
        renderer.draw_text(
            &self.description,
            modal_rect.x + 20.0,
            modal_rect.y + 55.0,
            14.0,
            theme::text_muted(),
        );

        let btn_w = 100.0;
        let btn_h = 36.0;
        let cancel_rect = Rect {
            x: modal_rect.x + modal_w - 230.0,
            y: modal_rect.y + modal_h - 56.0,
            width: btn_w,
            height: btn_h,
        };
        let confirm_rect = Rect {
            x: modal_rect.x + modal_w - 120.0,
            y: modal_rect.y + modal_h - 56.0,
            width: btn_w,
            height: btn_h,
        };

        renderer.fill_rounded_rect(cancel_rect, 6.0, theme::border_strong());
        renderer.draw_text(
            "Cancel",
            cancel_rect.x + 25.0,
            cancel_rect.y + 10.0,
            14.0,
            theme::text(),
        );

        renderer.fill_rounded_rect(confirm_rect, 6.0, theme::error_color());
        renderer.draw_text(
            "Confirm",
            confirm_rect.x + 20.0,
            confirm_rect.y + 10.0,
            14.0,
            theme::text(),
        );
    }
}

/// Context menu dropdown
pub struct Menu<V> {
    pub(crate) content: V,
}

impl<V: View> Menu<V> {
    pub fn new(content: V) -> Self {
        Self { content }
    }
}

impl<V: View> View for Menu<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Render menu as a floating glass box
        renderer.bifrost(rect, 15.0, 1.1, 0.95);
        renderer.fill_rounded_rect(rect, 8.0, [0.1, 0.1, 0.15, 0.9]);
        renderer.stroke_rounded_rect(rect, 8.0, [0.0, 0.8, 1.0, 0.4], 1.0);
        self.content.render(renderer, rect);
    }
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
pub struct ScrollView<V> {
    pub(crate) content: V,
    /// Unique identifier for this scroll view's state in the system state map.
    /// Must be non-zero for scroll state to persist across frames.
    pub(crate) scroll_id: u64,
    /// Content size hint (width, height). Used to compute scrollbar thumb size
    /// and max scroll offsets. If (0, 0), scrollbars are not shown.
    pub(crate) content_size: (f32, f32),
    /// Scroll speed multiplier for wheel events.
    pub(crate) scroll_speed: f32,
    /// Momentum decay factor per frame (0.0 = no momentum, 1.0 = infinite).
    /// Typical value: 0.90-0.95.
    pub(crate) momentum_decay: f32,
    /// Scrollbar width in pixels.
    pub(crate) scrollbar_width: f32,
    /// How long (in frames) the scrollbar stays visible after last scroll.
    pub(crate) scrollbar_fade_delay: u32,
    /// Scrollbar fade-out speed per frame (0.0 = instant, 1.0 = never fades).
    pub(crate) scrollbar_fade_speed: f32,
}

/// Internal scroll state stored in the system state map.
/// Bundles position, velocity, and scrollbar visibility into one struct.
#[derive(Clone, Copy, Debug, Default)]
struct ScrollState {
    pos_x: f32,
    pos_y: f32,
    vel_x: f32,
    vel_y: f32,
    scrollbar_opacity: f32,
    /// Frame counter for auto-hide delay.
    last_scroll_frame: u32,
    /// Whether a drag gesture is currently active.
    is_dragging: bool,
    /// Last pointer position during drag.
    last_pointer_x: f32,
    last_pointer_y: f32,
}

impl<V: View> ScrollView<V> {
    /// Create a new ScrollView wrapping `content`.
    pub fn new(content: V) -> Self {
        Self {
            content,
            scroll_id: 0,
            content_size: (0.0, 0.0),
            scroll_speed: 1.0,
            momentum_decay: 0.92,
            scrollbar_width: 6.0,
            scrollbar_fade_delay: 60,
            scrollbar_fade_speed: 0.85,
        }
    }

    /// Set a unique ID for this scroll view's state in the system state map.
    /// Views that share the same ID will share scroll position.
    pub fn scroll_id(mut self, id: u64) -> Self {
        self.scroll_id = id;
        self
    }

    /// Set the content size hint for scrollbar calculations.
    pub fn content_size(mut self, width: f32, height: f32) -> Self {
        self.content_size = (width, height);
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

    /// Apply momentum decay and clamp position to valid range.
    /// Returns updated state with velocity applied.
    fn apply_momentum(
        state: ScrollState,
        viewport_w: f32,
        viewport_h: f32,
        content_w: f32,
        content_h: f32,
        decay: f32,
    ) -> ScrollState {
        let max_x = (content_w - viewport_w).max(0.0);
        let max_y = (content_h - viewport_h).max(0.0);

        let mut new_state = state;

        // Apply velocity to position
        if !state.is_dragging {
            new_state.pos_x += state.vel_x;
            new_state.pos_y += state.vel_y;

            // Decay velocity
            new_state.vel_x *= decay;
            new_state.vel_y *= decay;

            // Stop very small velocities
            if new_state.vel_x.abs() < 0.01 {
                new_state.vel_x = 0.0;
            }
            if new_state.vel_y.abs() < 0.01 {
                new_state.vel_y = 0.0;
            }
        }

        // Clamp position
        new_state.pos_x = new_state.pos_x.clamp(0.0, max_x);
        new_state.pos_y = new_state.pos_y.clamp(0.0, max_y);

        // Bounce back if out of bounds (simple clamp for now)
        if new_state.pos_x <= 0.0 || new_state.pos_x >= max_x {
            new_state.vel_x = 0.0;
        }
        if new_state.pos_y <= 0.0 || new_state.pos_y >= max_y {
            new_state.vel_y = 0.0;
        }

        new_state
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
        let thumb_hover_color = [0.6, 0.6, 0.7, 0.75 * opacity];

        let thumb_ratio = viewport_size / content_size;
        let thumb_size = (viewport_size * thumb_ratio).max(24.0);
        let max_scroll = (content_size - viewport_size).max(0.0);
        let thumb_pos = if max_scroll > 0.0 {
            ((scroll_pos / max_scroll) * (viewport_size - thumb_size)).round()
        } else {
            0.0
        };

        if is_vertical {
            // Vertical scrollbar on the right edge
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
            // Use a slightly brighter color for the thumb
            let _ = thumb_hover_color; // available for hover state
            renderer.fill_rounded_rect(thumb_rect, sb_w / 2.0, thumb_color);
        } else {
            // Horizontal scrollbar on the bottom edge
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
        let content_w = self.content_size.0;
        let content_h = self.content_size.1;

        // ── Read current state ──
        let mut state = self.read_state();

        // ── Apply momentum (velocity decay) ──
        state = Self::apply_momentum(
            state,
            rect.width,
            rect.height,
            content_w,
            content_h,
            self.momentum_decay,
        );

        // ── Auto-hide scrollbar opacity ──
        let is_scrolling = state.vel_x.abs() > 0.1 || state.vel_y.abs() > 0.1 || state.is_dragging;
        if is_scrolling {
            state.scrollbar_opacity = 1.0;
            state.last_scroll_frame = 0;
        } else {
            state.last_scroll_frame += 1;
            if state.last_scroll_frame > self.scrollbar_fade_delay {
                state.scrollbar_opacity *= self.scrollbar_fade_speed;
                if state.scrollbar_opacity < 0.01 {
                    state.scrollbar_opacity = 0.0;
                }
            }
        }

        // ── Persist state ──
        self.write_state(state);

        // ── Register event handlers ──
        if self.scroll_id != 0 {
            let scroll_id = self.scroll_id;
            let speed = self.scroll_speed;
            let decay = self.momentum_decay;

            // Wheel handler: applies delta to position and velocity
            renderer.register_handler(
                "pointerwheel",
                std::sync::Arc::new(move |event| {
                    if let cvkg_core::Event::PointerWheel {
                        delta_x, delta_y, ..
                    } = event
                    {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            let mut st: ScrollState = s
                                .get_component_state::<ScrollState>(scroll_id)
                                .and_then(|g| g.read().ok().map(|v| *v))
                                .unwrap_or_default();
                            st.pos_x = (st.pos_x + delta_x * speed).max(0.0);
                            st.pos_y = (st.pos_y + delta_y * speed).max(0.0);
                            // Add velocity for momentum
                            st.vel_x += delta_x * speed * 0.5;
                            st.vel_y += delta_y * speed * 0.5;
                            st.scrollbar_opacity = 1.0;
                            st.last_scroll_frame = 0;
                            s.set_component_state(scroll_id, st);
                            s
                        });
                    }
                }),
            );

            // Pointer down: start drag tracking
            renderer.register_handler(
                "pointerdown",
                std::sync::Arc::new(move |event| {
                    if let cvkg_core::Event::PointerDown { x, y, .. } = event {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            let mut st: ScrollState = s
                                .get_component_state::<ScrollState>(scroll_id)
                                .and_then(|g| g.read().ok().map(|v| *v))
                                .unwrap_or_default();
                            st.is_dragging = true;
                            st.last_pointer_x = x;
                            st.last_pointer_y = y;
                            st.vel_x = 0.0;
                            st.vel_y = 0.0;
                            st.scrollbar_opacity = 1.0;
                            st.last_scroll_frame = 0;
                            s.set_component_state(scroll_id, st);
                            s
                        });
                    }
                }),
            );

            // Pointer move: apply drag delta
            renderer.register_handler(
                "pointermove",
                std::sync::Arc::new(move |event| {
                    if let cvkg_core::Event::PointerMove { x, y, .. } = event {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            let mut st: ScrollState = s
                                .get_component_state::<ScrollState>(scroll_id)
                                .and_then(|g| g.read().ok().map(|v| *v))
                                .unwrap_or_default();
                            if st.is_dragging {
                                let dx = st.last_pointer_x - x;
                                let dy = st.last_pointer_y - y;
                                st.pos_x = (st.pos_x + dx).max(0.0);
                                st.pos_y = (st.pos_y + dy).max(0.0);
                                // Track velocity for momentum on release
                                st.vel_x = dx * 0.5;
                                st.vel_y = dy * 0.5;
                                st.last_pointer_x = x;
                                st.last_pointer_y = y;
                                st.scrollbar_opacity = 1.0;
                                st.last_scroll_frame = 0;
                            }
                            s.set_component_state(scroll_id, st);
                            s
                        });
                    }
                }),
            );

            // Pointer up: end drag, keep velocity for momentum
            renderer.register_handler(
                "pointerup",
                std::sync::Arc::new(move |event| {
                    let _ = event;
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        let mut st: ScrollState = s
                            .get_component_state::<ScrollState>(scroll_id)
                            .and_then(|g| g.read().ok().map(|v| *v))
                            .unwrap_or_default();
                        st.is_dragging = false;
                        // Velocity is already set from last drag move
                        // Apply decay so it doesn't go forever
                        st.vel_x *= decay;
                        st.vel_y *= decay;
                        s.set_component_state(scroll_id, st);
                        s
                    });
                }),
            );

            // Keyboard handler: scroll-to-child / arrow key navigation
            renderer.register_handler(
                "keydown",
                std::sync::Arc::new(move |event| {
                    if let cvkg_core::Event::KeyDown { key, .. } = event {
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
                            let mut st: ScrollState = s
                                .get_component_state::<ScrollState>(scroll_id)
                                .and_then(|g| g.read().ok().map(|v| *v))
                                .unwrap_or_default();
                            if scroll_amount == f32::MAX {
                                st.pos_y = f32::MAX; // Will be clamped by momentum
                            } else if scroll_amount == f32::MIN {
                                st.pos_y = 0.0;
                                st.pos_x = 0.0;
                            } else {
                                // Determine direction from key
                                match key.as_str() {
                                    "ArrowLeft" | "ArrowRight" => {
                                        st.pos_x = (st.pos_x + scroll_amount).max(0.0);
                                    }
                                    _ => {
                                        st.pos_y = (st.pos_y + scroll_amount).max(0.0);
                                    }
                                }
                            }
                            st.vel_x = 0.0;
                            st.vel_y = 0.0;
                            st.scrollbar_opacity = 1.0;
                            st.last_scroll_frame = 0;
                            s.set_component_state(scroll_id, st);
                            s
                        });
                    }
                }),
            );
        }

        // ── Render content with clipping and transform ──
        renderer.push_clip_rect(rect);

        // Apply scroll offset via transform (rounded to avoid sub-pixel blur)
        renderer.push_transform([-state.pos_x.round(), -state.pos_y.round()], [1.0, 1.0], 0.0);

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

        // ── Render scrollbars ──
        let needs_v_scrollbar = content_h > rect.height;
        let needs_h_scrollbar = content_w > rect.width;

        if needs_v_scrollbar {
            self.render_scrollbar(
                renderer,
                rect,
                content_h,
                rect.height,
                state.pos_y,
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
                state.pos_x,
                state.scrollbar_opacity,
                false,
            );
        }
    }
}

/// Multi-column table layout
pub struct Table<V> {
    pub(crate) content: V,
}

impl<V: View> Table<V> {
    pub fn new(content: V) -> Self {
        Self { content }
    }
}

impl<V: View> View for Table<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Table layout logic would go here
        self.content.render(renderer, rect);
    }
}

/// Settings style grouped form layout
pub struct Form<V> {
    pub(crate) content: V,
}

impl<V: View> Form<V> {
    pub fn new(content: V) -> Self {
        Self { content }
    }
}

impl<V: View> View for Form<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Grouped form layout logic
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

/// Tooltip component for displaying short messages on hover.
pub struct Tooltip<V> {
    pub(crate) content: V,
    pub(crate) text: String,
    pub(crate) position: TooltipPosition,
}

impl<V: View> Tooltip<V> {
    pub fn new(content: V, text: impl Into<String>) -> Self {
        Self {
            content,
            text: text.into(),
            position: TooltipPosition::Top,
        }
    }

    pub fn position(mut self, position: TooltipPosition) -> Self {
        self.position = position;
        self
    }
}

pub enum TooltipPosition {
    Top,
    Right,
    Bottom,
    Left,
}

impl<V: View> View for Tooltip<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        self.content.render(renderer, rect);

        let (tw, th) = renderer.measure_text(&self.text, 12.0);
        let bubble_w = tw + 16.0;
        let bubble_h = th + 8.0;

        let bubble_rect = match self.position {
            TooltipPosition::Top => Rect {
                x: rect.x + (rect.width - bubble_w) / 2.0,
                y: rect.y - bubble_h - 5.0,
                width: bubble_w,
                height: bubble_h,
            },
            TooltipPosition::Bottom => Rect {
                x: rect.x + (rect.width - bubble_w) / 2.0,
                y: rect.y + rect.height + 5.0,
                width: bubble_w,
                height: bubble_h,
            },
            TooltipPosition::Left => Rect {
                x: rect.x - bubble_w - 5.0,
                y: rect.y + (rect.height - bubble_h) / 2.0,
                width: bubble_w,
                height: bubble_h,
            },
            TooltipPosition::Right => Rect {
                x: rect.x + rect.width + 5.0,
                y: rect.y + (rect.height - bubble_h) / 2.0,
                width: bubble_w,
                height: bubble_h,
            },
        };

        renderer.fill_rounded_rect(bubble_rect, 4.0, [0.05, 0.05, 0.1, 0.9]);
        renderer.stroke_rounded_rect(bubble_rect, 4.0, [0.0, 0.8, 1.0, 0.5], 1.0);
        renderer.draw_text(
            &self.text,
            bubble_rect.x + 8.0,
            bubble_rect.y + 4.0,
            12.0,
            theme::text(),
        );
    }
}

/// Popover component for displaying rich content in a floating bubble.
pub struct Popover<T, C> {
    pub(crate) trigger: T,
    pub(crate) content: C,
    pub(crate) is_open: bool,
    pub(crate) position: PopoverPosition,
}

impl<T: View, C: View> Popover<T, C> {
    pub fn new(trigger: T, content: C) -> Self {
        Self {
            trigger,
            content,
            is_open: false,
            position: PopoverPosition::Bottom,
        }
    }

    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }
}

pub enum PopoverPosition {
    Top,
    Right,
    Bottom,
    Left,
}

impl<T: View, C: View> View for Popover<T, C> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        self.trigger.render(renderer, rect);

        if self.is_open {
            let popover_w = 200.0;
            let popover_h = 150.0;

            let popover_rect = match self.position {
                PopoverPosition::Bottom => Rect {
                    x: rect.x + (rect.width - popover_w) / 2.0,
                    y: rect.y + rect.height + 8.0,
                    width: popover_w,
                    height: popover_h,
                },
                _ => Rect {
                    x: rect.x,
                    y: rect.y + rect.height + 8.0,
                    width: popover_w,
                    height: popover_h,
                },
            };

            renderer.bifrost(popover_rect, 15.0, 1.2, 0.9);
            renderer.fill_rounded_rect(popover_rect, 8.0, [0.05, 0.05, 0.1, 0.95]);
            renderer.stroke_rounded_rect(popover_rect, 8.0, [0.0, 1.0, 1.0, 0.4], 1.5);
            self.content.render(renderer, popover_rect);
        }
    }
}

/// Accordion component for collapsible content sections.
pub struct Accordion<V> {
    pub(crate) items: Vec<AccordionItem<V>>,
}

impl<V: View> Default for Accordion<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: View> Accordion<V> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    pub fn item(mut self, title: impl Into<String>, content: V) -> Self {
        self.items.push(AccordionItem {
            title: title.into(),
            content,
            is_expanded: false,
        });
        self
    }
}

pub struct AccordionItem<V> {
    pub(crate) title: String,
    pub(crate) content: V,
    pub(crate) is_expanded: bool,
}

impl<V: View> View for Accordion<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Accordion");

        // System-state hash for tracking expanded indices (Vec<bool>)
        let acc_hash: u64 = 0xE00_0001;
        let mut expanded: Vec<bool> = cvkg_core::load_system_state()
            .get_component_state::<Vec<bool>>(acc_hash)
            .and_then(|v| v.read().ok().map(|g| g.clone()))
            .unwrap_or_else(|| self.items.iter().map(|i| i.is_expanded).collect());
        while expanded.len() < self.items.len() {
            expanded.push(false);
        }

        let mut current_y = rect.y;
        for (idx, item) in self.items.iter().enumerate() {
            let header_h = 36.0;
            let header_rect = Rect {
                x: rect.x,
                y: current_y,
                width: rect.width,
                height: header_h,
            };

            let is_expanded = expanded.get(idx).copied().unwrap_or(false);

            // Header background
            if is_expanded {
                renderer.fill_rounded_rect(header_rect, 4.0, [0.08, 0.08, 0.14, 0.95]);
                renderer.stroke_rounded_rect(header_rect, 4.0, [0.0, 0.8, 1.0, 0.4], 1.5);
            } else {
                renderer.fill_rounded_rect(header_rect, 4.0, theme::surface());
                renderer.stroke_rounded_rect(header_rect, 4.0, [0.25, 0.25, 0.35, 0.5], 1.0);
            }

            let arrow = if is_expanded { "▼" } else { "▶" };
            let accent = if is_expanded { theme::accent() } else { theme::text_muted() };
            renderer.draw_text(arrow, header_rect.x + 8.0, header_rect.y + 10.0, 12.0, accent);
            renderer.draw_text(
                &item.title,
                header_rect.x + 28.0,
                header_rect.y + 10.0,
                FONT_BASE + 1.0,
                [1.0, 1.0, 1.0, 0.95],
            );

            // ── Click-to-toggle handler ──
            let hdr_x = header_rect.x;
            let hdr_y = header_rect.y;
            let hdr_w = header_rect.width;
            let hdr_h = header_rect.height;
            let item_idx = idx;
            let item_count = self.items.len();
            renderer.register_handler(
                "pointerclick",
                Arc::new(move |event| {
                    if let Event::PointerClick { x, y, .. } = event {
                        if x >= hdr_x && x <= hdr_x + hdr_w && y >= hdr_y && y <= hdr_y + hdr_h {
                            cvkg_core::update_system_state(move |s| {
                                let mut s = s.clone();
                                let mut state: Vec<bool> = s
                                    .get_component_state::<Vec<bool>>(acc_hash)
                                    .and_then(|v| v.read().ok().map(|g| g.clone()))
                                    .unwrap_or_else(|| vec![false; item_count]);
                                while state.len() <= item_idx {
                                    state.push(false);
                                }
                                state[item_idx] = !state[item_idx];
                                s.set_component_state(acc_hash, state);
                                s
                            });
                        }
                    }
                }),
            );

            current_y += header_h + 4.0;

            // Expanded content
            if is_expanded {
                let content_h = 100.0;
                let content_rect = Rect {
                    x: rect.x + 8.0,
                    y: current_y,
                    width: rect.width - 16.0,
                    height: content_h,
                };
                renderer.fill_rounded_rect(content_rect, 4.0, [0.04, 0.04, 0.07, 0.4]);
                item.content.render(renderer, content_rect);
                current_y += content_h + 8.0;
            }
        }

        renderer.pop_vnode();
    }
}

const COLLAPSIBLE_ANIM_HASH: u64 = 0xD00_0001;

/// Collapsible component for hiding/showing content with animation.
pub struct Collapsible<V> {
    pub(crate) content: V,
    pub(crate) header: String,
    pub(crate) is_open: bool,
}

impl<V: View> Collapsible<V> {
    pub fn new(header: impl Into<String>, content: V, is_open: bool) -> Self {
        Self { header: header.into(), content, is_open }
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
        let header_rect = Rect { x: rect.x, y: rect.y, width: rect.width, height: header_h };

        // ── Header bar ──
        renderer.fill_rounded_rect(header_rect, 6.0, [0.08, 0.08, 0.12, 0.9]);
        renderer.stroke_rounded_rect(header_rect, 6.0, [0.2, 0.2, 0.3, 0.5], 1.0);

        // Arrow indicator
        let arrow = if self.is_open { "▼" } else { "▶" };
        let accent = if self.is_open { theme::accent() } else { theme::text_muted() };
        renderer.draw_text(arrow, rect.x + 10.0, rect.y + 12.0, 12.0, accent);
        renderer.draw_text(&self.header, rect.x + 30.0, rect.y + 10.0, FONT_BASE + 2.0, [1.0, 1.0, 1.0, 0.95]);

        // ── Click-to-toggle handler ──
        let hdr = header_rect;
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event| {
                if let Event::PointerClick { x, y, .. } = event {
                    if hdr.contains(x, y) {
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
        renderer.stroke_rect(handle_rect, [0.0, 0.8, 1.0, if is_dragging { 0.8 } else { 0.4 }], 1.0);

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
                if let Event::PointerDown { x, y, .. } = event {
                    if h_rect.contains(x, y) {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            s.set_component_state(SPLITTER_DRAG_HASH, true);
                            s
                        });
                    }
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
                .map(|v| v.read().unwrap().clone())
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
                                .map(|v| v.read().unwrap().clone())
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
