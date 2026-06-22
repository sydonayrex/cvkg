use crate::theme;
use crate::{draw_focus_ring, FONT_BASE, RADIUS_LG, RADIUS_MD, RADIUS_XL};
use cvkg_core::{Event, Never, Rect, Renderer, View};
use std::sync::Arc;

/// Position from which a sheet slides in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SheetPosition {
    Left,
    Right,
    Top,
    Bottom,
}

impl SheetPosition {
    /// Returns true if the position is horizontal (sliding from Left or Right).
    pub fn is_horizontal(self) -> bool {
        matches!(self, SheetPosition::Left | SheetPosition::Right)
    }
}

/// Modal bottom sheet or side drawer container
///
/// # Contract
/// Manages slide-in dialogs positioned along viewport edges with spring physics.
pub struct GraniSheet<V> {
    pub(crate) content: V,
    pub(crate) position: SheetPosition,
    pub(crate) is_open: bool,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) on_dismiss: Option<Arc<dyn Fn() + Send + Sync>>,
}

const SHEET_ANIM_HASH: u64 = 0xB00_0100;

impl<V: View> GraniSheet<V> {
    /// Creates a new GraniSheet with the given content and anchor position.
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

    /// Sets the sheet's starting position.
    pub fn position(mut self, position: SheetPosition) -> Self {
        self.position = position;
        self
    }

    /// Configures the sheet's width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Configures the sheet's height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Configures a dismissal callback.
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

        renderer.fill_rect(rect, theme::with_alpha(theme::bg(), 0.65));

        let mut anim = 0.0f32;
        let solver_hash = SHEET_ANIM_HASH;
        {
            let sys = cvkg_core::load_system_state();
            if sys
                .get_component_state::<cvkg_core::SpringSolver>(solver_hash)
                .is_none()
            {
                cvkg_core::update_system_state(move |s| {
                    let mut ns = s.clone();
                    ns.set_component_state(
                        solver_hash,
                        cvkg_core::SpringSolver::new(
                            cvkg_core::SpringParams::fluid(),
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
                sys.get_component_state::<cvkg_core::SpringSolver>(solver_hash)
            {
                let mut solver = solver_arc.write().unwrap_or_else(|e| e.into_inner());
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

        if crate::theme::glassmorphism_enabled() {
            renderer.bifrost(sheet_rect, 16.0, 1.2, 0.9);
        }
        renderer.fill_rounded_rect(sheet_rect, RADIUS_XL, theme::surface_elevated());
        renderer.stroke_rounded_rect(sheet_rect, RADIUS_XL, theme::with_alpha(theme::accent(), 0.3), 1.5);

        let padding = 16.0;
        let sheet_content_rect = Rect {
            x: sheet_rect.x + padding,
            y: sheet_rect.y + padding + 44.0,
            width: (sheet_rect.width - padding * 2.0).max(0.0),
            height: (sheet_rect.height - padding * 2.0 - 44.0).max(0.0),
        };
        self.content.render(renderer, sheet_content_rect);

        let btn_size: f32 = 44.0;
        let close_rect = Rect {
            x: sheet_rect.x + sheet_rect.width - btn_size - 8.0,
            y: sheet_rect.y + 8.0,
            width: btn_size,
            height: btn_size,
        };
        renderer.fill_rounded_rect(close_rect, RADIUS_XL, theme::surface_elevated());
        renderer.draw_text(
            "×",
            close_rect.x + 15.0,
            close_rect.y + 13.0,
            FONT_BASE + 2.0,
            theme::text_muted(),
        );

        let close_cb = self.on_dismiss.clone();
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

            if crate::theme::glassmorphism_enabled() {
                renderer.bifrost(modal_rect, 25.0, 1.5, 0.85);
            }
            renderer.fill_rounded_rect(modal_rect, RADIUS_XL, theme::with_alpha(theme::bg(), 0.3));
            renderer.stroke_rounded_rect(modal_rect, RADIUS_XL, theme::border(), 2.0);

            self.content.render(renderer, modal_rect);
        }
    }
}

/// System-state hash key for the dialog open/close state.
const DIALOG_OPEN_HASH: u64 = 0xB00_0001;

/// A modal dialog with title, content, and actions.
///
/// # Contract
/// Fully blocks interaction with the background, offering title/action controls and keyboard accessibility (ESC/Enter/Tab focus trap).
pub struct GeriDialog<V> {
    pub(crate) is_presented: bool,
    pub(crate) title: Option<String>,
    pub(crate) content: V,
    pub(crate) actions: Vec<DialogAction>,
}

impl<V: View> GeriDialog<V> {
    /// Creates a new modal dialog wrapper.
    pub fn new(content: V) -> Self {
        Self {
            is_presented: false,
            title: None,
            content,
            actions: Vec::new(),
        }
    }

    /// Sets the presentation status.
    pub fn presented(mut self, is_presented: bool) -> Self {
        self.is_presented = is_presented;
        self
    }

    /// Sets the dialog's header title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Appends an action button with action callback.
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

        renderer.fill_rect(rect, theme::shadow());

        let modal_w = (rect.width * 0.8).min(450.0);
        let modal_h = (rect.height * 0.5).min(350.0);
        let modal_rect = Rect {
            x: rect.x + (rect.width - modal_w) / 2.0,
            y: rect.y + (rect.height - modal_h) / 2.0,
            width: modal_w,
            height: modal_h,
        };

        let anim_hash = DIALOG_OPEN_HASH + 100;
        let mut opacity: f32 = cvkg_core::load_system_state()
            .get_component_state::<f32>(anim_hash)
            .and_then(|g| g.read().ok().map(|v| *v))
            .unwrap_or(0.0);
        opacity = (opacity + 0.15).min(1.0);
        cvkg_core::update_system_state(move |s| {
            let mut s = s.clone();
            s.set_component_state(anim_hash, opacity);
            s
        });

        let backdrop_alpha = 0.7 * opacity;
        renderer.fill_rect(rect, theme::with_alpha(theme::bg(), backdrop_alpha));

        if crate::theme::glassmorphism_enabled() {
            renderer.bifrost(modal_rect, 20.0, 1.2, 0.9);
        }
        let modal_bg = theme::with_alpha(theme::surface_elevated(), 0.8 * opacity);
        renderer.fill_rounded_rect(modal_rect, RADIUS_LG, modal_bg);
        renderer.stroke_rounded_rect(
            modal_rect,
            RADIUS_LG,
            theme::with_alpha(theme::accent(), 0.5 * opacity),
            1.5,
        );

        if opacity > 0.5 {
            draw_focus_ring(renderer, modal_rect);
        }

        let padding = 20.0;
        let mut current_y = modal_rect.y + padding;

        if let Some(title) = &self.title {
            renderer.draw_text(
                title,
                modal_rect.x + padding,
                current_y,
                FONT_BASE + 6.0,
                theme::with_alpha(theme::text(), opacity),
            );
            current_y += 30.0;
        }

        let content_h = modal_h - (current_y - modal_rect.y) - 60.0;
        let content_rect = Rect {
            x: modal_rect.x + padding,
            y: current_y,
            width: modal_w - 2.0 * padding,
            height: content_h,
        };
        self.content.render(renderer, content_rect);

        let action_y = modal_rect.y + modal_h - 45.0;
        let action_w = 80.0;
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
            let bg = theme::with_alpha(theme::surface_elevated(), opacity);
            renderer.fill_rounded_rect(action_rect, RADIUS_MD, bg);
            renderer.stroke_rounded_rect(
                action_rect,
                RADIUS_MD,
                theme::with_alpha(theme::accent(), 0.8 * opacity),
                if is_focused { 2.0 } else { 1.0 },
            );
            if is_focused && opacity > 0.5 {
                draw_focus_ring(renderer, action_rect);
            }
            renderer.draw_text(
                &action.label,
                action_rect.x + 8.0,
                action_rect.y + 8.0,
                FONT_BASE,
                theme::with_alpha(theme::text(), opacity),
            );

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

        let mr = modal_rect;
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event| {
                if let Event::PointerClick { x, y, .. } = event {
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

        let action_count = self.actions.len();
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

/// Encapsulates a single action callback button for modal dialogs.
#[derive(Clone)]
pub struct DialogAction {
    pub label: String,
    pub style: DialogActionStyle,
    pub on_click: std::sync::Arc<dyn Fn() + Send + Sync>,
}

/// Visual styles of dialog actions.
#[derive(Clone)]
pub enum DialogActionStyle {
    Default,
    Destructive,
    Cancel,
}
