use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::Arc;

/// Vegvísir - A radial tactical menu (Norse compass)
#[derive(Clone)]
pub struct Vegvísir {
    pub items: Vec<VegvísirItem>,
    pub is_open: bool,
    pub on_select: Arc<dyn Fn(usize) + Send + Sync>,
}

#[derive(Clone)]
pub struct VegvísirItem {
    pub icon: String,
    pub label: String,
}

impl Vegvísir {
    pub fn new(on_select: impl Fn(usize) + Send + Sync + 'static) -> Self {
        Self {
            items: Vec::new(),
            is_open: false,
            on_select: Arc::new(on_select),
        }
    }

    pub fn add_item(mut self, icon: &str, label: &str) -> Self {
        self.items.push(VegvísirItem {
            icon: icon.to_string(),
            label: label.to_string(),
        });
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }
}

impl View for Vegvísir {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.is_open || self.items.is_empty() {
            renderer.set_aria_role("navigation");
            return;
        }

        renderer.push_vnode(rect, "Vegvísir");

        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        let radius = (rect.width / 2.0).min(rect.height / 2.0) * 0.6;

        let segment_angle = 2.0 * std::f32::consts::PI / self.items.len() as f32;

        for (i, item) in self.items.iter().enumerate() {
            let angle = segment_angle * i as f32 - std::f32::consts::PI / 2.0;
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();

            renderer.fill_rounded_rect(
                Rect {
                    x: x - 30.0,
                    y: y - 30.0,
                    width: 60.0,
                    height: 60.0,
                },
                30.0,
                theme::accent(),
            );

            renderer.draw_text(&item.label, x - 20.0, y + 5.0, 10.0, theme::text());
        }

        let on_select = self.on_select.clone();
        let items_len = self.items.len();

        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event| {
                if let cvkg_core::Event::PointerClick { x, y, .. } = event {
                    let dx = x - center_x;
                    let dy = y - center_y;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist >= radius - 40.0 && dist <= radius + 40.0 {
                        let mut angle = dy.atan2(dx) + std::f32::consts::PI / 2.0;
                        if angle < 0.0 {
                            angle += 2.0 * std::f32::consts::PI;
                        }

                        let idx = ((angle / segment_angle) + 0.5) as usize % items_len;
                        on_select(idx);
                    }
                }
            }),
        );

        renderer.pop_vnode();
    }
}

/// TacticalGauge - A high-fidelity HUD gauge for monitoring real-time kinetics.
#[derive(Clone)]
pub struct TacticalGauge {
    pub label: String,
    pub value: f32, // [0.0, 1.0]
    pub color: [f32; 4],
    pub warning_level: f32,
    pub critical_level: f32,
}

impl TacticalGauge {
    pub fn new(label: &str, value: f32) -> Self {
        Self {
            label: label.to_string(),
            value: value.clamp(0.0, 1.0),
            color: theme::accent(),
            warning_level: 0.7,
            critical_level: 0.9,
        }
    }

    pub fn warning_level(mut self, warning_level: f32) -> Self {
        self.warning_level = warning_level;
        self
    }

    pub fn critical_level(mut self, critical_level: f32) -> Self {
        self.critical_level = critical_level;
        self
    }
}

impl View for TacticalGauge {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();

        renderer.set_aria_role("meter");
        // 1. Label
        renderer.draw_text(
            &self.label,
            rect.x,
            rect.y - 5.0,
            10.0,
            theme::text(),
        );

        // 2. Background Track
        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: 4.0,
            },
            theme::surface_elevated(),
        );

        // 3. Fill
        let mut color = self.color;
        if self.value >= self.critical_level {
            color = theme::error_color();
        } else if self.value >= self.warning_level {
            color = theme::warning();
        }

        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width * self.value,
                height: 4.0,
            },
            color,
        );

        // 4. Kinetic Flicker (Micro-animation)
        let flicker = (t * 20.0).sin() * 0.1 + 0.9;
        if self.value > 0.0 {
            renderer.fill_rect(
                Rect {
                    x: rect.x + rect.width * self.value - 2.0,
                    y: rect.y - 2.0,
                    width: 2.0,
                    height: 8.0,
                },
                [color[0], color[1], color[2], color[3] * flicker],
            );
        }
    }
}
/// GjallarAlert - A high-priority tactical notification (toast).
/// Named after the Gjallarhorn, the loud horn used to signal danger or major events.
#[derive(Clone)]
pub struct GjallarAlert {
    pub title: String,
    pub message: String,
    pub kind: AlertKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    Information,
    Warning,
    Critical,
}

impl GjallarAlert {
    /// Creates a new GjallarAlert.
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            kind: AlertKind::Information,
        }
    }

    /// Sets the alert severity level.
    pub fn kind(mut self, kind: AlertKind) -> Self {
        self.kind = kind;
        self
    }
}

impl View for GjallarAlert {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "GjallarAlert");

        renderer.set_aria_role("alert");
        let t = renderer.elapsed_time();
        let accent_color = match self.kind {
            AlertKind::Information => theme::accent(),   // Cyan
            AlertKind::Warning => theme::warning(),      // Orange
            AlertKind::Critical => theme::error_color(), // Red
        };

        // 1. Mimir's Refraction (Glass Depth)
        // Simulate refractive thickness by layering bifrost with slight offsets
        renderer.bifrost(rect, 15.0, 1.5, 0.95);
        renderer.fill_rounded_rect(rect, 4.0, theme::surface_elevated());

        // Secondary internal refraction line
        let inner_rect = Rect {
            x: rect.x + 2.0,
            y: rect.y + 2.0,
            width: rect.width - 4.0,
            height: rect.height - 4.0,
        };
        renderer.stroke_rounded_rect(
            inner_rect,
            4.0,
            [accent_color[0], accent_color[1], accent_color[2], 0.2],
            0.5,
        );

        // 2. Surtur's Reactive Materials (Kinetic Glow)
        let pulse = (t * 4.0).sin() * 0.2 + 0.8;
        let border_alpha = 0.4 * pulse;
        renderer.stroke_rect(
            rect,
            [
                accent_color[0],
                accent_color[1],
                accent_color[2],
                border_alpha,
            ],
            1.5,
        );

        // 3. Gjallarhorn Signal Line (Side Bar)
        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: 4.0,
                height: rect.height,
            },
            accent_color,
        );

        // 4. Text Content (Inscribed wisdom)
        renderer.draw_text(
            &self.title,
            rect.x + 12.0,
            rect.y + 10.0,
            14.0,
            accent_color,
        );
        renderer.draw_text(
            &self.message,
            rect.x + 12.0,
            rect.y + 28.0,
            12.0,
            theme::text(),
        );

        renderer.pop_vnode();
    }
}
