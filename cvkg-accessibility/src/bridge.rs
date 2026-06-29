//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     -- State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     -- Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     -- Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    -- Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     -- Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     -- Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   -- Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//!   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//!   CVKG Extended: Section 2 of the CVKG Design Specification

//! Screen reader bridge — platform-agnostic announcement API.
//!
//! # Why this exists
//! Screen reader APIs are platform-specific: NVDA/JAWS on Windows use COM
//! automation, VoiceOver on macOS uses NSAccessibility, Orca on Linux uses
//! AT-SPI/DBus. This trait abstracts the announcement mechanism so that CVKG
//! core code stays platform-agnostic. Platform crates provide concrete
//! implementations; test and headless environments use `NullScreenReaderBridge`.

use serde::{Deserialize, Serialize};

/// Priority for screen reader announcements.
///
/// Matches the ARIA `aria-live` polite/assertive distinction.
/// Polite is preferred for most status updates; assertive is reserved for
/// time-critical alerts (e.g., form validation errors, system warnings).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnnouncementPriority {
    /// Wait for current speech to finish before announcing. Use by default.
    Polite,
    /// Interrupt current speech to announce immediately. Use sparingly.
    Assertive,
}

/// A screen reader announcement message with priority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Announcement {
    /// The text to announce. Should be concise and meaningful out of context.
    pub message: String,
    /// How urgently this message should interrupt ongoing speech.
    pub priority: AnnouncementPriority,
}

impl Announcement {
    /// Create a polite announcement (the common case).
    pub fn polite(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            priority: AnnouncementPriority::Polite,
        }
    }

    /// Create an assertive announcement (interrupts current speech).
    pub fn assertive(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            priority: AnnouncementPriority::Assertive,
        }
    }
}

/// Platform bridge for screen reader announcements.
///
/// # WHY
/// Screen reader APIs are platform-specific (NVDA/JAWS on Windows,
/// VoiceOver on macOS, Orca on Linux). This trait abstracts the
/// announcement mechanism so CVKG core code stays platform-agnostic.
///
/// # Contract
/// - Implementations MUST be `Send + Sync` so they can be stored in
///   `Arc<dyn ScreenReaderBridge>` and shared across threads.
/// - `announce` MUST NOT block the calling thread. Platform bridging
///   should be dispatched asynchronously if the underlying API is blocking.
/// - `is_active` is advisory; the bridge may return `false` even when a
///   screen reader is present if detection is unavailable on the platform.
pub trait ScreenReaderBridge: Send + Sync {
    /// Announce a message to the screen reader.
    ///
    /// `Polite` priority waits for current speech to finish.
    /// `Assertive` priority interrupts immediately.
    fn announce(&self, announcement: Announcement);

    /// Returns `true` if a screen reader is currently active and listening.
    ///
    /// Callers can use this to skip expensive accessible-text construction
    /// when no AT is running, but MUST NOT rely on this to suppress
    /// structural accessibility tree updates.
    fn is_active(&self) -> bool;
}

/// No-op implementation of `ScreenReaderBridge`.
///
/// Used in headless / test environments where no platform screen reader is
/// present. Announcements are logged at DEBUG level so they remain visible
/// in test output without triggering platform APIs.
pub struct NullScreenReaderBridge;

impl ScreenReaderBridge for NullScreenReaderBridge {
    /// Log the announcement but do not invoke any platform speech API.
    fn announce(&self, announcement: Announcement) {
        let priority = match announcement.priority {
            AnnouncementPriority::Polite => "polite",
            AnnouncementPriority::Assertive => "assertive",
        };
        tracing::debug!(
            "[A11y] {} ({}): {}",
            "announce",
            priority,
            announcement.message
        );
    }

    /// Always returns `false` — no screen reader is active in this null bridge.
    fn is_active(&self) -> bool {
        false
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_bridge_is_not_active() {
        let bridge = NullScreenReaderBridge;
        assert!(!bridge.is_active());
    }

    #[test]
    fn null_bridge_announce_does_not_panic() {
        let bridge = NullScreenReaderBridge;
        // Must not panic; logging may or may not be captured depending on env.
        bridge.announce(Announcement::polite("Focus moved to Save button"));
        bridge.announce(Announcement::assertive("Error: required field is empty"));
    }

    #[test]
    fn announcement_polite_constructor() {
        let a = Announcement::polite("hello");
        assert_eq!(a.message, "hello");
        assert_eq!(a.priority, AnnouncementPriority::Polite);
    }

    #[test]
    fn announcement_assertive_constructor() {
        let a = Announcement::assertive("warning");
        assert_eq!(a.message, "warning");
        assert_eq!(a.priority, AnnouncementPriority::Assertive);
    }

    #[test]
    fn announcement_serde_roundtrip() {
        let a = Announcement::assertive("test message");
        let json = serde_json::to_string(&a).expect("serialize");
        let decoded: Announcement = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.message, a.message);
        assert_eq!(decoded.priority, a.priority);
    }

    #[test]
    fn null_bridge_is_send_sync() {
        // Compile-time check: NullScreenReaderBridge must satisfy the trait bounds.
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<NullScreenReaderBridge>();
    }
}
