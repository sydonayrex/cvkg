//! A11yBeacon -- Live Region Accessibility Integration.
//!
//! Provides screen reader announcements for dynamic content changes.
//! Wraps any view and announces messages when content changes.

use cvkg_core::{AnnouncementPriority, Never, Rect, Renderer, View};

/// A wrapper that announces content changes to screen readers.
///
/// Use A11yBeacon for dynamic content that screen reader users need to be
/// notified about: toast notifications, form validation errors, streaming
/// text updates, search result counts.
#[derive(Clone)]
pub struct A11yBeacon<V: View> {
    content: V,
    message: String,
    priority: AnnouncementPriority,
}

impl<V: View> A11yBeacon<V> {
    /// Create a new A11yBeacon that announces `message` with the given `priority`.
    pub fn new(content: V, message: impl Into<String>, priority: AnnouncementPriority) -> Self {
        Self {
            content,
            message: message.into(),
            priority,
        }
    }

    /// Set a new announcement message.
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    /// Get the current announcement message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl<V: View> View for A11yBeacon<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Announce the message (renderer queues it for the screen reader).
        // Only announce non-empty messages.
        if !self.message.is_empty() {
            renderer.announce(&self.message, self.priority);
        }
        self.content.render(renderer, rect);
    }
}

/// Extension trait for adding accessibility announcements to any view.
pub trait A11yBeaconExt: View + Sized {
    /// Wrap this view with a polite accessibility announcement.
    fn announce(self, message: impl Into<String>) -> A11yBeacon<Self> {
        A11yBeacon::new(self, message, AnnouncementPriority::Polite)
    }

    /// Wrap this view with an assertive accessibility announcement.
    fn announce_urgent(self, message: impl Into<String>) -> A11yBeacon<Self> {
        A11yBeacon::new(self, message, AnnouncementPriority::Assertive)
    }
}

impl<T: View + Sized> A11yBeaconExt for T {}
