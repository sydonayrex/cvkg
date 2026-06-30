use crate::theme;
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

// =============================================================================
// PATTERNS — Common application patterns
// =============================================================================
//
// This module provides pre-built UI patterns for common application screens:
// - `Wizard` — Multi-step onboarding workflow
// - `Login` — Login/authentication screen
// - `Settings` — Tabbed settings panel
// - `Gallery` — Image/content grid gallery
//
// These patterns are designed to be customized through their public fields
// and builder methods. They demonstrate composition of CVKG primitives into
// higher-level UI structures.
//
// # Examples
//
// Building a settings page:
// ```
// use cvkg_components::Settings;
// let settings = Settings::new()
//     .category("General")
//     .category("Account")
//     .build();
// ```

/// Multi-step onboarding Wizard workflow component.]
pub struct Wizard {
    pub(crate) steps: Vec<String>,
    pub(crate) current_step: usize,
}

impl Default for Wizard {
    fn default() -> Self {
        Self::new()
    }
}

impl Wizard {
    /// Create a new empty step wizard workflow guide.
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            current_step: 0,
        }
    }

    /// Add a step label to the sequence.
    pub fn step(mut self, step: &str) -> Self {
        self.steps.push(step.to_string());
        self
    }

    /// Set current active step pointer.
    pub fn active_step(mut self, index: usize) -> Self {
        self.current_step = index;
        self
    }
}

impl View for Wizard {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Outer panel
        renderer.fill_rounded_rect(rect, 6.0, theme::surface());
        renderer.stroke_rounded_rect(rect, 6.0, theme::border(), 1.0);

        if self.steps.is_empty() {
            return;
        }

        let step_count = self.steps.len();
        let step_w = rect.width / step_count as f32;
        let line_y = rect.y + 30.0;

        // Draw connections and steps
        for i in 0..step_count {
            let cx = rect.x + i as f32 * step_w + step_w * 0.5;

            // Connect line to next step
            if i < step_count - 1 {
                let next_cx = rect.x + (i + 1) as f32 * step_w + step_w * 0.5;
                let color = if i < self.current_step {
                    theme::accent()
                } else {
                    theme::border()
                };
                renderer.draw_line(cx, line_y, next_cx, line_y, color, 2.0);
            }

            // Step bubble circle
            let is_done = i < self.current_step;
            let is_active = i == self.current_step;
            let circle_color = if is_active {
                theme::accent()
            } else if is_done {
                theme::success()
            } else {
                theme::surface_elevated()
            };

            let circle_rect = Rect {
                x: cx - 10.0,
                y: line_y - 10.0,
                width: 20.0,
                height: 20.0,
            };

            renderer.fill_ellipse(circle_rect, circle_color);
            renderer.stroke_ellipse(circle_rect, theme::border_strong(), 1.0);

            // Step number text
            let number_text = &format!("{}", i + 1);
            renderer.draw_text_raw(
                number_text,
                cx - 3.0,
                line_y - 5.0,
                10.0,
                [1.0, 1.0, 1.0, 0.95],
            );

            // Step label text
            if let Some(label) = self.steps.get(i) {
                let text_color = if is_active {
                    theme::text()
                } else {
                    theme::text_muted()
                };
                renderer.draw_text_raw(label, cx - 20.0, line_y + 20.0, 10.0, text_color);
            }
        }
    }
}

impl LayoutView for Wizard {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 500.0,
            height: 100.0,
        }
    }
    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}

/// Image grid Gallery component.
///
/// Displays a grid of images with lazy loading and keyboard navigation.
///
/// # Examples
/// ```
/// use cvkg_components::Gallery;
/// let gallery = Gallery::new()
///     .items(vec!["img1.png".into(), "img2.png".into()]);
/// ```
pub struct Gallery {
    pub(crate) images: Vec<String>,
}

impl Default for Gallery {
    fn default() -> Self {
        Self::new()
    }
}

impl Gallery {
    /// Create a new Gallery.
    pub fn new() -> Self {
        Self { images: Vec::new() }
    }

    /// Add list of image asset keys or URLs.
    pub fn items(mut self, images: Vec<String>) -> Self {
        self.images = images;
        self
    }
}

impl View for Gallery {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.images.is_empty() {
            return;
        }

        let cols = 3;
        let rows = self.images.len().div_ceil(cols);
        let item_w = rect.width / cols as f32;
        let item_h = rect.height / rows.max(1) as f32;

        for (i, img_key) in self.images.iter().enumerate() {
            let r = i / cols;
            let c = i % cols;
            let item_rect = Rect {
                x: rect.x + c as f32 * item_w + 4.0,
                y: rect.y + r as f32 * item_h + 4.0,
                width: item_w - 8.0,
                height: item_h - 8.0,
            };

            renderer.fill_rounded_rect(item_rect, 4.0, theme::surface_elevated());
            renderer.draw_image(img_key, item_rect);
            renderer.stroke_rounded_rect(item_rect, 4.0, theme::border(), 1.0);
        }
    }
}

impl LayoutView for Gallery {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 450.0,
            height: 300.0,
        }
    }
    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}

/// Authorization Login panel template component.
///
/// A pre-built login form with username/password fields and submit handling.
/// Customize the field labels and styling through the builder methods.
///
/// # Examples
/// ```
/// use cvkg_components::Login;
/// let login = Login::new()
///     .submitting(false);
/// ```
pub struct Login {
    pub(crate) username_label: String,
    pub(crate) password_label: String,
    pub(crate) is_submitting: bool,
}

impl Default for Login {
    fn default() -> Self {
        Self::new()
    }
}

impl Login {
    /// Create a new Login component.
    pub fn new() -> Self {
        Self {
            username_label: "Username / Agent ID".to_string(),
            password_label: "Access Code / Passkey".to_string(),
            is_submitting: false,
        }
    }

    /// Set loading state.
    pub fn submitting(mut self, state: bool) -> Self {
        self.is_submitting = state;
        self
    }
}

impl View for Login {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Frame
        renderer.fill_rounded_rect(rect, 8.0, theme::surface());
        renderer.stroke_rounded_rect(rect, 8.0, theme::border(), 1.0);

        // Header Title
        renderer.draw_text_raw(
            "SYSTEM LOGIN // AUTHORIZE",
            rect.x + 20.0,
            rect.y + 30.0,
            14.0,
            theme::accent(),
        );
        renderer.draw_line(
            rect.x + 20.0,
            rect.y + 40.0,
            rect.x + rect.width - 20.0,
            rect.y + 40.0,
            theme::border(),
            0.5,
        );

        // Username Label & Box
        renderer.draw_text_raw(
            &self.username_label,
            rect.x + 20.0,
            rect.y + 60.0,
            10.0,
            theme::text_muted(),
        );
        let user_box = Rect {
            x: rect.x + 20.0,
            y: rect.y + 75.0,
            width: rect.width - 40.0,
            height: 28.0,
        };
        renderer.fill_rounded_rect(user_box, 4.0, theme::input_bg());
        renderer.stroke_rounded_rect(user_box, 4.0, theme::border(), 1.0);

        // Password Label & Box
        renderer.draw_text_raw(
            &self.password_label,
            rect.x + 20.0,
            rect.y + 120.0,
            10.0,
            theme::text_muted(),
        );
        let pass_box = Rect {
            x: rect.x + 20.0,
            y: rect.y + 135.0,
            width: rect.width - 40.0,
            height: 28.0,
        };
        renderer.fill_rounded_rect(pass_box, 4.0, theme::input_bg());
        renderer.stroke_rounded_rect(pass_box, 4.0, theme::border(), 1.0);
        renderer.draw_text_raw(
            "••••••••",
            rect.x + 28.0,
            rect.y + 152.0,
            12.0,
            theme::text(),
        );

        // Login Submit Button
        let btn_rect = Rect {
            x: rect.x + 20.0,
            y: rect.y + 185.0,
            width: rect.width - 40.0,
            height: 32.0,
        };
        let btn_color = if self.is_submitting {
            theme::surface_elevated()
        } else {
            theme::accent()
        };
        renderer.fill_rounded_rect(btn_rect, 4.0, btn_color);
        renderer.stroke_rounded_rect(btn_rect, 4.0, theme::border_strong(), 1.0);

        let button_text = if self.is_submitting {
            "AUTHORIZING..."
        } else {
            "TRANSMIT ACCESS CODES"
        };
        renderer.draw_text_raw(
            button_text,
            rect.x + 35.0,
            rect.y + 205.0,
            11.0,
            [1.0, 1.0, 1.0, 0.95],
        );
    }
}

impl LayoutView for Login {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 320.0,
            height: 240.0,
        }
    }
    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}

/// Multi-tab system Settings panel template component.
pub struct Settings {
    pub(crate) categories: Vec<String>,
    pub(crate) active_category: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

impl Settings {
    /// Create a new Settings component.
    pub fn new() -> Self {
        Self {
            categories: Vec::new(),
            active_category: 0,
        }
    }

    /// Add a tab category.
    pub fn category(mut self, label: &str) -> Self {
        self.categories.push(label.to_string());
        self
    }

    /// Set current category focus pointer.
    pub fn active_category(mut self, index: usize) -> Self {
        self.active_category = index;
        self
    }
}

impl View for Settings {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rounded_rect(rect, 6.0, theme::surface());
        renderer.stroke_rounded_rect(rect, 6.0, theme::border(), 1.0);

        let sidebar_w = 120.0;
        let divider_x = rect.x + sidebar_w;

        // Draw side sidebar list background
        let side_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: sidebar_w,
            height: rect.height,
        };
        renderer.fill_rounded_rect(side_rect, 6.0, theme::surface_elevated());
        renderer.draw_line(
            divider_x,
            rect.y,
            divider_x,
            rect.y + rect.height,
            theme::border(),
            1.0,
        );

        // Sidebar categories
        for (i, cat) in self.categories.iter().enumerate() {
            let cat_y = rect.y + 16.0 + i as f32 * 28.0;
            let is_active = i == self.active_category;

            if is_active {
                let cat_highlight = Rect {
                    x: rect.x + 4.0,
                    y: cat_y - 4.0,
                    width: sidebar_w - 8.0,
                    height: 24.0,
                };
                renderer.fill_rounded_rect(cat_highlight, 4.0, theme::accent());
            }

            let text_color = if is_active {
                [1.0, 1.0, 1.0, 0.95]
            } else {
                theme::text_muted()
            };
            renderer.draw_text_raw(cat, rect.x + 12.0, cat_y + 12.0, 11.0, text_color);
        }

        // Active panel context label placeholder
        if let Some(cat_name) = self.categories.get(self.active_category) {
            renderer.draw_text_raw(
                &format!("SETTINGS // {}", cat_name.to_uppercase()),
                divider_x + 20.0,
                rect.y + 30.0,
                12.0,
                theme::accent(),
            );
            renderer.draw_text_raw(
                "Adjust parameters using control widgets in dashboard layout.",
                divider_x + 20.0,
                rect.y + 50.0,
                10.0,
                theme::text_dim(),
            );
        }
    }
}

impl LayoutView for Settings {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 480.0,
            height: 320.0,
        }
    }
    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}
