//! ConsentGate — GDPR consent dialog and data provenance UI.
//!
//! Provides a consent request dialog for AI data usage and a data provenance
//! indicator showing what data was used in AI inference.

use crate::lingua_tong;
use crate::theme;
use crate::{RADIUS_XL, RADIUS_MD};
use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::Arc;

/// A consent request dialog for data usage.
///
/// # When to show:
/// Before the AI system accesses personal data (documents, messages, files),
/// show this dialog. The user must explicitly opt in.
///
/// # Left limit:
/// - Never pre-check consent boxes. Default is ALWAYS unchecked.
/// - Never use dark patterns (tiny "reject" button, confusing language).
///   Both "Accept" and "Reject" must be equally prominent.
/// - Never proceed without explicit user action.
#[derive(Clone)]
pub struct ConsentGate {
    data_description: String,
    purpose: String,
    consented: Arc<std::sync::Mutex<bool>>,
    on_decision: Option<Arc<dyn Fn(bool) + Send + Sync>>,
}

impl ConsentGate {
    pub fn new(data_description: impl Into<String>, purpose: impl Into<String>) -> Self {
        Self {
            data_description: data_description.into(),
            purpose: purpose.into(),
            consented: Arc::new(std::sync::Mutex::new(false)),
            on_decision: None,
        }
    }

    pub fn on_decision<F: Fn(bool) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_decision = Some(Arc::new(f));
        self
    }

    /// Record that the user accepted.
    pub fn accept(&self) {
        if let Ok(mut c) = self.consented.lock() {
            *c = true;
        }
        if let Some(ref cb) = self.on_decision {
            cb(true);
        }
    }

    /// Record that the user rejected.
    pub fn reject(&self) {
        if let Ok(mut c) = self.consented.lock() {
            *c = false;
        }
        if let Some(ref cb) = self.on_decision {
            cb(false);
        }
    }

    /// Check if the user has consented.
    pub fn is_consented(&self) -> bool {
        self.consented.lock().map(|c| *c).unwrap_or(false)
    }
}

impl View for ConsentGate {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Semi-transparent backdrop
        renderer.fill_rect(rect, theme::with_alpha(theme::bg(), 0.5));

        // Dialog card (centered)
        let dialog_w = (rect.width - 40.0).min(400.0);
        let dialog_h = (rect.height - 40.0).min(250.0);
        let dialog_x = rect.x + (rect.width - dialog_w) / 2.0;
        let dialog_y = rect.y + (rect.height - dialog_h) / 2.0;
        let dialog = Rect::new(dialog_x, dialog_y, dialog_w, dialog_h);

        renderer.fill_rounded_rect(dialog, RADIUS_XL, theme::surface_elevated());
        renderer.stroke_rounded_rect(dialog, RADIUS_XL, theme::border(), 1.0);

        // Title
        let title = lingua_tong::t("consentgate.title");
        renderer.draw_text(
            &title,
            dialog.x + 16.0,
            dialog.y + 24.0,
            16.0,
            theme::text(),
        );

        // Data description
        let data_label = lingua_tong::t_with(
            "consentgate.data_label",
            &[("data", &self.data_description)],
        );
        renderer.draw_text(
            &data_label,
            dialog.x + 16.0,
            dialog.y + 56.0,
            13.0,
            theme::text(),
        );

        // Purpose
        let purpose_label =
            lingua_tong::t_with("consentgate.purpose_label", &[("purpose", &self.purpose)]);
        renderer.draw_text(
            &purpose_label,
            dialog.x + 16.0,
            dialog.y + 80.0,
            13.0,
            theme::text_muted(),
        );

        // Buttons: Reject and Accept, EQUAL size and prominence.
        let button_y = dialog.y + dialog_h - 50.0;
        let button_w = (dialog_w - 48.0) / 2.0;

        // Reject button (left, secondary style)
        let reject_rect = Rect::new(dialog.x + 16.0, button_y, button_w, 36.0);
        renderer.fill_rounded_rect(reject_rect, RADIUS_MD, theme::surface());
        renderer.stroke_rounded_rect(reject_rect, RADIUS_MD, theme::border(), 1.0);
        let reject_text = lingua_tong::t("consentgate.reject");
        let (rw, _) = renderer.measure_text(&reject_text, 14.0);
        renderer.draw_text(
            &reject_text,
            reject_rect.x + (button_w - rw) / 2.0,
            reject_rect.y + 22.0,
            14.0,
            theme::text(),
        );

        // Accept button (right, accent style)
        let accept_rect = Rect::new(dialog.x + 32.0 + button_w, button_y, button_w, 36.0);
        renderer.fill_rounded_rect(accept_rect, RADIUS_MD, theme::accent());
        let accept_text = lingua_tong::t("consentgate.accept");
        let (aw, _) = renderer.measure_text(&accept_text, 14.0);
        renderer.draw_text(
            &accept_text,
            accept_rect.x + (button_w - aw) / 2.0,
            accept_rect.y + 22.0,
            14.0,
            [1.0, 1.0, 1.0, 1.0],
        );
    }
}

/// A data provenance indicator showing what data was used in AI inference.
///
/// # Why this matters:
/// When the AI gives a response, users should know what data it used.
/// This builds trust and lets users correct the AI if it used wrong data.
#[derive(Clone)]
pub struct DataTrail {
    pub sources: Vec<TrailSource>,
}

#[derive(Debug, Clone)]
pub struct TrailSource {
    pub name: String,
    pub item_count: u32,
    pub kind: TrailKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrailKind {
    Document,
    Message,
    File,
    Database,
    Web,
}

impl View for DataTrail {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let mut y = rect.y;
        let data_used_text = lingua_tong::t("consentgate.data_used");
        renderer.draw_text(&data_used_text, rect.x, y, 11.0, theme::text_muted());
        y += 14.0;

        for source in &self.sources {
            let icon = match source.kind {
                TrailKind::Document => "[doc]",
                TrailKind::Message => "[msg]",
                TrailKind::File => "[file]",
                TrailKind::Database => "[db]",
                TrailKind::Web => "[web]",
            };
            let items_label = lingua_tong::t_with(
                "consentgate.items_count",
                &[("count", &source.item_count.to_string())],
            );
            let text = format!("{} {} {}", icon, source.name, items_label);
            renderer.draw_text(&text, rect.x + 8.0, y, 11.0, theme::text());
            y += 14.0;
        }
    }
}
