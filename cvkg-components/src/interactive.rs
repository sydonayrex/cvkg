use cvkg_core::{Never, Rect, Renderer, View};

/// Button with action callback
#[allow(dead_code)]
pub struct Button {
    pub(crate) label: String,
    pub(crate) on_click: std::sync::Arc<dyn Fn() + Send + Sync>,
}

impl Button {
    /// Create a new Button with a label and an action callback.
    pub fn new(label: impl Into<String>, on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            label: label.into(),
            on_click: std::sync::Arc::new(on_click),
        }
    }
}

impl View for Button {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Button");
        renderer.set_key(&self.label);
        renderer.set_aria_role("button");
        renderer.set_aria_label(&self.label);

        // Get pressed state from system state
        let state_registry = cvkg_core::get_system_state();
        let id_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            self.label.hash(&mut s);
            s.finish()
        };

        let is_pressed = {
            let s = state_registry.read().unwrap();
            s.get_component_state::<bool>(id_hash)
                .map(|v| *v.read().unwrap())
                .unwrap_or(false)
        };

        // Background: dark panel, slightly brighter if pressed
        let bg = if is_pressed {
            [0.2, 0.2, 0.25, 1.0]
        } else {
            [0.1, 0.1, 0.15, 1.0]
        };
        renderer.fill_rounded_rect(rect, 6.0, bg);
        
        // Neon cyan border, thicker if pressed
        let border_width = if is_pressed { 3.0 } else { 2.0 };
        renderer.stroke_rect(rect, [0.0, 0.9, 1.0, 1.0], border_width);
        
        // Label text
        let text_x = rect.x + 8.0;
        let text_y = rect.y + (rect.height - 14.0) / 2.0;
        renderer.draw_text(&self.label, text_x, text_y, 14.0, [1.0, 1.0, 1.0, 1.0]);

        // Register interaction handlers
        let on_click = self.on_click.clone();
        let state_registry_down = state_registry.clone();
        let state_registry_up = state_registry.clone();

        renderer.register_handler(
            "pointerdown",
            std::sync::Arc::new(move |_| {
                let mut s = state_registry_down.write().unwrap();
                s.set_component_state(id_hash, true);
            }),
        );

        renderer.register_handler(
            "pointerup",
            std::sync::Arc::new(move |_| {
                let mut s = state_registry_up.write().unwrap();
                s.set_component_state(id_hash, false);
            }),
        );

        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |_| {
                (on_click)();
            }),
        );
        renderer.pop_vnode();
    }
}

pub struct Toggle {
    pub(crate) label: String,
    pub(crate) is_on: bool,
    pub(crate) on_change: std::sync::Arc<dyn Fn(bool) + Send + Sync>,
}

impl Toggle {
    /// Create a new Toggle switch.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::Toggle;
    /// let toggle = Toggle::new("Enable feature", false, |val| println!("Toggled: {}", val));
    /// ```
    pub fn new(
        label: impl Into<String>,
        is_on: bool,
        on_change: impl Fn(bool) + Send + Sync + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            is_on,
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for Toggle {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Toggle");
        let track_w = 40.0;
        let track_h = 20.0;
        let track_x = rect.x;
        let track_y = rect.y + (rect.height - track_h) / 2.0;
        let track = Rect {
            x: track_x,
            y: track_y,
            width: track_w,
            height: track_h,
        };

        let bg = if self.is_on {
            [0.0, 0.8, 0.4, 1.0]
        } else {
            [0.2, 0.2, 0.25, 1.0]
        };

        renderer.set_aria_role("switch");
        renderer.set_aria_label(&self.label);
        renderer.fill_rounded_rect(track, track_h / 2.0, bg);
        // Thumb
        let thumb_x = if self.is_on {
            track_x + track_w - track_h + 2.0
        } else {
            track_x + 2.0
        };
        renderer.fill_rounded_rect(
            Rect {
                x: thumb_x,
                y: track_y + 2.0,
                width: track_h - 4.0,
                height: track_h - 4.0,
            },
            (track_h - 4.0) / 2.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        // Label
        renderer.draw_text(
            &self.label,
            rect.x + track_w + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        // Interaction
        let is_on = self.is_on;
        let on_change = self.on_change.clone();
        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |_| {
                (on_change)(!is_on);
            }),
        );
        renderer.pop_vnode();
    }
}

pub struct Slider {
    pub(crate) value: f32,
    pub(crate) range: std::ops::RangeInclusive<f32>,
    pub(crate) on_change: std::sync::Arc<dyn Fn(f32) + Send + Sync>,
}

impl Slider {
    /// Create a new Slider.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::Slider;
    /// let slider = Slider::new(0.5, 0.0..=1.0, |val| println!("Value: {}", val));
    /// ```
    pub fn new(
        value: f32,
        range: std::ops::RangeInclusive<f32>,
        on_change: impl Fn(f32) + Send + Sync + 'static,
    ) -> Self {
        Self {
            value,
            range,
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for Slider {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let track_h = 4.0;
        let track_y = rect.y + (rect.height - track_h) / 2.0;
        // Track background
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x,
                y: track_y,
                width: rect.width,
                height: track_h,
            },
            track_h / 2.0,
            [0.2, 0.2, 0.25, 1.0],
        );
        // Track fill
        let start = *self.range.start();
        let end = *self.range.end();
        let pct = if (end - start).abs() > f32::EPSILON {
            ((self.value - start) / (end - start)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x,
                y: track_y,
                width: rect.width * pct,
                height: track_h,
            },
            track_h / 2.0,
            [0.0, 0.85, 1.0, 1.0],
        );
        // Thumb
        let thumb_r = 8.0;
        let thumb_x = rect.x + rect.width * pct - thumb_r;
        renderer.fill_rounded_rect(
            Rect {
                x: thumb_x,
                y: track_y - thumb_r + track_h / 2.0,
                width: thumb_r * 2.0,
                height: thumb_r * 2.0,
            },
            thumb_r,
            [1.0, 1.0, 1.0, 1.0],
        );

        // Interaction
        let start = *self.range.start();
        let end = *self.range.end();
        let on_change = self.on_change.clone();
        let slider_rect = rect;
        let is_dragging = std::sync::Arc::new(std::sync::Mutex::new(false));

        let drag_start = is_dragging.clone();
        renderer.register_handler(
            "pointerdown",
            std::sync::Arc::new(move |_| {
                if let Ok(mut dragging) = drag_start.lock() {
                    *dragging = true;
                }
            }),
        );

        let drag_stop = is_dragging.clone();
        renderer.register_handler(
            "pointerup",
            std::sync::Arc::new(move |_| {
                if let Ok(mut dragging) = drag_stop.lock() {
                    *dragging = false;
                }
            }),
        );

        renderer.register_handler(
            "pointermove",
            std::sync::Arc::new(move |event| {
                if let Ok(dragging) = is_dragging.lock() {
                    if !*dragging {
                        return;
                    }
                }
                if let cvkg_core::Event::PointerMove { x, .. } = event {
                    let pct = ((x - slider_rect.x) / slider_rect.width).clamp(0.0, 1.0);
                    let val = start + pct * (end - start);
                    (on_change)(val);
                }
            }),
        );
    }
}

/// Stepper for discrete increment/decrement
pub struct Stepper {
    pub(crate) label: String,
    pub(crate) value: i32,
    pub(crate) on_change: std::sync::Arc<dyn Fn(i32) + Send + Sync>,
}

impl Stepper {
    pub fn new(
        label: impl Into<String>,
        value: i32,
        on_change: impl Fn(i32) + Send + Sync + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            value,
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for Stepper {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.set_aria_role("spinbutton");
        renderer.set_aria_label(&self.label);

        // Stepper container
        renderer.fill_rounded_rect(rect, 4.0, [0.12, 0.12, 0.15, 1.0]);
        renderer.stroke_rect(rect, [0.3, 0.3, 0.4, 1.0], 1.0);

        let label_w = rect.width * 0.4;
        renderer.draw_text(
            &self.label,
            rect.x + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        // Buttons
        let btn_w = 30.0;
        let minus_rect = Rect {
            x: rect.x + label_w,
            y: rect.y + 4.0,
            width: btn_w,
            height: rect.height - 8.0,
        };
        let plus_rect = Rect {
            x: rect.x + rect.width - btn_w - 4.0,
            y: rect.y + 4.0,
            width: btn_w,
            height: rect.height - 8.0,
        };

        renderer.fill_rounded_rect(minus_rect, 2.0, [0.2, 0.2, 0.25, 1.0]);
        renderer.draw_text(
            "-",
            minus_rect.x + 10.0,
            minus_rect.y + (minus_rect.height - 14.0) / 2.0,
            14.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        renderer.fill_rounded_rect(plus_rect, 2.0, [0.2, 0.2, 0.25, 1.0]);
        renderer.draw_text(
            "+",
            plus_rect.x + 10.0,
            plus_rect.y + (plus_rect.height - 14.0) / 2.0,
            14.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        let val_text = self.value.to_string();
        let val_x = minus_rect.x + btn_w + 10.0;
        renderer.draw_text(
            &val_text,
            val_x,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            [0.0, 0.85, 1.0, 1.0],
        );

        // Interaction
        let on_change = self.on_change.clone();
        let value = self.value;

        // We use a shared element to detect which button was clicked
        // For now, we simulate by checking coordinates in a single handler or using multiple sub-nodes.
        // In CVKG VDOM, we can just register handlers on the parent and check coords.
        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |event| {
                if let cvkg_core::Event::PointerClick { x, .. } = event {
                    if x >= minus_rect.x && x <= minus_rect.x + minus_rect.width {
                        (on_change)(value - 1);
                    } else if x >= plus_rect.x && x <= plus_rect.x + plus_rect.width {
                        (on_change)(value + 1);
                    }
                }
            }),
        );
    }
}

pub struct TextField {
    pub(crate) placeholder: String,
    pub(crate) text: String,
    pub(crate) on_change: std::sync::Arc<dyn Fn(String) + Send + Sync>,
}

impl TextField {
    /// Create a new TextField.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::TextField;
    /// let field = TextField::new("Enter name", "", |t| println!("Name: {}", t));
    /// ```
    pub fn new(
        placeholder: impl Into<String>,
        text: impl Into<String>,
        on_change: impl Fn(String) + Send + Sync + 'static,
    ) -> Self {
        Self {
            placeholder: placeholder.into(),
            text: text.into(),
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for TextField {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "TextField");
        renderer.set_aria_role("textbox");
        renderer.set_aria_label(&self.placeholder);

        // Input background
        renderer.fill_rounded_rect(rect, 6.0, [0.08, 0.08, 0.12, 1.0]);
        renderer.stroke_rect(rect, [0.3, 0.3, 0.4, 1.0], 1.0);

        let is_focused = true; // Simplified focus for now
        let display_text = if self.text.is_empty() {
            &self.placeholder
        } else {
            &self.text
        };
        let text_color = if self.text.is_empty() {
            [0.5, 0.5, 0.55, 1.0]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        };

        renderer.draw_text(
            display_text,
            rect.x + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            text_color,
        );

        // Draw Cursor (simulated at end for now, but with proper rendering)
        if is_focused && !self.text.is_empty() {
            let (tw, _) = renderer.measure_text(&self.text, 14.0);
            let cursor_x = rect.x + 8.0 + tw;
            let cursor_y = rect.y + (rect.height - 16.0) / 2.0;
            // Flashing cursor based on some global timer or just solid for now
            renderer.draw_line(
                cursor_x,
                cursor_y,
                cursor_x,
                cursor_y + 16.0,
                [0.0, 1.0, 1.0, 1.0],
                2.0,
            );
        }

        // Interaction
        let on_change = self.on_change.clone();
        let text_mutex = std::sync::Arc::new(std::sync::Mutex::new(self.text.clone()));

        renderer.register_handler(
            "keydown",
            std::sync::Arc::new(move |event| {
                if let cvkg_core::Event::KeyDown { key } = event {
                    let mut changed = false;
                    let mut new_text = String::new();

                    if let Ok(mut text_guard) = text_mutex.lock() {
                        if key.len() == 1 {
                            text_guard.push_str(&key);
                            changed = true;
                        } else if key == "Back" || key == "Backspace" {
                            text_guard.pop();
                            changed = true;
                        } else if key == "Return" || key == "Enter" {
                            // Handle submission or blur?
                        }
                        if changed {
                            new_text = text_guard.clone();
                        }
                    }

                    if changed {
                        (on_change)(new_text);
                    }
                }
            }),
        );
        renderer.pop_vnode();
    }
}

/// Secure password input
pub struct SecureField {
    pub(crate) placeholder: String,
    pub(crate) text: String,
    pub(crate) on_change: std::sync::Arc<dyn Fn(String) + Send + Sync>,
}

impl SecureField {
    pub fn new(
        placeholder: impl Into<String>,
        text: impl Into<String>,
        on_change: impl Fn(String) + Send + Sync + 'static,
    ) -> Self {
        Self {
            placeholder: placeholder.into(),
            text: text.into(),
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for SecureField {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.set_aria_role("password");
        renderer.set_aria_label(&self.placeholder);

        // Input background
        renderer.fill_rounded_rect(rect, 6.0, [0.08, 0.08, 0.12, 1.0]);
        renderer.stroke_rect(rect, [0.4, 0.2, 0.4, 1.0], 1.0); // Slightly purple for security

        let display = if self.text.is_empty() {
            self.placeholder.clone()
        } else {
            "*".repeat(self.text.len())
        };

        let text_color = if self.text.is_empty() {
            [0.5, 0.5, 0.55, 1.0]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        };
        renderer.draw_text(
            &display,
            rect.x + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            text_color,
        );

        // Interaction (Secure key entry)
        let current_text = std::sync::Arc::new(std::sync::Mutex::new(self.text.clone()));
        let on_change = self.on_change.clone();

        renderer.register_handler(
            "keydown",
            std::sync::Arc::new(move |event| {
                if let cvkg_core::Event::KeyDown { key } = event {
                    let mut changed = false;
                    let mut new_text = String::new();
                    if let Ok(mut text_guard) = current_text.lock() {
                        if key.len() == 1 {
                            text_guard.push_str(&key);
                            new_text = text_guard.clone();
                            changed = true;
                        } else if key == "Backspace" {
                            text_guard.pop();
                            new_text = text_guard.clone();
                            changed = true;
                        }
                    }
                    if changed {
                        (on_change)(new_text);
                    }
                }
            }),
        );
    }
}

/// Multi-line text area
pub struct TextEditor {
    pub(crate) text: String,
    pub(crate) on_change: std::sync::Arc<dyn Fn(String) + Send + Sync>,
}

impl TextEditor {
    pub fn new(
        text: impl Into<String>,
        on_change: impl Fn(String) + Send + Sync + 'static,
    ) -> Self {
        Self {
            text: text.into(),
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for TextEditor {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "TextEditor");
        renderer.set_aria_role("textbox");

        // Editor background
        renderer.fill_rounded_rect(rect, 4.0, [0.05, 0.05, 0.08, 1.0]);
        renderer.stroke_rect(rect, [0.2, 0.2, 0.3, 1.0], 1.0);

        // Draw text
        let lines: Vec<&str> = self.text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            renderer.draw_text(
                line,
                rect.x + 8.0,
                rect.y + 8.0 + (i as f32 * 20.0),
                14.0,
                [1.0, 1.0, 1.0, 1.0],
            );
        }

        // Draw Cursor on last line
        let last_line = lines.last().copied().unwrap_or("");
        let (tw, _) = renderer.measure_text(last_line, 14.0);
        let cursor_x = rect.x + 8.0 + tw;
        let cursor_y = rect.y + 8.0 + (lines.len().max(1) - 1) as f32 * 20.0;
        renderer.draw_line(
            cursor_x,
            cursor_y,
            cursor_x,
            cursor_y + 16.0,
            [0.0, 1.0, 1.0, 1.0],
            2.0,
        );

        // Interaction
        let on_change = self.on_change.clone();
        let text_mutex = std::sync::Arc::new(std::sync::Mutex::new(self.text.clone()));

        renderer.register_handler(
            "keydown",
            std::sync::Arc::new(move |event| {
                if let cvkg_core::Event::KeyDown { key } = event {
                    let mut changed = false;
                    let mut new_text = String::new();

                    if let Ok(mut text_guard) = text_mutex.lock() {
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
                        (on_change)(new_text);
                    }
                }
            }),
        );
        renderer.pop_vnode();
    }
}

/// Picker for selection from a list of options
pub struct Picker {
    pub(crate) selection: usize,
    pub(crate) options: Vec<String>,
    pub(crate) on_change: std::sync::Arc<dyn Fn(usize) + Send + Sync>,
}

impl Picker {
    pub fn new(
        selection: usize,
        options: Vec<String>,
        on_change: impl Fn(usize) + Send + Sync + 'static,
    ) -> Self {
        Self {
            selection,
            options,
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for Picker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.set_aria_role("combobox");

        // Picker background
        renderer.fill_rounded_rect(rect, 6.0, [0.15, 0.15, 0.2, 1.0]);
        renderer.stroke_rect(rect, [0.3, 0.3, 0.4, 1.0], 1.0);

        let selected_text = self
            .options
            .get(self.selection)
            .cloned()
            .unwrap_or_default();
        renderer.draw_text(
            &selected_text,
            rect.x + 10.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        // Chevron
        renderer.draw_text(
            "▼",
            rect.x + rect.width - 20.0,
            rect.y + (rect.height - 14.0) / 2.0,
            12.0,
            [0.5, 0.5, 0.6, 1.0],
        );

        // Interaction (Cycle options on click)
        let on_change = self.on_change.clone();
        let selection = self.selection;
        let count = self.options.len();

        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |_| {
                if count > 0 {
                    (on_change)((selection + 1) % count);
                }
            }),
        );
    }
}

/// DatePicker for calendar date selection
pub struct DatePicker {
    pub(crate) timestamp: u64,
    pub(crate) on_change: std::sync::Arc<dyn Fn(u64) + Send + Sync>,
}

impl DatePicker {
    pub fn new(timestamp: u64, on_change: impl Fn(u64) + Send + Sync + 'static) -> Self {
        Self {
            timestamp,
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for DatePicker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.set_aria_role("grid");

        // DatePicker background
        renderer.fill_rounded_rect(rect, 6.0, [0.12, 0.12, 0.15, 1.0]);
        renderer.stroke_rect(rect, [0.3, 0.3, 0.4, 1.0], 1.0);

        // Human-readable date simulation (DD-MM-YYYY)
        // Since we don't have a full date lib, we do a simple epoch-to-date estimation for the demo
        let days_since_epoch = self.timestamp / 86400;
        let years = 1970 + (days_since_epoch / 365);
        let days = (days_since_epoch % 365) + 1;

        let date_str = format!("Date: {:04}-{:03}", years, days);
        renderer.draw_text(
            &date_str,
            rect.x + 10.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            [0.0, 0.85, 1.0, 1.0],
        );

        // Interaction (Increment timestamp by 1 day on click)
        let on_change = self.on_change.clone();
        let timestamp = self.timestamp;

        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |_| {
                (on_change)(timestamp + 86400);
            }),
        );

        // Calendar icon placeholder
        renderer.draw_text(
            "📅",
            rect.x + rect.width - 24.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            [0.5, 0.5, 0.6, 1.0],
        );
    }
}

/// ColorPicker for RGBA color selection
pub struct ColorPicker {
    pub(crate) color: crate::Color,
    pub(crate) on_change: std::sync::Arc<dyn Fn(crate::Color) + Send + Sync>,
}

impl ColorPicker {
    pub fn new(
        color: crate::Color,
        on_change: impl Fn(crate::Color) + Send + Sync + 'static,
    ) -> Self {
        Self {
            color,
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for ColorPicker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.set_aria_role("colorwell");

        // ColorPicker container
        renderer.fill_rounded_rect(rect, 6.0, [0.15, 0.15, 0.18, 1.0]);
        renderer.stroke_rect(rect, [0.3, 0.3, 0.4, 1.0], 1.0);

        // Current color preview
        let preview_w = 40.0;
        let preview_rect = Rect {
            x: rect.x + 8.0,
            y: rect.y + 5.0,
            width: preview_w,
            height: rect.height - 10.0,
        };
        renderer.fill_rounded_rect(preview_rect, 2.0, self.color.as_array());
        renderer.stroke_rect(preview_rect, [1.0, 1.0, 1.0, 0.3], 1.0);

        // Color grid (4 colors for demo)
        let colors = [
            crate::Color::BLACK,
            crate::Color::WHITE,
            crate::Color::RED,
            crate::Color::CYAN,
        ];

        let grid_relative_x = 8.0 + preview_w + 12.0;
        let available_w = (rect.width - grid_relative_x - 10.0).max(0.0);
        let cell_w = available_w / 4.0;
        let cell_h = rect.height - 10.0;

        for (i, &col) in colors.iter().enumerate() {
            let cell_rect = Rect {
                x: rect.x + grid_relative_x + (i as f32 * (cell_w + 5.0)),
                y: rect.y + 5.0,
                width: cell_w,
                height: cell_h,
            };

            renderer.fill_rounded_rect(cell_rect, 2.0, col.as_array());

            // Interaction
            let on_change = self.on_change.clone();
            renderer.register_handler(
                "pointerclick",
                std::sync::Arc::new(move |event| {
                    if let cvkg_core::Event::PointerClick { x, .. } = event {
                        if x >= cell_rect.x && x <= cell_rect.x + cell_rect.width {
                            (on_change)(col);
                        }
                    }
                }),
            );
        }
    }
}
