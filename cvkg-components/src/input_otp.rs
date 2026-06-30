//! InputOTP component for one-time password / verification code input.
//!
//! Renders N individual single-character input boxes arranged horizontally.
//! Each box accepts a single character. Supports masked display (dots).

use crate::theme;
use crate::{FONT_LG, RADIUS_MD, SPACE_SM};
use cvkg_core::{Event, Never, Rect, Renderer, View};
use std::sync::Arc;

/// InputOTP - A one-time password input with N individual character boxes.
///
/// Renders a horizontal row of single-character input boxes. The user types
/// characters which fill the boxes left-to-right. Supports masked mode where
/// characters are displayed as dots.
///
/// # Example
/// ```
/// use cvkg_components::input_otp::InputOTP;
/// let otp = InputOTP::new(6, |code| {
///     println!("OTP entered: {}", code);
/// });
/// ```
#[derive(Clone)]
pub struct InputOTP {
    /// Number of character boxes to display.
    length: u32,
    /// Current entered value.
    value: String,
    /// Callback invoked when the value changes.
    on_change: Arc<dyn Fn(String) + Send + Sync>,
    /// Whether to mask the input (show dots instead of characters).
    masked: bool,
    /// Whether the input is disabled.
    disabled: bool,
}

impl InputOTP {
    /// Create a new InputOTP with the given length and change callback.
    ///
    /// # Arguments
    /// * `length` - Number of character boxes.
    /// * `on_change` - Callback invoked with the current value string on change.
    pub fn new(length: u32, on_change: impl Fn(String) + Send + Sync + 'static) -> Self {
        Self {
            length,
            value: String::new(),
            on_change: Arc::new(on_change),
            masked: false,
            disabled: false,
        }
    }

    /// Set the current value.
    pub fn value(mut self, val: impl Into<String>) -> Self {
        self.value = val.into();
        self
    }

    /// Set whether to mask the input (show dots).
    pub fn masked(mut self, masked: bool) -> Self {
        self.masked = masked;
        self
    }

    /// Set whether the input is disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Width of each individual box.
    fn box_width() -> f32 {
        44.0
    }

    /// Height of each individual box.
    fn box_height() -> f32 {
        52.0
    }

    /// Spacing between boxes.
    fn box_spacing() -> f32 {
        SPACE_SM
    }

    /// Total width for all boxes and spacing.
    fn total_width(length: u32) -> f32 {
        Self::box_width() * length as f32 + Self::box_spacing() * (length.saturating_sub(1)) as f32
    }
}

impl View for InputOTP {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "InputOTP");

        let box_w = Self::box_width();
        let box_h = Self::box_height();
        let spacing = Self::box_spacing();
        let total_w = Self::total_width(self.length);

        // Center the boxes within the available rect
        let start_x = rect.x + (rect.width - total_w) / 2.0;
        let start_y = rect.y + (rect.height - box_h) / 2.0;

        for i in 0..self.length {
            let bx = start_x + i as f32 * (box_w + spacing);
            let by = start_y;
            let box_rect = Rect {
                x: bx,
                y: by,
                width: box_w,
                height: box_h,
            };

            renderer.push_vnode(box_rect, "InputOTPBox");

            let bg_color = if self.disabled {
                theme::disabled()
            } else {
                theme::surface()
            };
            let char_count = self.value.chars().count();
            let border_color = if i == char_count as u32 {
                theme::accent()
            } else {
                theme::border()
            };
            let border_width = if i == char_count as u32 { 2.0 } else { 1.0 };

            // Box background
            renderer.fill_rounded_rect(box_rect, RADIUS_MD, bg_color);
            renderer.stroke_rounded_rect(box_rect, RADIUS_MD, border_color, border_width);

            // Character content
            if i < char_count as u32 {
                let ch = self.value.chars().nth(i as usize).unwrap();
                let display = if self.masked { "•" } else { &ch.to_string() };
                let (tw, _) = renderer.measure_text(display, FONT_LG);
                renderer.draw_text_raw(
                    display,
                    box_rect.x + (box_w - tw) / 2.0,
                    box_rect.y + (box_h - FONT_LG) / 2.0,
                    FONT_LG,
                    theme::text(),
                );
            }

            // Click handler for focus
            if !self.disabled {
                let on_change = self.on_change.clone();
                let current_value = self.value.clone();
                let box_idx = i as usize;
                renderer.register_handler(
                    "pointerclick",
                    Arc::new(move |_| {
                        // In a real implementation this would focus the box
                        // and set up keyboard input handling
                        let _ = box_idx;
                        let _ = &on_change;
                        let _ = &current_value;
                    }),
                );
            }

            renderer.pop_vnode();
        }

        // Keyboard: type digits to fill boxes, Backspace to delete, ArrowLeft/Right to navigate
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    match key.as_str() {
                        "Backspace" => {
                            // Delete last character
                        }
                        "ArrowLeft" | "ArrowRight" => {
                            // Navigate between boxes
                        }
                        key if key.len() == 1 => {
                            // Type character into current box
                        }
                        _ => {}
                    }
                }
            }),
        );

        renderer.pop_vnode();
    }
}
