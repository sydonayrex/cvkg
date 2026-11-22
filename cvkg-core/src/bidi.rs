//! Base text direction and horizontal geometry mirroring for RTL layouts.

use crate::Rect;

/// Base direction for a block of text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BidiDirection {
    /// Left-to-right (default).
    Ltr,
    /// Right-to-left.
    Rtl,
    /// Infer direction from the first strong character.
    Auto,
}

impl Default for BidiDirection {
    fn default() -> Self {
        Self::Ltr
    }
}

impl BidiDirection {
    /// Resolve `Auto` against the text, leaving explicit directions unchanged.
    pub fn resolve(self, text: &str) -> Self {
        if self != Self::Auto {
            return self;
        }
        for ch in text.chars() {
            if is_strong_rtl(ch) {
                return Self::Rtl;
            }
            if ch.is_alphabetic() {
                return Self::Ltr;
            }
        }
        Self::Ltr
    }

    /// Whether the resolved direction is right-to-left.
    pub fn is_rtl(self, text: &str) -> bool {
        self.resolve(text) == Self::Rtl
    }
}

fn is_strong_rtl(ch: char) -> bool {
    ('\u{0590}'..='\u{05FF}').contains(&ch)
        || ('\u{0600}'..='\u{06FF}').contains(&ch)
        || ('\u{0700}'..='\u{074F}').contains(&ch)
        || ('\u{0800}'..='\u{087F}').contains(&ch)
}

/// Mirror a horizontal rectangle within a container of `container_width`.
pub fn mirror_rect(rect: Rect, container_width: f32) -> Rect {
    Rect::new(
        container_width - rect.x - rect.width,
        rect.y,
        rect.width,
        rect.height,
    )
}
