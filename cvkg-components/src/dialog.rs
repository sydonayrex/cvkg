//! Dialog and overlay components.
//!
//! AlertDialog -- modal dialog for critical confirmations.
//! ConfirmationDialog -- destructive action confirmation.
//! FullScreenCover -- full screen overlay.
//!
//! All components use cvkg theme system (theme::*) for full themability.

use crate::integration::{CompanionBundle, WorldSpaceConfig};
use crate::lingua_tong;
use crate::theme;
use crate::{RADIUS_LG, RADIUS_MD, RADIUS_XL};
use cvkg_core::Event;
use cvkg_core::{Never, Rect, Renderer, Size, SizeProposal, View};
use std::sync::Arc;

// ----------------------------------------------------------------------------
// AlertDialog -- modal dialog for critical confirmations
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct AlertDialog {
    /// Dialog title.
    pub title: String,
    /// Dialog description.
    pub description: String,
    /// Confirm button label.
    pub confirm_label: String,
    /// Cancel button label.
    pub cancel_label: String,
    /// Whether the dialog is open.
    pub open: bool,
    /// Visual variant.
    pub variant: AlertVariant,
    /// Close callback (invoked when dialog is dismissed).
    pub on_close: Option<Arc<dyn Fn() + Send + Sync>>,
    /// VDOM companion bundle — modal focus trap + ARIA semantics.
    pub companions: CompanionBundle,
    /// Optional 3D world-space placement (renders the dialog as a 3D panel
    /// composited into the scene with glass material + spring settling).
    pub world: WorldSpaceConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertVariant {
    Default,
    Destructive,
    Warning,
}

impl AlertDialog {
    /// Create a new AlertDialog.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            description: String::new(),
            confirm_label: lingua_tong::t("dialog.confirm"),
            cancel_label: lingua_tong::t("dialog.cancel"),
            open: false,
            variant: AlertVariant::Default,
            on_close: None,
            companions: CompanionBundle::focusable()
                .with_role("dialog")
                .with_label(title.to_string()),
            world: WorldSpaceConfig::default(),
        }
    }

    /// Opt this dialog into 3D world-space rendering.
    pub fn world(mut self, config: WorldSpaceConfig) -> Self {
        self.world = config;
        self
    }

    /// Set the description.
    pub fn description(mut self, d: &str) -> Self {
        self.description = d.to_string();
        self
    }

    /// Set the confirm label.
    pub fn confirm_label(mut self, l: &str) -> Self {
        self.confirm_label = l.to_string();
        self
    }

    /// Set the cancel label.
    pub fn cancel_label(mut self, l: &str) -> Self {
        self.cancel_label = l.to_string();
        self
    }

    /// Set open state.
    pub fn open(mut self, o: bool) -> Self {
        self.open = o;
        self
    }

    /// Set the variant.
    pub fn variant(mut self, v: AlertVariant) -> Self {
        self.variant = v;
        self
    }
    /// Set the close callback.
    pub fn on_close_cb(mut self, cb: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_close = Some(Arc::new(cb));
        self
    }
}

impl View for AlertDialog {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn companion_states(&self) -> Vec<Box<dyn cvkg_core::Companion>> {
        let mut bundle = self.companions.clone();
        if !self.title.is_empty() {
            bundle = bundle.with_label(self.title.clone());
        }
        bundle.to_vec()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.open {
            return;
        }
        let panel_id = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            "alert_dialog".hash(&mut s);
            self.title.hash(&mut s);
            s.finish()
        };
        renderer.push_vnode_with_companions(rect, "AlertDialog", self.companions.to_vec());
        renderer.register_a11y("alertdialog", &self.title);

        // 3D world-space: redirect draw calls to offscreen texture when enabled.
        self.world.begin(renderer, panel_id);
        renderer.fill_rect(rect, theme::with_alpha(theme::bg(), 0.5));
        let dlg_w = 400.0;
        let dlg_h = 180.0;
        let dlg_rect = Rect {
            x: rect.x + (rect.width - dlg_w) / 2.0,
            y: rect.y + (rect.height - dlg_h) / 2.0,
            width: dlg_w,
            height: dlg_h,
        };
        renderer.fill_rounded_rect(dlg_rect, RADIUS_XL, theme::surface_elevated());
        renderer.stroke_rounded_rect(dlg_rect, RADIUS_XL, theme::border(), 1.0);
        let icon_color = match self.variant {
            AlertVariant::Destructive => theme::error_color(),
            AlertVariant::Warning => theme::warning(),
            AlertVariant::Default => theme::info(),
        };
        renderer.fill_ellipse(
            Rect {
                x: dlg_rect.x + 24.0,
                y: dlg_rect.y + 28.0,
                width: 24.0,
                height: 24.0,
            },
            icon_color,
        );
        renderer.draw_text_raw(
            &self.title,
            dlg_rect.x + 60.0,
            dlg_rect.y + 36.0,
            16.0,
            theme::text(),
        );
        if !self.description.is_empty() {
            renderer.draw_text_raw(
                &self.description,
                dlg_rect.x + 24.0,
                dlg_rect.y + 64.0,
                13.0,
                theme::text_muted(),
            );
        }
        let btn_y = dlg_rect.y + dlg_h - 60.0;
        let cancel_rect = Rect {
            x: dlg_rect.x + dlg_w - 200.0,
            y: btn_y,
            width: 88.0,
            height: 44.0,
        };
        renderer.push_vnode(cancel_rect, "AlertDialog:Cancel");
        renderer.set_aria_role("button");
        renderer.set_aria_label(&self.cancel_label);
        renderer.fill_rounded_rect(cancel_rect, RADIUS_LG, theme::surface());
        renderer.stroke_rounded_rect(cancel_rect, RADIUS_LG, theme::border(), 1.0);
        let (ctw, cth) = renderer.measure_text(&self.cancel_label, 13.0);
        renderer.draw_text_raw(
            &self.cancel_label,
            cancel_rect.x + (88.0 - ctw) / 2.0,
            cancel_rect.y + (44.0 - cth) / 2.0,
            13.0,
            theme::text(),
        );
        renderer.pop_vnode();
        let confirm_rect = Rect {
            x: dlg_rect.x + dlg_w - 104.0,
            y: btn_y,
            width: 88.0,
            height: 44.0,
        };
        let confirm_bg = match self.variant {
            AlertVariant::Destructive => theme::error_color(),
            _ => theme::accent(),
        };
        renderer.push_vnode(confirm_rect, "AlertDialog:Confirm");
        renderer.set_aria_role("button");
        renderer.set_aria_label(&self.confirm_label);
        renderer.fill_rounded_rect(confirm_rect, RADIUS_LG, confirm_bg);
        let (ftw, fth) = renderer.measure_text(&self.confirm_label, 13.0);
        renderer.draw_text_raw(
            &self.confirm_label,
            confirm_rect.x + (88.0 - ftw) / 2.0,
            confirm_rect.y + (44.0 - fth) / 2.0,
            13.0,
            theme::bg(),
        );
        renderer.pop_vnode();
        // ── Keyboard handler: Escape to close, Enter/Space to activate ──
        let close_cb = self.on_close.clone();
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    match key.as_str() {
                        "Escape" => {
                            if let Some(ref cb) = close_cb {
                                (cb)();
                            }
                        }
                        "Enter" | " " => {
                            if let Some(ref cb) = close_cb {
                                (cb)();
                            }
                        }
                        _ => {}
                    }
                }
            }),
        );
        renderer.pop_vnode();

        // End 3D world-space redirection if it was begun above.
        if self.world.is_enabled() {
            renderer.end_world_space_panel(panel_id);
        }
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: proposal.width.unwrap_or(400.0),
            height: 180.0,
        }
    }
}

impl cvkg_core::LayoutView for AlertDialog {
    fn size_that_fits(
        &self,
        proposal: cvkg_core::SizeProposal,
        _subviews: &[&dyn cvkg_core::LayoutView],
        _cache: &mut cvkg_core::LayoutCache,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(400.0),
            height: 180.0,
        }
    }

    fn place_subviews(
        &self,
        _bounds: cvkg_core::Rect,
        _subviews: &mut [&mut dyn cvkg_core::LayoutView],
        _cache: &mut cvkg_core::LayoutCache,
    ) {
    }
}

// ----------------------------------------------------------------------------
// ConfirmationDialog -- confirmation dialog with destructive action
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct ConfirmationDialog {
    /// Dialog title.
    pub title: String,
    /// Message.
    pub message: String,
    /// Confirm button label.
    pub confirm_label: String,
    /// Whether the dialog is open.
    pub open: bool,
    /// Confirm callback (invoked when Enter/Space is pressed).
    pub on_confirm: Arc<dyn Fn() + Send + Sync>,
    /// Cancel callback (invoked when Escape is pressed).
    pub on_cancel: Arc<dyn Fn() + Send + Sync>,
}

impl ConfirmationDialog {
    /// Create a new ConfirmationDialog.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            message: String::new(),
            confirm_label: lingua_tong::t("dialog.delete"),
            open: false,
            on_confirm: Arc::new(|| {}),
            on_cancel: Arc::new(|| {}),
        }
    }

    /// Set the message.
    pub fn message(mut self, m: &str) -> Self {
        self.message = m.to_string();
        self
    }

    /// Set the confirm label.
    pub fn confirm_label(mut self, l: &str) -> Self {
        self.confirm_label = l.to_string();
        self
    }

    /// Set open state.
    pub fn open(mut self, o: bool) -> Self {
        self.open = o;
        self
    }
    /// Set the confirm callback.
    pub fn on_confirm_cb(mut self, cb: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_confirm = Arc::new(cb);
        self
    }
    /// Set the cancel callback.
    pub fn on_cancel_cb(mut self, cb: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_cancel = Arc::new(cb);
        self
    }
}

impl View for ConfirmationDialog {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.open {
            return;
        }
        renderer.push_vnode(rect, "ConfirmationDialog");
        renderer.register_a11y("alertdialog", &self.title);
        renderer.fill_rect(rect, theme::with_alpha(theme::bg(), 0.5));
        let dlg_w = 360.0;
        let dlg_h = 160.0;
        let dlg_rect = Rect {
            x: rect.x + (rect.width - dlg_w) / 2.0,
            y: rect.y + (rect.height - dlg_h) / 2.0,
            width: dlg_w,
            height: dlg_h,
        };
        renderer.fill_rounded_rect(dlg_rect, RADIUS_XL, theme::surface_elevated());
        renderer.stroke_rounded_rect(dlg_rect, RADIUS_XL, theme::border(), 1.0);
        renderer.draw_text_raw(
            &self.title,
            dlg_rect.x + 20.0,
            dlg_rect.y + 28.0,
            16.0,
            theme::text(),
        );
        if !self.message.is_empty() {
            renderer.draw_text_raw(
                &self.message,
                dlg_rect.x + 20.0,
                dlg_rect.y + 52.0,
                13.0,
                theme::text_muted(),
            );
        }
        let btn_y = dlg_rect.y + dlg_h - 56.0;
        let cancel_rect = Rect {
            x: dlg_rect.x + dlg_w - 180.0,
            y: btn_y,
            width: 72.0,
            height: 44.0,
        };
        renderer.push_vnode(cancel_rect, "ConfirmationDialog:Cancel");
        renderer.register_a11y("button", &lingua_tong::t("dialog.cancel"));
        renderer.fill_rounded_rect(cancel_rect, RADIUS_MD, theme::surface());
        renderer.stroke_rounded_rect(cancel_rect, RADIUS_MD, theme::border(), 1.0);
        let cancel_text = lingua_tong::t("dialog.cancel");
        let (ctw, cth) = renderer.measure_text(&cancel_text, 12.0);
        renderer.draw_text_raw(
            &cancel_text,
            cancel_rect.x + (72.0 - ctw) / 2.0,
            cancel_rect.y + (44.0 - cth) / 2.0,
            12.0,
            theme::text(),
        );
        renderer.pop_vnode();
        let confirm_rect = Rect {
            x: dlg_rect.x + dlg_w - 100.0,
            y: btn_y,
            width: 80.0,
            height: 44.0,
        };
        renderer.push_vnode(confirm_rect, "ConfirmationDialog:Confirm");
        renderer.register_a11y("button", &self.confirm_label);
        renderer.fill_rounded_rect(confirm_rect, RADIUS_MD, theme::error_color());
        let (ftw, fth) = renderer.measure_text(&self.confirm_label, 12.0);
        renderer.draw_text_raw(
            &self.confirm_label,
            confirm_rect.x + (80.0 - ftw) / 2.0,
            confirm_rect.y + (44.0 - fth) / 2.0,
            12.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        renderer.pop_vnode();
        // ── Keyboard handler: Escape to cancel, Enter/Space to confirm ──
        let cancel_cb = self.on_cancel.clone();
        let confirm_cb = self.on_confirm.clone();
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    match key.as_str() {
                        "Escape" => {
                            (cancel_cb)();
                        }
                        "Enter" | " " => {
                            (confirm_cb)();
                        }
                        _ => {}
                    }
                }
            }),
        );
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: proposal.width.unwrap_or(360.0),
            height: 160.0,
        }
    }
}

impl cvkg_core::LayoutView for ConfirmationDialog {
    fn size_that_fits(
        &self,
        proposal: cvkg_core::SizeProposal,
        _subviews: &[&dyn cvkg_core::LayoutView],
        _cache: &mut cvkg_core::LayoutCache,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(360.0),
            height: 160.0,
        }
    }

    fn place_subviews(
        &self,
        _bounds: cvkg_core::Rect,
        _subviews: &mut [&mut dyn cvkg_core::LayoutView],
        _cache: &mut cvkg_core::LayoutCache,
    ) {
    }
}

// ----------------------------------------------------------------------------
// FullScreenCover -- full screen overlay
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct FullScreenCover {
    /// Content title.
    pub title: String,
    /// Whether the cover is presented.
    pub presented: bool,
    /// Animation progress.
    pub progress: f32,
    /// Background color.
    pub bg_color: [f32; 4],
}

impl FullScreenCover {
    /// Create a new FullScreenCover.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            presented: false,
            progress: 0.0,
            bg_color: theme::bg(),
        }
    }

    /// Set presented.
    pub fn presented(mut self, p: bool) -> Self {
        self.presented = p;
        self
    }

    /// Set progress.
    pub fn progress(mut self, p: f32) -> Self {
        self.progress = p.clamp(0.0, 1.0);
        self
    }

    /// Set background color.
    pub fn bg_color(mut self, c: [f32; 4]) -> Self {
        self.bg_color = c;
        self
    }
}

impl View for FullScreenCover {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.presented && self.progress <= 0.0 {
            return;
        }
        renderer.push_vnode(rect, "FullScreenCover");
        let offset_y = (1.0 - self.progress) * rect.height;
        let cover_rect = Rect {
            x: rect.x,
            y: rect.y + offset_y,
            width: rect.width,
            height: rect.height,
        };
        renderer.fill_rect(cover_rect, self.bg_color);
        let (tw, _th) = renderer.measure_text(&self.title, 24.0);
        renderer.draw_text_raw(
            &self.title,
            cover_rect.x + (cover_rect.width - tw) / 2.0,
            cover_rect.y + 60.0,
            24.0,
            theme::text(),
        );
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: proposal.width.unwrap_or(800.0),
            height: 600.0,
        }
    }
}
