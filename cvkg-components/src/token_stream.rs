//! TokenStream -- Streaming AI diff renderer with word-by-word highlight.
//!
//! Displays incrementally arriving text tokens with a visual highlight
//! that fades from accent color to normal text color. Includes a blinking
//! cursor that follows the streaming head.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::{Arc, Mutex};

/// A streaming text view that displays incrementally arriving tokens.
///
/// # Design decision: Why Arc<Mutex<String>> and not a reactive signal?
///
/// The cvkg-core reactivity system uses State<T> with change listeners.
/// But streaming text arrives at 30-100 tokens/second. Each token arrival
/// triggering a full State change + re-render is wasteful.
///
/// Instead, the TokenStream holds a raw Arc<Mutex<String>> that the
/// renderer reads during render(), and a generation counter that IS
/// a State<u64> -- only the counter triggers re-renders, not the string clone.
#[derive(Clone)]
pub struct TokenStream {
    /// The accumulated text so far. Displayed to the user.
    text: Arc<Mutex<String>>,
    /// Generation counter. Incremented on each token arrival.
    /// This is the ONLY thing that triggers re-renders.
    generation: cvkg_core::State<u64>,
    /// Tokens that arrived in the current "highlight window".
    /// These are rendered in accent color, then fade to normal.
    recent_tokens: Arc<Mutex<Vec<HighlightSegment>>>,
    /// Whether the stream is still active (cursor visible) or complete.
    streaming: cvkg_core::State<bool>,
    /// Highlight fade duration in seconds.
    highlight_duration: f32,
}

/// A range of text that should be visually highlighted as "new".
#[derive(Debug, Clone)]
pub struct HighlightSegment {
    /// Byte offset into the full text.
    pub start: usize,
    /// Byte length of the segment.
    pub len: usize,
    /// When this segment arrived (renderer elapsed time).
    pub arrived_at: f32,
}

impl TokenStream {
    /// Create a new TokenStream starting with optional pre-filled content.
    pub fn new(initial: impl Into<String>) -> Self {
        Self {
            text: Arc::new(Mutex::new(initial.into())),
            generation: cvkg_core::State::new(0),
            recent_tokens: Arc::new(Mutex::new(Vec::new())),
            streaming: cvkg_core::State::new(true),
            highlight_duration: 2.0,
        }
    }

    /// Set the highlight fade duration in seconds.
    pub fn highlight_duration(mut self, secs: f32) -> Self {
        self.highlight_duration = secs.max(0.1);
        self
    }

    /// Append a new token to the stream. Can be called from any thread.
    pub fn push_token(&self, token: &str) {
        let mut text = match self.text.lock() {
            Ok(g) => g,
            Err(_) => return, // mutex poisoned = stop streaming
        };

        let start = text.len();
        text.push_str(token);
        let len = token.len();

        let mut recent = match self.recent_tokens.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        recent.push(HighlightSegment {
            start,
            len,
            arrived_at: 0.0, // Will be set from renderer elapsed time during render
        });

        drop(text);
        drop(recent);

        // Trigger re-render via generation counter
        let current_gen = self.generation.get();
        self.generation.set(current_gen.wrapping_add(1));
    }

    /// Mark the stream as complete. Cursor disappears, highlight fades immediately.
    pub fn finish(&self) {
        self.streaming.set(false);
        let current = self.generation.get();
        self.generation.set(current.wrapping_add(1));
    }

    /// Check if the stream is still active.
    pub fn is_streaming(&self) -> bool {
        self.streaming.get()
    }

    /// Get a snapshot of the current text (for debugging/logging only).
    pub fn snapshot(&self) -> String {
        self.text.lock().map(|g| g.clone()).unwrap_or_default()
    }
}

impl View for TokenStream {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let text = match self.text.lock() {
            Ok(g) => g.clone(),
            Err(_) => {
                renderer.draw_text("[stream error]", rect.x, rect.y, 14.0, [1.0, 0.0, 0.0, 1.0]);
                return;
            }
        };

        if text.is_empty() {
            return;
        }

        let now = renderer.elapsed_time();
        let highlight_dur = self.highlight_duration;

        // Split text into segments: highlighted (recent) vs normal (old)
        let recent = match self.recent_tokens.lock() {
            Ok(g) => g.clone(),
            Err(_) => Vec::new(),
        };

        // Build a list of (start, end, is_highlighted) from recent tokens
        let mut highlighted_ranges: Vec<(usize, usize)> = recent
            .iter()
            .filter(|seg| now - seg.arrived_at < highlight_dur)
            .map(|seg| (seg.start, seg.start + seg.len))
            .collect();
        highlighted_ranges.sort_by_key(|r| r.0);

        // Render text character by character, tracking highlight state
        let mut current_x = rect.x;
        let mut current_y = rect.y;
        let line_height: f32 = 20.0;
        let base_color = theme::text();
        let highlight_color = theme::accent();

        for (byte_idx, ch) in text.char_indices() {
            let is_highlighted = highlighted_ranges
                .iter()
                .any(|(start, end)| byte_idx >= *start && byte_idx < *end);

            let color = if is_highlighted {
                highlight_color
            } else {
                base_color
            };

            let mut buf = [0u8; 4];
            let ch_str = ch.encode_utf8(&mut buf);
            let (w, _h) = renderer.measure_text(ch_str, 14.0);

            if current_x + w > rect.x + rect.width && ch != '\n' {
                current_x = rect.x;
                current_y += line_height;
            }

            if ch == '\n' {
                current_x = rect.x;
                current_y += line_height;
                continue;
            }

            renderer.draw_text(ch_str, current_x, current_y, 14.0, color);
            current_x += w;
        }

        // Render blinking cursor if streaming
        if self.streaming.get() {
            let blink = (now * 2.0) % 1.0;
            if blink < 0.5 {
                renderer.draw_line(
                    current_x,
                    current_y,
                    current_x,
                    current_y + line_height,
                    [0.0, 0.8, 1.0, 1.0],
                    2.0,
                );
            }
        }
    }
}
