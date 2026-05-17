use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::Arc;

/// Button with action callback
#[allow(dead_code)]
#[derive(Clone)]
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

        // Get pressed state from system state (lock-free snapshot)
        let id_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            self.label.hash(&mut s);
            s.finish()
        };

        let is_pressed = {
            let s = cvkg_core::load_system_state();
            s.get_component_state::<bool>(id_hash)
                .map(|v| *v.read().unwrap())
                .unwrap_or(false)
        };

        // Background: dark panel, slightly brighter if pressed
        let bg = if is_pressed {
            [0.2, 0.2, 0.25, 1.0]
        } else {
            [0.12, 0.12, 0.18, 1.0]
        };

        // Elevation & Depth
        renderer.push_shadow(1.0, [0.0, 0.0, 0.0, 0.5], [0.0, 1.0]);
        renderer.fill_rounded_rect(rect, 6.0, bg);
        renderer.pop_shadow();

        // Neon cyan border, thicker if pressed
        let border_width = if is_pressed { 3.0 } else { 1.5 };
        renderer.stroke_rect(rect, [0.0, 0.9, 1.0, 0.8], border_width);

        // Label text
        let text_x = rect.x + 8.0;
        let text_y = rect.y + (rect.height - 14.0) / 2.0;
        renderer.draw_text(&self.label, text_x, text_y, 14.0, [1.0, 1.0, 1.0, 1.0]);

        // Register interaction handlers
        let on_click = self.on_click.clone();

        renderer.register_handler(
            "pointerdown",
            std::sync::Arc::new(move |_| {
                cvkg_core::update_system_state(|s| {
                    let mut s = s.clone();
                    s.set_component_state(id_hash, true);
                    s
                });
            }),
        );

        renderer.register_handler(
            "pointerup",
            std::sync::Arc::new(move |_| {
                cvkg_core::update_system_state(|s| {
                    let mut s = s.clone();
                    s.set_component_state(id_hash, false);
                    s
                });
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

    fn layout(&self) -> Option<&dyn cvkg_core::layout::LayoutView> {
        Some(self)
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        _proposal: cvkg_core::layout::SizeProposal,
    ) -> cvkg_core::Size {
        let (tw, th) = renderer.measure_text(&self.label, 14.0);
        cvkg_core::Size {
            width: tw + 16.0,
            height: th + 12.0,
        }
    }
}

impl cvkg_core::layout::LayoutView for Button {
    fn size_that_fits(
        &self,
        _proposal: cvkg_core::layout::SizeProposal,
        _subviews: &[&dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: self.label.len() as f32 * 14.0 * 0.6 + 20.0,
            height: 32.0,
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) {
    }
}

#[derive(Clone)]
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

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        _proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let (tw, th) = renderer.measure_text(&self.label, 14.0);
        cvkg_core::Size {
            width: 40.0 + 8.0 + tw,
            height: th.max(20.0) + 4.0,
        }
    }
}

#[derive(Clone)]
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
        let is_dragging = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        let drag_start = is_dragging.clone();
        renderer.register_handler(
            "pointerdown",
            std::sync::Arc::new(move |_| {
                drag_start.store(true, std::sync::atomic::Ordering::Relaxed);
            }),
        );

        let drag_stop = is_dragging.clone();
        renderer.register_handler(
            "pointerup",
            std::sync::Arc::new(move |_| {
                drag_stop.store(false, std::sync::atomic::Ordering::Relaxed);
            }),
        );

        renderer.register_handler(
            "pointermove",
            std::sync::Arc::new(move |event| {
                if !is_dragging.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }
                if let cvkg_core::Event::PointerMove { x, .. } = event {
                    let pct = ((x - slider_rect.x) / slider_rect.width).clamp(0.0, 1.0);
                    let val = start + pct * (end - start);
                    (on_change)(val);
                }
            }),
        );
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(150.0),
            height: 24.0,
        }
    }
}

/// Stepper for discrete increment/decrement
#[derive(Clone)]
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

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        _proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let (lw, lh) = renderer.measure_text(&self.label, 14.0);
        let (vw, _) = renderer.measure_text(&self.value.to_string(), 14.0);
        cvkg_core::Size {
            width: lw + 8.0 + 30.0 + vw + 20.0 + 30.0 + 8.0,
            height: lh.max(30.0) + 8.0,
        }
    }
}

#[derive(Clone)]
pub struct Input {
    pub(crate) placeholder: String,
    pub(crate) text: String,
    pub(crate) on_change: std::sync::Arc<dyn Fn(String) + Send + Sync>,
    pub(crate) is_focused: bool,
}

impl Input {
    /// Create a new Input field.
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            text: String::new(),
            on_change: std::sync::Arc::new(|_| {}),
            is_focused: false,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.text = value.into();
        self
    }

    pub fn on_change(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_change = std::sync::Arc::new(callback);
        self
    }

    pub fn focused(mut self, is_focused: bool) -> Self {
        self.is_focused = is_focused;
        self
    }
}

impl View for Input {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Input");
        renderer.set_aria_role("textbox");
        renderer.set_aria_label(&self.placeholder);

        // Input background
        renderer.fill_rounded_rect(rect, 6.0, [0.08, 0.08, 0.12, 1.0]);
        renderer.stroke_rect(rect, [0.3, 0.3, 0.4, 1.0], 1.0);

        let is_focused = self.is_focused;
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

        let on_change_kd = on_change.clone();
        let text_mutex_kd = text_mutex.clone();
        renderer.register_handler(
            "keydown",
            std::sync::Arc::new(move |event| {
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
                            // Handle submission or blur?
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
            std::sync::Arc::new(move |event| {
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
        renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let text = if self.text.is_empty() {
            &self.placeholder
        } else {
            &self.text
        };
        let (tw, th) = renderer.measure_text(text, 14.0);
        cvkg_core::Size {
            width: proposal.width.unwrap_or(tw + 24.0).max(100.0),
            height: th + 16.0,
        }
    }
}

/// Secure password input
#[derive(Clone)]
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

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let text = if self.text.is_empty() {
            &self.placeholder
        } else {
            "*"
        }; // Proxy for measurement
        let (tw, th) = renderer.measure_text(text, 14.0);
        let width = if self.text.is_empty() {
            tw + 24.0
        } else {
            (self.text.len() as f32 * 10.0) + 24.0
        };
        cvkg_core::Size {
            width: proposal.width.unwrap_or(width).max(100.0),
            height: th + 16.0,
        }
    }
}

/// Multi-line text area
#[derive(Clone)]
pub struct Textarea {
    pub(crate) placeholder: String,
    pub(crate) text: String,
    pub(crate) rows: usize,
    pub(crate) on_change: std::sync::Arc<dyn Fn(String) + Send + Sync>,
}

impl Textarea {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            text: String::new(),
            rows: 3,
            on_change: std::sync::Arc::new(|_| {}),
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
        self.on_change = std::sync::Arc::new(callback);
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

        // Editor background
        renderer.fill_rounded_rect(rect, 4.0, [0.05, 0.05, 0.08, 1.0]);
        renderer.stroke_rect(rect, [0.2, 0.2, 0.3, 1.0], 1.0);

        // Draw text
        if self.text.is_empty() {
            renderer.draw_text(
                &self.placeholder,
                rect.x + 8.0,
                rect.y + 8.0,
                14.0,
                [0.4, 0.4, 0.45, 1.0],
            );
        } else {
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
        }

        // Draw Cursor on last line
        let lines: Vec<&str> = self.text.lines().collect();
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

        let on_change_kd = on_change.clone();
        let text_mutex_kd = text_mutex.clone();
        renderer.register_handler(
            "keydown",
            std::sync::Arc::new(move |event| {
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
            std::sync::Arc::new(move |event| {
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
            height: proposal.height.unwrap_or(200.0),
        }
    }
}

/// Dropdown component for selecting from a list of options with a popover
pub struct Dropdown {
    pub(crate) selection: usize,
    pub(crate) options: Vec<String>,
    pub(crate) on_change: std::sync::Arc<dyn Fn(usize) + Send + Sync>,
}

impl Dropdown {
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

impl View for Dropdown {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Dropdown");

        let id_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            "dropdown".hash(&mut s);
            self.options.len().hash(&mut s);
            s.finish()
        };

        // Lock-free read of expanded state
        let is_expanded = {
            let s = cvkg_core::load_system_state();
            s.get_component_state::<bool>(id_hash)
                .map(|v| *v.read().unwrap())
                .unwrap_or(false)
        };

        // Main button
        renderer.fill_rounded_rect(rect, 4.0, [0.1, 0.1, 0.15, 1.0]);
        renderer.stroke_rect(rect, [0.0, 0.9, 1.0, 1.0], 1.0);

        let selected = self
            .options
            .get(self.selection)
            .cloned()
            .unwrap_or_default();
        renderer.draw_text(
            &selected,
            rect.x + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        renderer.draw_text(
            if is_expanded { "▲" } else { "▼" },
            rect.x + rect.width - 20.0,
            rect.y + (rect.height - 14.0) / 2.0,
            12.0,
            [0.5, 0.5, 0.6, 1.0],
        );

        if is_expanded {
            let popover_h = (self.options.len() as f32 * 30.0).min(200.0);
            let popover_rect = Rect {
                x: rect.x,
                y: rect.y + rect.height + 4.0,
                width: rect.width,
                height: popover_h,
            };

            // Z-Index boost for popover
            renderer.set_z_index(100.0);
            renderer.bifrost(popover_rect, 20.0, 1.2, 0.9);
            renderer.fill_rounded_rect(popover_rect, 4.0, [0.05, 0.05, 0.1, 0.95]);
            renderer.stroke_rect(popover_rect, [0.0, 1.0, 1.0, 0.5], 1.0);

            for (i, opt) in self.options.iter().enumerate() {
                let item_rect = Rect {
                    x: popover_rect.x,
                    y: popover_rect.y + i as f32 * 30.0,
                    width: popover_rect.width,
                    height: 30.0,
                };

                if i == self.selection {
                    renderer.fill_rect(item_rect, [0.0, 0.5, 0.8, 0.3]);
                }

                renderer.draw_text(
                    opt,
                    item_rect.x + 8.0,
                    item_rect.y + (item_rect.height - 14.0) / 2.0,
                    14.0,
                    [1.0, 1.0, 1.0, 1.0],
                );
            }
            renderer.set_z_index(0.0);
        }

        let options_count = self.options.len();
        let on_change = self.on_change.clone();

        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |event| {
                if let cvkg_core::Event::PointerClick { x, y, .. } = event {
                    if is_expanded {
                        let popover_h = (options_count as f32 * 30.0).min(200.0);
                        let popover_rect = Rect {
                            x: rect.x,
                            y: rect.y + rect.height + 4.0,
                            width: rect.width,
                            height: popover_h,
                        };

                        if x >= popover_rect.x
                            && x <= popover_rect.x + popover_rect.width
                            && y >= popover_rect.y
                            && y <= popover_rect.y + popover_rect.height
                        {
                            let idx = ((y - popover_rect.y) / 30.0) as usize;
                            if idx < options_count {
                                on_change(idx);
                            }
                        }
                    }

                    // Toggle expanded state atomically
                    cvkg_core::update_system_state(|s| {
                        let mut s = s.clone();
                        s.set_component_state(id_hash, !is_expanded);
                        s
                    });
                }
            }),
        );

        renderer.pop_vnode();
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let mut max_w = 0.0f32;
        for opt in &self.options {
            let (w, _) = renderer.measure_text(opt, 14.0);
            max_w = max_w.max(w);
        }
        cvkg_core::Size {
            width: proposal.width.unwrap_or(max_w + 40.0).max(120.0),
            height: 32.0,
        }
    }
}

/// Picker for selection from a list of options
#[derive(Clone)]
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

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let mut max_w = 0.0f32;
        let mut max_h = 0.0f32;
        for opt in &self.options {
            let (w, h) = renderer.measure_text(opt, 14.0);
            max_w = max_w.max(w);
            max_h = max_h.max(h);
        }
        cvkg_core::Size {
            width: proposal.width.unwrap_or(max_w + 40.0).max(120.0),
            height: max_h + 16.0,
        }
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
                    if let cvkg_core::Event::PointerClick { x, .. } = event
                        && x >= cell_rect.x
                        && x <= cell_rect.x + cell_rect.width
                    {
                        (on_change)(col);
                    }
                }),
            );
        }
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(200.0),
            height: 32.0,
        }
    }
}

/// Checkbox component for boolean input.
#[derive(Clone)]
pub struct Checkbox {
    pub(crate) is_checked: bool,
    pub(crate) label: Option<String>,
    pub(crate) on_change: std::sync::Arc<dyn Fn(bool) + Send + Sync>,
}

impl Checkbox {
    /// Create a new Checkbox.
    pub fn new(is_checked: bool, on_change: impl Fn(bool) + Send + Sync + 'static) -> Self {
        Self {
            is_checked,
            label: None,
            on_change: std::sync::Arc::new(on_change),
        }
    }

    /// Set the label for the checkbox.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl View for Checkbox {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Checkbox");
        let box_size = 18.0;
        let box_rect = Rect {
            x: rect.x,
            y: rect.y + (rect.height - box_size) / 2.0,
            width: box_size,
            height: box_size,
        };
        let bg = if self.is_checked {
            [0.0, 0.7, 1.0, 1.0]
        } else {
            [0.12, 0.12, 0.15, 1.0]
        };
        renderer.fill_rounded_rect(box_rect, 3.0, bg);
        renderer.stroke_rect(box_rect, [0.3, 0.3, 0.4, 1.0], 1.0);
        if self.is_checked {
            renderer.draw_text(
                "✓",
                box_rect.x + 3.0,
                box_rect.y - 2.0,
                14.0,
                [1.0, 1.0, 1.0, 1.0],
            );
        }
        if let Some(label) = &self.label {
            renderer.draw_text(
                label,
                box_rect.x + box_size + 8.0,
                rect.y + (rect.height - 14.0) / 2.0,
                14.0,
                [1.0, 1.0, 1.0, 1.0],
            );
        }
        let is_checked = self.is_checked;
        let on_change = self.on_change.clone();
        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |_| {
                (on_change)(!is_checked);
            }),
        );
        renderer.pop_vnode();
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        _proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let label_width = self
            .label
            .as_ref()
            .map_or(0.0, |l| renderer.measure_text(l, 14.0).0);
        cvkg_core::Size {
            width: 18.0
                + if self.label.is_some() {
                    8.0 + label_width
                } else {
                    0.0
                },
            height: 22.0,
        }
    }
}

/// Radio Group for exclusive selection.
#[derive(Clone)]
pub struct RadioGroup<V> {
    options: Vec<(String, V)>,
    selected_index: usize,
    on_change: std::sync::Arc<dyn Fn(usize) + Send + Sync>,
}

impl<V: View + Clone> RadioGroup<V> {
    pub fn new(selected_index: usize, on_change: impl Fn(usize) + Send + Sync + 'static) -> Self {
        Self {
            options: Vec::new(),
            selected_index,
            on_change: std::sync::Arc::new(on_change),
        }
    }
    pub fn option(mut self, label: impl Into<String>, view: V) -> Self {
        self.options.push((label.into(), view));
        self
    }
}

impl<V: View + Clone> View for RadioGroup<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "RadioGroup");
        for (idx, (label, _)) in self.options.iter().enumerate() {
            let item_rect = Rect {
                x: rect.x,
                y: rect.y + idx as f32 * 24.0,
                width: rect.width,
                height: 24.0,
            };
            renderer.push_vnode(item_rect, "RadioItem");

            let dot_radius = if idx == self.selected_index { 5.0 } else { 4.0 };
            renderer.fill_rounded_rect(
                Rect {
                    x: rect.x + 9.0 - dot_radius,
                    y: rect.y + idx as f32 * 24.0 + 12.0 - dot_radius,
                    width: dot_radius * 2.0,
                    height: dot_radius * 2.0,
                },
                dot_radius,
                if idx == self.selected_index {
                    [0.0, 0.8, 1.0, 1.0]
                } else {
                    [0.15, 0.15, 0.2, 1.0]
                },
            );
            if idx != self.selected_index {
                renderer.stroke_rect(
                    Rect {
                        x: rect.x + 9.0 - dot_radius,
                        y: rect.y + idx as f32 * 24.0 + 12.0 - dot_radius,
                        width: dot_radius * 2.0,
                        height: dot_radius * 2.0,
                    },
                    [0.4, 0.4, 0.5, 1.0],
                    1.0,
                );
            }
            renderer.draw_text(
                label,
                rect.x + 22.0,
                rect.y + idx as f32 * 24.0 + 11.0,
                14.0,
                [1.0, 1.0, 1.0, 1.0],
            );

            let on_change = self.on_change.clone();
            renderer.register_handler(
                "pointerclick",
                Arc::new(move |_| {
                    on_change(idx);
                }),
            );
            renderer.pop_vnode();
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let max_width = self
            .options
            .iter()
            .map(|(l, _)| renderer.measure_text(l, 14.0).0)
            .fold(0.0, f32::max);
        cvkg_core::Size {
            width: (proposal.width.unwrap_or(max_width + 30.0)).max(max_width + 30.0),
            height: self.options.len() as f32 * 24.0,
        }
    }
}

/// Tabs component for tabbed navigation.
#[derive(Clone)]
pub struct Tabs<V> {
    tabs: Vec<(String, V)>,
    selected_index: usize,
}

impl<V: View> Tabs<V> {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            selected_index: 0,
        }
    }
    pub fn tab(mut self, label: impl Into<String>, content: V) -> Self {
        self.tabs.push((label.into(), content));
        self
    }
    pub fn selected(mut self, index: usize) -> Self {
        self.selected_index = index.min(self.tabs.len().saturating_sub(1));
        self
    }
}

impl<V: View> Default for Tabs<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: View> View for Tabs<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Tabs");
        let tab_height = 36.0;
        for (idx, (label, _)) in self.tabs.iter().enumerate() {
            let tab_rect = Rect {
                x: rect.x + idx as f32 * (rect.width / self.tabs.len() as f32),
                y: rect.y,
                width: rect.width / self.tabs.len() as f32,
                height: tab_height,
            };
            let is_selected = idx == self.selected_index;
            renderer.fill_rounded_rect(
                tab_rect,
                6.0,
                if is_selected {
                    [0.15, 0.15, 0.2, 1.0]
                } else {
                    [0.08, 0.08, 0.12, 1.0]
                },
            );
            if is_selected {
                renderer.stroke_rect(tab_rect, [0.0, 0.8, 1.0, 1.0], 2.0);
            }
            renderer.draw_text(
                label,
                tab_rect.x + 12.0,
                tab_rect.y + (tab_rect.height - 14.0) / 2.0,
                14.0,
                if is_selected {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    [0.7, 0.7, 0.75, 1.0]
                },
            );
        }
        if let Some((_, content)) = self.tabs.get(self.selected_index) {
            let content_rect = Rect {
                x: rect.x,
                y: rect.y + tab_height + 8.0,
                width: rect.width,
                height: rect.height - tab_height - 8.0,
            };
            content.render(renderer, content_rect);
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let max_h = self
            .tabs
            .iter()
            .map(|(_, c)| c.intrinsic_size(renderer, proposal).height)
            .fold(0.0, f32::max);
        cvkg_core::Size {
            width: proposal.width.unwrap_or(300.0),
            height: 36.0 + 8.0 + max_h,
        }
    }
}

/// Select/Dropdown component.
#[derive(Clone)]
pub struct Select<V> {
    placeholder: String,
    options: Vec<(String, V)>,
    selected_index: Option<usize>,
}

impl<V: Clone> Select<V> {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            options: Vec::new(),
            selected_index: None,
        }
    }
    pub fn option(mut self, label: impl Into<String>, value: V) -> Self {
        self.options.push((label.into(), value));
        self
    }
}

impl<V: Clone + View> View for Select<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Select");
        renderer.fill_rounded_rect(rect, 6.0, [0.1, 0.1, 0.15, 1.0]);
        renderer.stroke_rect(rect, [0.3, 0.3, 0.4, 1.0], 1.0);
        let display_text = self
            .selected_index
            .and_then(|i| self.options.get(i))
            .map(|(l, _)| l.as_str())
            .unwrap_or(&self.placeholder);
        renderer.draw_text(
            display_text,
            rect.x + 12.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            if self.selected_index.is_some() {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.5, 0.5, 0.55, 1.0]
            },
        );
        renderer.pop_vnode();
    }
    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(150.0),
            height: 36.0,
        }
    }
}
/// ValkyrSelect - A tactical chooser/select component.
/// Named after the Valkyries, the "Choosers of the Slain".
#[derive(Clone)]
pub struct ValkyrSelect {
    pub(crate) options: Vec<String>,
    pub(crate) selected: Option<String>,
    pub(crate) placeholder: String,
    pub(crate) on_change: std::sync::Arc<dyn Fn(String) + Send + Sync>,
}

impl ValkyrSelect {
    /// Creates a new ValkyrSelect with the given options and change handler.
    pub fn new(options: Vec<String>, on_change: impl Fn(String) + Send + Sync + 'static) -> Self {
        Self {
            options,
            selected: None,
            placeholder: "Choose...".into(),
            on_change: std::sync::Arc::new(on_change),
        }
    }

    /// Sets the placeholder text.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Sets the currently selected value.
    pub fn selected(mut self, value: impl Into<String>) -> Self {
        self.selected = Some(value.into());
        self
    }
}

impl View for ValkyrSelect {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ValkyrSelect");

        // 1. Selector Base
        renderer.fill_rounded_rect(rect, 4.0, [0.1, 0.1, 0.15, 1.0]);
        renderer.stroke_rect(rect, [0.3, 0.3, 0.4, 1.0], 1.0);

        let display = self.selected.as_ref().unwrap_or(&self.placeholder);
        let text_color = if self.selected.is_none() {
            [0.5, 0.5, 0.6, 1.0]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        };

        renderer.draw_text(
            display,
            rect.x + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            text_color,
        );
        renderer.draw_text(
            "▼",
            rect.x + rect.width - 20.0,
            rect.y + (rect.height - 14.0) / 2.0,
            10.0,
            [0.6, 0.6, 0.7, 1.0],
        );

        // 2. Interaction State
        let id_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            "valkyrselect".hash(&mut s);
            rect.x.to_bits().hash(&mut s);
            rect.y.to_bits().hash(&mut s);
            s.finish()
        };

        let is_open = {
            let s = cvkg_core::load_system_state();
            s.get_component_state::<bool>(id_hash)
                .map(|v| *v.read().unwrap())
                .unwrap_or(false)
        };

        // 3. Popover Menu
        if is_open {
            let menu_h = (self.options.len() as f32 * 32.0).min(200.0);
            let menu_rect = Rect {
                x: rect.x,
                y: rect.y + rect.height + 4.0,
                width: rect.width,
                height: menu_h,
            };

            renderer.set_z_index(100.0);
            renderer.bifrost(menu_rect, 15.0, 1.1, 0.9);
            renderer.stroke_rect(menu_rect, [0.0, 0.9, 1.0, 0.6], 1.0);

            for (i, opt) in self.options.iter().enumerate() {
                let opt_rect = Rect {
                    x: menu_rect.x,
                    y: menu_rect.y + i as f32 * 32.0,
                    width: menu_rect.width,
                    height: 32.0,
                };

                let is_hovered = false; // Mocked for simplicity
                if is_hovered {
                    renderer.fill_rect(opt_rect, [0.0, 0.8, 1.0, 0.2]);
                }

                renderer.draw_text(
                    opt,
                    opt_rect.x + 12.0,
                    opt_rect.y + 10.0,
                    13.0,
                    [1.0, 1.0, 1.0, 0.9],
                );
            }
            renderer.set_z_index(0.0);
        }

        // 4. Handlers
        let on_change = self.on_change.clone();
        let options = self.options.clone();

        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event| {
                if let cvkg_core::Event::PointerClick { x: _, y, .. } = event {
                    if !is_open {
                        cvkg_core::update_system_state(|s| {
                            let mut s = s.clone();
                            s.set_component_state(id_hash, true);
                            s
                        });
                    } else {
                        // Check if clicked an option
                        let local_y = y - (rect.y + rect.height + 4.0);
                        if local_y >= 0.0 && local_y < (options.len() as f32 * 32.0) {
                            let idx = (local_y / 32.0) as usize;
                            if let Some(opt) = options.get(idx) {
                                on_change(opt.clone());
                            }
                        }

                        cvkg_core::update_system_state(|s| {
                            let mut s = s.clone();
                            s.set_component_state(id_hash, false);
                            s
                        });
                    }
                }
            }),
        );

        renderer.pop_vnode();
    }
}

/// HringrPagination - A cyclic pagination component for navigating data loops.
/// Named after the rings/circles (Hringr), representing the eternal cycle.
#[derive(Clone)]
pub struct HringrPagination {
    pub current_page: usize,
    pub total_pages: usize,
    pub on_change: std::sync::Arc<dyn Fn(usize) + Send + Sync>,
}

impl HringrPagination {
    /// Creates a new HringrPagination.
    pub fn new(total_pages: usize, on_change: impl Fn(usize) + Send + Sync + 'static) -> Self {
        Self {
            current_page: 1,
            total_pages,
            on_change: std::sync::Arc::new(on_change),
        }
    }

    /// Sets the current page.
    pub fn current_page(mut self, page: usize) -> Self {
        self.current_page = page.clamp(1, self.total_pages);
        self
    }
}

impl View for HringrPagination {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "HringrPagination");

        let btn_w = 32.0;
        let spacing = 4.0;
        let mut current_x = rect.x;

        // 1. Previous Button
        let prev_rect = Rect {
            x: current_x,
            y: rect.y,
            width: btn_w,
            height: rect.height,
        };
        renderer.fill_rounded_rect(prev_rect, 4.0, [0.1, 0.1, 0.15, 1.0]);
        renderer.draw_text(
            "<",
            prev_rect.x + 10.0,
            prev_rect.y + 10.0,
            14.0,
            [0.8, 0.8, 0.9, 1.0],
        );
        current_x += btn_w + spacing;

        // 2. Page Numbers (Simplified)
        for i in 1..=self.total_pages.min(5) {
            let page_rect = Rect {
                x: current_x,
                y: rect.y,
                width: btn_w,
                height: rect.height,
            };
            let is_selected = i == self.current_page;
            let bg = if is_selected {
                [0.0, 0.8, 1.0, 0.4]
            } else {
                [0.05, 0.05, 0.08, 1.0]
            };

            renderer.fill_rounded_rect(page_rect, 4.0, bg);
            if is_selected {
                renderer.stroke_rect(page_rect, [0.0, 1.0, 1.0, 0.8], 1.0);
            }

            renderer.draw_text(
                &i.to_string(),
                page_rect.x + 10.0,
                page_rect.y + 10.0,
                13.0,
                [1.0, 1.0, 1.0, 0.9],
            );
            current_x += btn_w + spacing;
        }

        // 3. Next Button
        let next_rect = Rect {
            x: current_x,
            y: rect.y,
            width: btn_w,
            height: rect.height,
        };
        renderer.fill_rounded_rect(next_rect, 4.0, [0.1, 0.1, 0.15, 1.0]);
        renderer.draw_text(
            ">",
            next_rect.x + 10.0,
            next_rect.y + 10.0,
            14.0,
            [0.8, 0.8, 0.9, 1.0],
        );

        renderer.pop_vnode();
    }
}

/// ValhallaRating - A tactical rating component for assessing quality.
/// Named after Valhalla, where the chosen are assessed for their worth.
pub struct ValhallaRating {
    pub value: f32,
    pub max: usize,
}

impl ValhallaRating {
    /// Creates a new ValhallaRating.
    pub fn new(value: f32) -> Self {
        Self { value, max: 5 }
    }

    /// Sets the maximum rating value.
    pub fn max(mut self, max: usize) -> Self {
        self.max = max;
        self
    }
}

impl View for ValhallaRating {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ValhallaRating");

        let t = renderer.elapsed_time();
        let star_w = rect.width / self.max as f32;
        let star_h = rect.height;

        for i in 0..self.max {
            let star_rect = Rect {
                x: rect.x + i as f32 * star_w,
                y: rect.y,
                width: star_w * 0.8,
                height: star_h,
            };

            let is_filled = (i as f32) < self.value;

            // 1. Bifrost Resonance (Glowing Star Spirits)
            let resonance = if is_filled {
                (t * 2.0 + i as f32).sin() * 0.2 + 0.8
            } else {
                1.0
            };
            let color = if is_filled {
                [1.0, 0.84, 0.0, 0.9 * resonance] // Viking Gold with resonance
            } else {
                [0.2, 0.2, 0.25, 0.3] // Dimmed stone
            };

            renderer.fill_ellipse(star_rect, color);
            if is_filled {
                // Einherjar Spirit Glow
                renderer.stroke_ellipse(star_rect, [1.0, 1.0, 0.5, 0.4 * resonance], 1.5);
            }
        }

        renderer.pop_vnode();
    }
}

/// BifrostColorPicker - A color selection component.
/// Named after the Bifrost, the rainbow bridge connecting the realms.
pub struct BifrostColorPicker {
    pub color: [f32; 4],
}

impl BifrostColorPicker {
    /// Creates a new BifrostColorPicker.
    pub fn new(color: [f32; 4]) -> Self {
        Self { color }
    }
}

impl View for BifrostColorPicker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "BifrostColorPicker");

        // 1. Rainbow Track (Bifrost Bridge)
        let track_h = rect.height * 0.2;
        let track_rect = Rect {
            x: rect.x,
            y: rect.y + (rect.height - track_h) / 2.0,
            width: rect.width,
            height: track_h,
        };

        // Mocking a rainbow gradient with segments
        let segments = 6;
        let seg_w = rect.width / segments as f32;
        let colors = [
            [1.0, 0.0, 0.0, 1.0],
            [1.0, 0.5, 0.0, 1.0],
            [1.0, 1.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
            [0.5, 0.0, 1.0, 1.0],
        ];

        for i in 0..segments {
            renderer.fill_rect(
                Rect {
                    x: rect.x + i as f32 * seg_w,
                    y: track_rect.y,
                    width: seg_w,
                    height: track_h,
                },
                colors[i],
            );
        }

        // 2. Mimir's Refraction (Refractive Color Indicator)
        // Heimdall's Watch: A magnifying glass effect over the selection
        let indicator_size = rect.height * 0.9;
        let indicator_rect = Rect {
            x: rect.x + (rect.width - indicator_size) / 2.0,
            y: rect.y + (rect.height - indicator_size) / 2.0,
            width: indicator_size,
            height: indicator_size,
        };

        // Advanced refractive lensing
        renderer.bifrost(indicator_rect, indicator_size / 2.0, 2.0, 0.98);
        renderer.fill_ellipse(indicator_rect, self.color);

        // Surtur's Reactive Materials (Glow Ring)
        let t = renderer.elapsed_time();
        let pulse = (t * 3.0).sin() * 0.1 + 0.9;
        renderer.stroke_ellipse(indicator_rect, [1.0, 1.0, 1.0, 0.7 * pulse], 2.0);

        renderer.pop_vnode();
    }
}
