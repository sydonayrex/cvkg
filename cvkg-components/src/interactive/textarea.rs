use crate::theme;
use crate::{FONT_BASE, RADIUS_SM};
use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::Arc;

/// Multi-line text area with proper state management via system state.
#[derive(Clone)]
pub struct Textarea {
    pub(crate) placeholder: String,
    pub(crate) text: String,
    pub(crate) rows: usize,
    pub(crate) on_change: Arc<dyn Fn(String) + Send + Sync>,
}

impl Textarea {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            text: String::new(),
            rows: 3,
            on_change: Arc::new(|_| {}),
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.text = value.into();
        self
    }

    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows;
        self
    }

    pub fn on_change(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_change = Arc::new(callback);
        self
    }
}

impl View for Textarea {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Textarea");
        renderer.set_aria_role("textbox");
        renderer.set_aria_label(&self.placeholder);

        // Editor background
        renderer.fill_rounded_rect(rect, RADIUS_SM, theme::surface());
        renderer.stroke_rect(rect, theme::border_strong(), 1.0);

        // Draw text
        let line_height = 20.0;
        if self.text.is_empty() {
            renderer.draw_text(
                &self.placeholder,
                rect.x + 8.0,
                rect.y + 8.0,
                FONT_BASE,
                theme::border_strong(),
            );
        } else {
            let lines: Vec<&str> = self.text.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                let y = rect.y + 8.0 + (i as f32 * line_height);
                if y - rect.y < rect.height - 8.0 {
                    renderer.draw_text(line, rect.x + 8.0, y, FONT_BASE, theme::text());
                }
            }
        }

        // Draw Cursor on last line
        let text_lines: Vec<&str> = self.text.lines().collect();
        let last_line = text_lines.last().copied().unwrap_or("");
        let (tw, _) = renderer.measure_text(last_line, FONT_BASE);
        let cursor_x = rect.x + 8.0 + tw;
        let cursor_y = rect.y + 8.0 + (text_lines.len().max(1) - 1) as f32 * line_height;
        let time = renderer.elapsed_time();
        let alpha = if (time * 2.0).sin() > 0.0 { 1.0 } else { 0.3 };
        renderer.draw_line(
            cursor_x,
            cursor_y,
            cursor_x,
            cursor_y + 16.0,
            [0.0, 1.0, 1.0, alpha],
            2.0,
        );

        // Character count
        let count_text = format!("{} chars", self.text.len());
        let (cw, _) = renderer.measure_text(&count_text, 12.0);
        renderer.draw_text(
            &count_text,
            rect.x + rect.width - cw - 8.0,
            rect.y + rect.height - 16.0,
            12.0,
            [0.4, 0.4, 0.5, 0.7],
        );

        // Focus ring
        crate::draw_focus_ring(renderer, rect);

        // Interaction
        let on_change = self.on_change.clone();
        let text_mutex = Arc::new(std::sync::Mutex::new(self.text.clone()));

        let on_change_kd = on_change.clone();
        let text_mutex_kd = text_mutex.clone();
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let cvkg_core::Event::KeyDown { key } = event {
                    let mut changed = false;
                    let mut new_text = String::new();
                    if let Ok(mut text_guard) = text_mutex_kd.lock() {
                        if key.len() == 1 {
                            text_guard.push_str(&key);
                            changed = true;
                        } else if key == "Back" || key == "Backspace" {
                            text_guard.pop();
                            changed = true;
                        } else if key == "Return" || key == "Enter" {
                            text_guard.push('\n');
                            changed = true;
                        }
                        if changed {
                            new_text = text_guard.clone();
                        }
                    }
                    if changed {
                        (on_change_kd)(new_text);
                    }
                }
            }),
        );

        let on_change_ime = on_change.clone();
        let text_mutex_ime = text_mutex.clone();
        renderer.register_handler(
            "ime",
            Arc::new(move |event| {
                if let cvkg_core::Event::Ime(composition) = event {
                    let mut new_text = String::new();
                    if let Ok(mut text_guard) = text_mutex_ime.lock() {
                        text_guard.push_str(composition.as_str());
                        new_text = text_guard.clone();
                    }
                    (on_change_ime)(new_text);
                }
            }),
        );

        renderer.pop_vnode();
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(300.0),
            height: proposal.height.unwrap_or(self.rows as f32 * 20.0 + 16.0),
        }
    }
}
