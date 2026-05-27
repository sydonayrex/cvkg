//! CVKG Component Showcase
//!
//! A living demo of every CVKG component with interactive controls,
//! theme switching, and live source code references.
//!
//! Keyboard shortcuts (OS-agnostic: Cmd on macOS, Ctrl elsewhere):
//!   Cmd+T    — Toggle dark/light theme
//!   Cmd+M    — Toggle reduce motion
//!   Cmd+B    — Toggle reduce transparency

use cvkg::prelude::*;
use cvkg_components::theme;

/// Root showcase application.
struct ShowcaseApp {
    current_section: u32,
    theme_dark: bool,
    reduce_motion: bool,
    reduce_transparency: bool,
    increase_contrast: bool,
    zoom: f32,
}

const SIDEBAR_W: f32 = 220.0;

impl ShowcaseApp {
    fn new() -> Self {
        Self {
            current_section: 0,
            theme_dark: true,
            reduce_motion: false,
            reduce_transparency: false,
            increase_contrast: false,
            zoom: 1.0,
        }
    }
}

impl View for ShowcaseApp {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ShowcaseRoot");

        let sidebar_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: SIDEBAR_W,
            height: rect.height,
        };
        let content_rect = Rect {
            x: rect.x + SIDEBAR_W,
            y: rect.y,
            width: rect.width - SIDEBAR_W,
            height: rect.height,
        };

        self.render_sidebar(renderer, sidebar_rect);
        self.render_content(renderer, content_rect);

        renderer.pop_vnode();
    }
}

impl ShowcaseApp {
    fn render_sidebar(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ShowcaseSidebar");
        renderer.fill_rect(rect, theme::surface());
        renderer.stroke_rect(
            Rect {
                x: rect.x + rect.width - 1.0,
                y: rect.y,
                width: 1.0,
                height: rect.height,
            },
            theme::border(),
            1.0,
        );

        let pad = 12.0;
        let item_h: f32 = 40.0;
        let sections: &[(u32, &str)] = &[
            (0, "Buttons"),
            (1, "Inputs"),
            (2, "Layout"),
            (3, "Overlays"),
            (4, "Visual"),
            (5, "Typography"),
            (6, "Forms"),
            (7, "Accessibility"),
        ];

        renderer.draw_text(
            "CVKG Showcase",
            rect.x + pad,
            rect.y + pad,
            18.0,
            theme::text(),
        );

        for &(id, label) in sections {
            let item_y = rect.y + pad + 32.0 + pad + id as f32 * item_h;
            let is_active = self.current_section == id;

            if is_active {
                renderer.fill_rect(
                    Rect {
                        x: rect.x + 4.0,
                        y: item_y + 2.0,
                        width: rect.width - 8.0,
                        height: item_h - 4.0,
                    },
                    [
                        theme::accent()[0],
                        theme::accent()[1],
                        theme::accent()[2],
                        0.15,
                    ],
                );
                renderer.fill_rect(
                    Rect {
                        x: rect.x,
                        y: item_y + 4.0,
                        width: 3.0,
                        height: item_h - 8.0,
                    },
                    theme::accent(),
                );
            }

            renderer.draw_text(
                label,
                rect.x + pad + 8.0,
                item_y + 10.0,
                14.0,
                theme::text(),
            );
        }

        // Bottom controls
        let bottom_y = rect.y + rect.height - 120.0;
        renderer.draw_text("Theme", rect.x + pad, bottom_y, 11.0, theme::text_dim());
        renderer.draw_text(
            if self.theme_dark { "Dark" } else { "Light" },
            rect.x + pad,
            bottom_y + 18.0,
            13.0,
            theme::text(),
        );
        renderer.draw_text(
            &format!(
                "Reduce Motion: {}",
                if self.reduce_motion { "ON" } else { "OFF" }
            ),
            rect.x + pad,
            bottom_y + 50.0,
            11.0,
            theme::text_dim(),
        );
        renderer.draw_text(
            &format!(
                "Reduce Transparency: {}",
                if self.reduce_transparency {
                    "ON"
                } else {
                    "OFF"
                }
            ),
            rect.x + pad,
            bottom_y + 70.0,
            11.0,
            theme::text_dim(),
        );
        renderer.draw_text(
            &format!(
                "High Contrast: {}",
                if self.increase_contrast { "ON" } else { "OFF" }
            ),
            rect.x + pad,
            bottom_y + 90.0,
            11.0,
            theme::text_dim(),
        );

        renderer.pop_vnode();
    }

    fn render_content(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ShowcaseContent");

        let header_h: f32 = 56.0;
        let header_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: header_h,
        };
        renderer.fill_rect(header_rect, theme::surface());
        renderer.draw_line(
            header_rect.x,
            header_rect.y + header_rect.height,
            header_rect.x + header_rect.width,
            header_rect.y + header_rect.height,
            theme::border(),
            1.0,
        );

        let titles = [
            "Buttons",
            "Inputs",
            "Layout",
            "Overlays",
            "Visual",
            "Typography",
            "Forms",
            "Accessibility",
        ];
        let title = titles
            .get(self.current_section as usize)
            .unwrap_or(&"Unknown");
        renderer.draw_text(
            title,
            header_rect.x + 20.0,
            header_rect.y + 16.0,
            20.0,
            theme::text(),
        );
        renderer.draw_text(
            &format!("{:.0}%", self.zoom * 100.0),
            header_rect.x + header_rect.width - 80.0,
            header_rect.y + 18.0,
            12.0,
            theme::text_dim(),
        );

        let page_rect = Rect {
            x: rect.x,
            y: rect.y + header_h,
            width: rect.width,
            height: rect.height - header_h,
        };

        match self.current_section {
            0 => self.render_buttons_page(renderer, page_rect),
            1 => self.render_inputs_page(renderer, page_rect),
            2 => self.render_layout_page(renderer, page_rect),
            3 => self.render_overlays_page(renderer, page_rect),
            4 => self.render_visual_page(renderer, page_rect),
            5 => self.render_typography_page(renderer, page_rect),
            6 => self.render_forms_page(renderer, page_rect),
            7 => self.render_a11y_page(renderer, page_rect),
            _ => {}
        }

        renderer.pop_vnode();
    }

    fn render_buttons_page(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ButtonsPage");
        let pad = 24.0;
        let mut y = rect.y + pad;

        renderer.draw_text("Button Variants", rect.x + pad, y, 16.0, theme::text());
        y += 28.0;

        for label in &["Default", "Destructive", "Secondary", "Ghost", "Link"] {
            renderer.fill_rounded_rect(
                Rect {
                    x: rect.x + pad,
                    y,
                    width: 120.0,
                    height: 36.0,
                },
                6.0,
                theme::accent(),
            );
            renderer.draw_text(
                label,
                rect.x + pad + 16.0,
                y + 10.0,
                13.0,
                [1.0, 1.0, 1.0, 1.0],
            );
            renderer.draw_text(
                label,
                rect.x + pad + 130.0,
                y + 8.0,
                12.0,
                theme::text_dim(),
            );
            y += 44.0;
        }

        y += 16.0;
        renderer.draw_text("Progress & Activity", rect.x + pad, y, 16.0, theme::text());
        y += 28.0;

        // Linear progress
        let pw = rect.width - pad * 2.0;
        let track_rect = Rect {
            x: rect.x + pad,
            y,
            width: pw,
            height: 8.0,
        };
        renderer.fill_rounded_rect(track_rect, 4.0, theme::surface_elevated());
        renderer.fill_rounded_rect(
            Rect {
                x: track_rect.x,
                y: track_rect.y,
                width: track_rect.width * 0.65,
                height: track_rect.height,
            },
            4.0,
            theme::accent(),
        );
        renderer.draw_text(
            "65%",
            rect.x + pad + pw + 8.0,
            y - 4.0,
            11.0,
            theme::text_dim(),
        );
        y += 32.0;

        // Spinner
        renderer.stroke_ellipse(
            Rect {
                x: rect.x + pad,
                y,
                width: 24.0,
                height: 24.0,
            },
            [
                theme::accent()[0],
                theme::accent()[1],
                theme::accent()[2],
                0.2,
            ],
            2.0,
        );
        renderer.draw_text(
            "Loading...",
            rect.x + pad + 32.0,
            y + 4.0,
            12.0,
            theme::text_dim(),
        );
        y += 40.0;

        // Gauge
        renderer.stroke_ellipse(
            Rect {
                x: rect.x + pad,
                y,
                width: 80.0,
                height: 80.0,
            },
            theme::surface_elevated(),
            6.0,
        );
        renderer.draw_text("75", rect.x + pad + 28.0, y + 30.0, 16.0, theme::text());

        renderer.pop_vnode();
    }

    fn render_inputs_page(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.push_vnode(rect, "InputsPage");
        let pad = 24.0;
        let mut y = rect.y + pad;
        let field_w = (rect.width - pad * 2.0).min(500.0);
        let field_h: f32 = 40.0;

        renderer.draw_text("Text Input", rect.x + pad, y, 16.0, theme::text());
        y += 28.0;

        // Text field
        let tf_rect = Rect {
            x: rect.x + pad,
            y,
            width: field_w,
            height: field_h,
        };
        renderer.fill_rounded_rect(tf_rect, 6.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(tf_rect, 6.0, theme::border(), 1.0);
        renderer.draw_text(
            "Type here (full keyboard, clipboard, IME support)",
            rect.x + pad + 12.0,
            y + 10.0,
            14.0,
            theme::text_dim(),
        );
        y += field_h + 24.0;

        // Slider
        renderer.draw_text("Slider", rect.x + pad, y, 16.0, theme::text());
        y += 28.0;
        let slider_w = field_w;
        let track_rect = Rect {
            x: rect.x + pad,
            y: y + 11.0,
            width: slider_w,
            height: 6.0,
        };
        renderer.fill_rounded_rect(track_rect, 3.0, theme::surface_elevated());
        renderer.fill_rounded_rect(
            Rect {
                x: track_rect.x,
                y: track_rect.y,
                width: slider_w * 0.6,
                height: track_rect.height,
            },
            3.0,
            theme::accent(),
        );
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x + pad + slider_w * 0.6 - 10.0,
                y,
                width: 20.0,
                height: 20.0,
            },
            10.0,
            theme::accent(),
        );
        y += 44.0;

        // Toggle
        renderer.draw_text("Toggle / Switch", rect.x + pad, y, 16.0, theme::text());
        y += 28.0;
        // On toggle
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x + pad,
                y,
                width: 52.0,
                height: 28.0,
            },
            14.0,
            theme::accent(),
        );
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x + pad + 28.0,
                y: y + 2.0,
                width: 24.0,
                height: 24.0,
            },
            12.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        renderer.draw_text(
            "Dark Mode (ON)",
            rect.x + pad + 64.0,
            y + 4.0,
            12.0,
            theme::text_dim(),
        );
        y += 36.0;
        // Off toggle
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x + pad,
                y,
                width: 52.0,
                height: 28.0,
            },
            14.0,
            theme::surface_elevated(),
        );
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x + pad + 2.0,
                y: y + 2.0,
                width: 24.0,
                height: 24.0,
            },
            12.0,
            [0.7, 0.7, 0.7, 1.0],
        );
        renderer.draw_text(
            "Reduce Motion (OFF)",
            rect.x + pad + 64.0,
            y + 4.0,
            12.0,
            theme::text_dim(),
        );
        y += 44.0;

        // Stepper
        renderer.draw_text("Stepper", rect.x + pad, y, 16.0, theme::text());
        y += 28.0;
        let stepper_rect = Rect {
            x: rect.x + pad,
            y,
            width: 120.0,
            height: 32.0,
        };
        renderer.fill_rounded_rect(stepper_rect, 6.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(stepper_rect, 6.0, theme::border(), 1.0);
        renderer.draw_line(
            stepper_rect.x + 60.0,
            stepper_rect.y + 4.0,
            stepper_rect.x + 60.0,
            stepper_rect.y + 28.0,
            theme::border(),
            1.0,
        );
        renderer.draw_text("-", rect.x + pad + 14.0, y + 6.0, 16.0, theme::text());
        renderer.draw_text("+", rect.x + pad + 74.0, y + 6.0, 16.0, theme::text());
        renderer.draw_text("42", rect.x + pad + 132.0, y + 6.0, 14.0, theme::text());

        renderer.pop_vnode();
    }

    fn render_layout_page(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.push_vnode(rect, "LayoutPage");
        let pad = 24.0;
        let mut y = rect.y + pad;

        renderer.draw_text(
            "HStack - Horizontal Layout",
            rect.x + pad,
            y,
            16.0,
            theme::text(),
        );
        y += 28.0;
        let block_h: f32 = 40.0;
        let total_w = rect.width - pad * 2.0;
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x + pad,
                y,
                width: total_w * 0.5,
                height: block_h,
            },
            6.0,
            theme::accent(),
        );
        renderer.draw_text(
            "50%",
            rect.x + pad + total_w * 0.25 - 16.0,
            y + 10.0,
            14.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x + pad + total_w * 0.5 + 8.0,
                y,
                width: total_w * 0.3,
                height: block_h,
            },
            6.0,
            theme::warning(),
        );
        renderer.draw_text(
            "30%",
            rect.x + pad + total_w * 0.5 + 8.0 + total_w * 0.15 - 16.0,
            y + 10.0,
            14.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x + pad + total_w * 0.8 + 16.0,
                y,
                width: total_w * 0.2 - 16.0,
                height: block_h,
            },
            6.0,
            theme::error_color(),
        );
        renderer.draw_text(
            "20%",
            rect.x + pad + total_w * 0.8 + 16.0 + total_w * 0.1 - 16.0,
            y + 10.0,
            14.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        y += block_h + 32.0;

        renderer.draw_text("Grid - 3x3", rect.x + pad, y, 16.0, theme::text());
        y += 28.0;
        let cell_size: f32 = 60.0;
        let cell_gap: f32 = 8.0;
        for row in 0..3u32 {
            for col in 0..3u32 {
                let cx = rect.x + pad + col as f32 * (cell_size + cell_gap);
                let cy = y + row as f32 * (cell_size + cell_gap);
                renderer.fill_rounded_rect(
                    Rect {
                        x: cx,
                        y: cy,
                        width: cell_size,
                        height: cell_size,
                    },
                    6.0,
                    theme::surface_elevated(),
                );
                renderer.draw_text(
                    &format!("{},{}", row, col),
                    cx + 16.0,
                    cy + 20.0,
                    11.0,
                    theme::text_dim(),
                );
            }
        }

        renderer.pop_vnode();
    }

    fn render_overlays_page(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.push_vnode(rect, "OverlaysPage");
        let pad = 24.0;
        let mut y = rect.y + pad;

        renderer.draw_text("Dialog / Alert", rect.x + pad, y, 16.0, theme::text());
        y += 28.0;
        let dialog_w = rect.width - pad * 2.0;
        let dialog_h: f32 = 160.0;
        renderer.fill_rect(rect, [0.0, 0.0, 0.0, 0.5]);
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x + pad,
                y,
                width: dialog_w,
                height: dialog_h,
            },
            16.0,
            theme::surface(),
        );
        renderer.stroke_rounded_rect(
            Rect {
                x: rect.x + pad,
                y,
                width: dialog_w,
                height: dialog_h,
            },
            16.0,
            theme::border(),
            1.0,
        );
        renderer.draw_text(
            "Confirm Action",
            rect.x + pad + 20.0,
            y + 16.0,
            18.0,
            theme::text(),
        );
        renderer.draw_text(
            "This action cannot be undone. Are you sure?",
            rect.x + pad + 20.0,
            y + 48.0,
            13.0,
            theme::text_dim(),
        );
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x + pad + dialog_w - 200.0,
                y: y + dialog_h - 44.0,
                width: 80.0,
                height: 32.0,
            },
            6.0,
            theme::surface_elevated(),
        );
        renderer.draw_text(
            "Cancel",
            rect.x + pad + dialog_w - 180.0,
            y + dialog_h - 36.0,
            13.0,
            theme::text(),
        );
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x + pad + dialog_w - 108.0,
                y: y + dialog_h - 44.0,
                width: 88.0,
                height: 32.0,
            },
            6.0,
            theme::error_color(),
        );
        renderer.draw_text(
            "Delete",
            rect.x + pad + dialog_w - 88.0,
            y + dialog_h - 36.0,
            13.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        y += dialog_h + 32.0;

        renderer.draw_text("Toast (auto-dismiss)", rect.x + pad, y, 16.0, theme::text());
        y += 28.0;
        let toast_rect = Rect {
            x: rect.x + pad,
            y,
            width: 280.0,
            height: 64.0,
        };
        renderer.fill_rounded_rect(toast_rect, 8.0, theme::surface_elevated());
        renderer.fill_rounded_rect(
            Rect {
                x: toast_rect.x,
                y: toast_rect.y + 8.0,
                width: 4.0,
                height: toast_rect.height - 16.0,
            },
            2.0,
            theme::accent(),
        );
        renderer.draw_text(
            "Success",
            rect.x + pad + 16.0,
            y + 10.0,
            14.0,
            theme::text(),
        );
        renderer.draw_text(
            "File saved successfully.",
            rect.x + pad + 16.0,
            y + 32.0,
            12.0,
            theme::text_dim(),
        );

        renderer.pop_vnode();
    }

    fn render_visual_page(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.push_vnode(rect, "VisualPage");
        let pad = 24.0;
        let mut y = rect.y + pad;

        renderer.draw_text("Progress Bars", rect.x + pad, y, 16.0, theme::text());
        y += 28.0;

        for (pct, label) in &[(0.25, "25%"), (0.50, "50%"), (0.75, "75%"), (1.0, "100%")] {
            let pw = rect.width - pad * 2.0;
            let track_rect = Rect {
                x: rect.x + pad,
                y,
                width: pw,
                height: 8.0,
            };
            renderer.fill_rounded_rect(track_rect, 4.0, theme::surface_elevated());
            renderer.fill_rounded_rect(
                Rect {
                    x: track_rect.x,
                    y: track_rect.y,
                    width: track_rect.width * pct,
                    height: track_rect.height,
                },
                4.0,
                theme::accent(),
            );
            renderer.draw_text(
                label,
                rect.x + pad + pw + 8.0,
                y - 4.0,
                11.0,
                theme::text_dim(),
            );
            y += 24.0;
        }
        y += 16.0;

        renderer.draw_text("Spinners", rect.x + pad, y, 16.0, theme::text());
        y += 28.0;
        renderer.stroke_ellipse(
            Rect {
                x: rect.x + pad,
                y,
                width: 24.0,
                height: 24.0,
            },
            [
                theme::accent()[0],
                theme::accent()[1],
                theme::accent()[2],
                0.2,
            ],
            2.0,
        );
        renderer.draw_text(
            "Loading...",
            rect.x + pad + 32.0,
            y + 4.0,
            12.0,
            theme::text_dim(),
        );
        y += 40.0;

        renderer.draw_text("Gauges", rect.x + pad, y, 16.0, theme::text());
        y += 28.0;
        for pct in &[0.33, 0.66, 0.9] {
            renderer.stroke_ellipse(
                Rect {
                    x: rect.x + pad,
                    y,
                    width: 64.0,
                    height: 64.0,
                },
                theme::surface_elevated(),
                5.0,
            );
            renderer.draw_text(
                &format!("{:.0}%", pct * 100.0),
                rect.x + pad + 16.0,
                y + 22.0,
                12.0,
                theme::text(),
            );
            y += 76.0;
        }

        renderer.pop_vnode();
    }

    fn render_typography_page(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.push_vnode(rect, "TypographyPage");
        let pad = 24.0;
        let mut y = rect.y + pad;

        renderer.draw_text("Type Scale", rect.x + pad, y, 16.0, theme::text());
        y += 32.0;

        let sizes = [
            ("Display", 48.0f32),
            ("H1", 32.0),
            ("H2", 24.0),
            ("H3", 20.0),
            ("Body", 16.0),
            ("Caption", 12.0),
            ("Footnote", 10.0),
        ];

        for (name, size) in &sizes {
            renderer.draw_text(name, rect.x + pad, y, 11.0, theme::text_dim());
            renderer.draw_text(
                &format!("{} - The quick brown fox", name),
                rect.x + pad + 112.0,
                y,
                *size,
                theme::text(),
            );
            y += size + 12.0;
        }

        renderer.pop_vnode();
    }

    fn render_forms_page(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.push_vnode(rect, "FormsPage");
        let pad = 24.0;
        let mut y = rect.y + pad;
        let field_w = (rect.width - pad * 2.0).min(500.0);
        let field_h: f32 = 40.0;
        let label_w: f32 = 120.0;

        renderer.draw_text(
            "Complete Form Example",
            rect.x + pad,
            y,
            16.0,
            theme::text(),
        );
        y += 32.0;

        // Name
        renderer.draw_text("Name", rect.x + pad, y + 8.0, 13.0, theme::text_dim());
        let fr = Rect {
            x: rect.x + pad + label_w,
            y,
            width: field_w - label_w,
            height: field_h,
        };
        renderer.fill_rounded_rect(fr, 6.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(fr, 6.0, theme::border(), 1.0);
        renderer.draw_text(
            "Enter your name",
            rect.x + pad + label_w + 12.0,
            y + 10.0,
            14.0,
            theme::text_dim(),
        );
        y += 52.0;

        // Email
        renderer.draw_text("Email", rect.x + pad, y + 8.0, 13.0, theme::text_dim());
        let fr = Rect {
            x: rect.x + pad + label_w,
            y,
            width: field_w - label_w,
            height: field_h,
        };
        renderer.fill_rounded_rect(fr, 6.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(fr, 6.0, theme::border(), 1.0);
        renderer.draw_text(
            "user@example.com",
            rect.x + pad + label_w + 12.0,
            y + 10.0,
            14.0,
            theme::text(),
        );
        y += 52.0;

        // Toggle
        renderer.draw_text(
            "Notifications",
            rect.x + pad,
            y + 4.0,
            13.0,
            theme::text_dim(),
        );
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x + pad + label_w,
                y,
                width: 52.0,
                height: 28.0,
            },
            14.0,
            theme::accent(),
        );
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x + pad + label_w + 28.0,
                y: y + 2.0,
                width: 24.0,
                height: 24.0,
            },
            12.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        y += 44.0;

        // Submit
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x + pad + label_w,
                y,
                width: 120.0,
                height: 40.0,
            },
            8.0,
            theme::accent(),
        );
        renderer.draw_text(
            "Submit",
            rect.x + pad + label_w + 36.0,
            y + 10.0,
            14.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        renderer.pop_vnode();
    }

    fn render_a11y_page(&self, renderer: &mut dyn cvkg_core::Renderer, rect: Rect) {
        renderer.push_vnode(rect, "A11yPage");
        let pad = 24.0;
        let mut y = rect.y + pad;

        renderer.draw_text(
            "Accessibility Preferences",
            rect.x + pad,
            y,
            16.0,
            theme::text(),
        );
        y += 32.0;

        let prefs = [
            (
                "Reduce Motion",
                "Disables or simplifies animations system-wide",
                self.reduce_motion,
            ),
            (
                "Reduce Transparency",
                "Replaces glass/blur effects with opaque surfaces",
                self.reduce_transparency,
            ),
            (
                "Increase Contrast",
                "Makes borders more visible, removes pure-transparency elements",
                self.increase_contrast,
            ),
        ];

        for (name, desc, enabled) in &prefs {
            let toggle_rect = Rect {
                x: rect.x + pad,
                y,
                width: 52.0,
                height: 28.0,
            };
            if *enabled {
                renderer.fill_rounded_rect(toggle_rect, 14.0, theme::accent());
                renderer.fill_rounded_rect(
                    Rect {
                        x: rect.x + pad + 28.0,
                        y: y + 2.0,
                        width: 24.0,
                        height: 24.0,
                    },
                    12.0,
                    [1.0, 1.0, 1.0, 1.0],
                );
            } else {
                renderer.fill_rounded_rect(toggle_rect, 14.0, theme::surface_elevated());
                renderer.fill_rounded_rect(
                    Rect {
                        x: rect.x + pad + 2.0,
                        y: y + 2.0,
                        width: 24.0,
                        height: 24.0,
                    },
                    12.0,
                    [0.7, 0.7, 0.7, 1.0],
                );
            }
            renderer.draw_text(name, rect.x + pad + 64.0, y + 4.0, 14.0, theme::text());
            y += 28.0;
            renderer.draw_text(desc, rect.x + pad + 64.0, y, 11.0, theme::text_dim());
            y += 36.0;
        }

        y += 24.0;
        renderer.draw_text("Keyboard Navigation", rect.x + pad, y, 16.0, theme::text());
        y += 28.0;
        renderer.draw_text(
            "Tab / Shift+Tab - Navigate between controls",
            rect.x + pad,
            y,
            12.0,
            theme::text_dim(),
        );
        y += 22.0;
        renderer.draw_text(
            "Arrow keys - Navigate within compound controls",
            rect.x + pad,
            y,
            12.0,
            theme::text_dim(),
        );
        y += 22.0;
        renderer.draw_text(
            "Space / Enter - Activate focused control",
            rect.x + pad,
            y,
            12.0,
            theme::text_dim(),
        );
        y += 22.0;
        renderer.draw_text(
            "Escape - Dismiss overlays and popovers",
            rect.x + pad,
            y,
            12.0,
            theme::text_dim(),
        );
        y += 22.0;
        renderer.draw_text(
            "Cmd/Ctrl+Z - Undo  |  Cmd/Ctrl+Shift+Z - Redo",
            rect.x + pad,
            y,
            12.0,
            theme::text_dim(),
        );

        renderer.pop_vnode();
    }
}

fn main() {
    env_logger::init();
    log::info!("[Showcase] Starting CVKG Component Showcase");
    let _app = ShowcaseApp::new();
    log::info!("[Showcase] Showcase app ready");
}
