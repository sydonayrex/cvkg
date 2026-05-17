#![allow(
    dead_code,
    unused_imports,
    clippy::new_without_default,
    clippy::type_complexity
)]

use cvkg_components::{Badge, BadgeVariant, Button};
use cvkg_core::{Never, Rect, Renderer, View};
use wasm_bindgen::prelude::*;

mod data;
use data::{DesignSystem, FilterCriteria};

const DATA_JSON: &str = include_str!("data.json");

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    Ok(())
}

#[derive(Clone, PartialEq)]
pub enum ViewMode {
    Catalog,
    Detail(String), // system data as ID
    Comparison,
}

pub struct AdeleApp {
    systems: Vec<DesignSystem>,
    criteria: FilterCriteria,
    view_mode: ViewMode,
    selected_systems: Vec<String>, // system data as IDs
}

impl AdeleApp {
    pub fn new() -> Self {
        let systems: Vec<DesignSystem> = serde_json::from_str(DATA_JSON).unwrap_or_else(|e| {
            log::error!("Failed to parse data.json: {}", e);
            Vec::new()
        });

        Self {
            systems,
            criteria: FilterCriteria::default(),
            view_mode: ViewMode::Catalog,
            selected_systems: Vec::new(),
        }
    }
}

impl View for AdeleApp {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // App background (glassmorphic dark)
        renderer.fill_rect(rect, [0.01, 0.01, 0.03, 1.0]);

        let header_h = 70.0;
        let sidebar_w = 260.0;

        // 1. Header
        let header_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: header_h,
        };
        self.render_header(renderer, header_rect);

        match &self.view_mode {
            ViewMode::Catalog => {
                // 2. Sidebar (Filters)
                let sidebar_rect = Rect {
                    x: rect.x,
                    y: rect.y + header_h,
                    width: sidebar_w,
                    height: rect.height - header_h,
                };
                self.render_sidebar(renderer, sidebar_rect);

                // 3. Main Content (Grid)
                let main_rect = Rect {
                    x: rect.x + sidebar_w,
                    y: rect.y + header_h,
                    width: rect.width - sidebar_w,
                    height: rect.height - header_h,
                };
                self.render_grid(renderer, main_rect);
            }
            ViewMode::Detail(id) => {
                let detail_rect = Rect {
                    x: rect.x + 40.0,
                    y: rect.y + header_h + 40.0,
                    width: rect.width - 80.0,
                    height: rect.height - header_h - 80.0,
                };
                if let Some(system) = self.systems.iter().find(|s| s.system.data == *id) {
                    self.render_detail_view(renderer, detail_rect, system);
                }
            }
            ViewMode::Comparison => {
                let comp_rect = Rect {
                    x: rect.x + 40.0,
                    y: rect.y + header_h + 40.0,
                    width: rect.width - 80.0,
                    height: rect.height - header_h - 80.0,
                };
                self.render_comparison_view(renderer, comp_rect);
            }
        }
    }
}

impl AdeleApp {
    fn render_header(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.bifrost(rect, 20.0, 1.2, 0.8);
        renderer.fill_rect(rect, [0.05, 0.05, 0.12, 0.6]);
        renderer.draw_line(
            rect.x,
            rect.y + rect.height,
            rect.x + rect.width,
            rect.y + rect.height,
            [0.0, 0.8, 1.0, 0.3],
            1.5,
        );

        let title = "ADELE";
        renderer.draw_text(
            title,
            rect.x + 30.0,
            rect.y + 25.0,
            28.0,
            [0.0, 1.0, 1.0, 1.0],
        );
        renderer.draw_text(
            "DESIGN SYSTEMS EXPLORER",
            rect.x + 120.0,
            rect.y + 32.0,
            12.0,
            [0.5, 0.5, 0.6, 1.0],
        );

        // Navigation
        let mut x = rect.x + 350.0;
        let navs = [
            ("Catalog", ViewMode::Catalog),
            ("Comparison", ViewMode::Comparison),
        ];
        for (label, mode) in navs {
            let is_active = self.view_mode == mode;
            renderer.draw_text(
                label,
                x,
                rect.y + 30.0,
                14.0,
                if is_active {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    [0.5, 0.5, 0.5, 1.0]
                },
            );
            if is_active {
                renderer.draw_line(
                    x,
                    rect.y + 50.0,
                    x + 50.0,
                    rect.y + 50.0,
                    [0.0, 0.8, 1.0, 1.0],
                    2.0,
                );
            }
            x += 100.0;
        }

        let stats = format!(
            "{} SYSTEMS",
            self.systems
                .iter()
                .filter(|s| self.criteria.matches(s))
                .count()
        );
        let (sw, _) = renderer.measure_text(&stats, 12.0);
        renderer.draw_text(
            &stats,
            rect.x + rect.width - sw - 30.0,
            rect.y + 30.0,
            12.0,
            [0.0, 0.8, 1.0, 1.0],
        );
    }

    fn render_sidebar(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rect(rect, [0.02, 0.02, 0.05, 0.7]);
        renderer.draw_line(
            rect.x + rect.width,
            rect.y,
            rect.x + rect.width,
            rect.y + rect.height,
            [0.2, 0.2, 0.3, 0.5],
            1.0,
        );

        renderer.draw_text(
            "FILTERS",
            rect.x + 25.0,
            rect.y + 30.0,
            10.0,
            [0.4, 0.4, 0.5, 1.0],
        );

        // Search Input
        let search_rect = Rect {
            x: rect.x + 15.0,
            y: rect.y + 60.0,
            width: rect.width - 30.0,
            height: 40.0,
        };
        renderer.fill_rounded_rect(search_rect, 4.0, [0.1, 0.1, 0.15, 1.0]);
        renderer.stroke_rounded_rect(search_rect, 4.0, [0.2, 0.2, 0.3, 1.0], 1.0);
        renderer.draw_text(
            if self.criteria.search_query.is_empty() {
                "Search systems..."
            } else {
                &self.criteria.search_query
            },
            search_rect.x + 12.0,
            search_rect.y + 12.0,
            14.0,
            [0.5, 0.5, 0.6, 1.0],
        );

        // Filter Categories
        let mut y = rect.y + 130.0;
        let categories = [
            (
                "TECHNOLOGY",
                vec!["React", "Vue", "TypeScript", "Web Components"],
            ),
            ("COMPONENTS", vec!["UI Kit", "Design Tokens", "Storybook"]),
            ("DESIGN", vec!["Accessibility", "Animation", "Typography"]),
        ];

        for (cat, options) in categories {
            renderer.draw_text(cat, rect.x + 25.0, y, 10.0, [0.4, 0.4, 0.5, 1.0]);
            y += 25.0;
            for opt in options {
                renderer.draw_text(opt, rect.x + 35.0, y, 13.0, [0.8, 0.8, 0.9, 1.0]);
                y += 24.0;
            }
            y += 20.0;
        }
    }

    fn render_grid(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let filtered: Vec<&DesignSystem> = self
            .systems
            .iter()
            .filter(|s| self.criteria.matches(s))
            .collect();

        let card_w = 320.0;
        let card_h = 180.0;
        let spacing = 24.0;

        let cols = ((rect.width - spacing) / (card_w + spacing)).floor() as usize;
        let cols = cols.max(1);

        for (i, system) in filtered.iter().enumerate().take(30) {
            let row = i / cols;
            let col = i % cols;

            let card_rect = Rect {
                x: rect.x + spacing + col as f32 * (card_w + spacing),
                y: rect.y + spacing + row as f32 * (card_h + spacing),
                width: card_w,
                height: card_h,
            };

            self.render_system_card(renderer, card_rect, system);
        }
    }

    fn render_system_card(&self, renderer: &mut dyn Renderer, rect: Rect, system: &DesignSystem) {
        renderer.bifrost(rect, 10.0, 1.1, 0.9);
        renderer.fill_rounded_rect(rect, 8.0, [0.08, 0.08, 0.12, 0.5]);
        renderer.stroke_rounded_rect(rect, 8.0, [0.2, 0.3, 0.4, 0.4], 1.0);

        // Header
        renderer.draw_text(
            &system.system.data.to_uppercase(),
            rect.x + 20.0,
            rect.y + 20.0,
            18.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        renderer.draw_text(
            &system.company.data,
            rect.x + 20.0,
            rect.y + 45.0,
            14.0,
            [0.0, 0.8, 1.0, 1.0],
        );

        // Tech Badges
        let mut badge_x = rect.x + 20.0;
        let badge_y = rect.y + rect.height - 40.0;

        if system.ts.data.to_lowercase() == "yes" {
            let ts_badge = Badge::new("TS").variant(BadgeVariant::Secondary);
            ts_badge.render(
                renderer,
                Rect {
                    x: badge_x,
                    y: badge_y,
                    width: 35.0,
                    height: 22.0,
                },
            );
            badge_x += 42.0;
        }

        let js = system.js.data.to_lowercase();
        if js.contains("react") {
            let react_badge = Badge::new("React").variant(BadgeVariant::Outline);
            react_badge.render(
                renderer,
                Rect {
                    x: badge_x,
                    y: badge_y,
                    width: 55.0,
                    height: 22.0,
                },
            );
            badge_x += 62.0;
        } else if js.contains("vue") {
            let vue_badge = Badge::new("Vue").variant(BadgeVariant::Outline);
            vue_badge.render(
                renderer,
                Rect {
                    x: badge_x,
                    y: badge_y,
                    width: 45.0,
                    height: 22.0,
                },
            );
            badge_x += 52.0;
        }

        if system.accessibility_guidelines.data.to_lowercase() == "yes" {
            let a11y_badge = Badge::new("A11Y").variant(BadgeVariant::Default);
            a11y_badge.render(
                renderer,
                Rect {
                    x: badge_x,
                    y: badge_y,
                    width: 45.0,
                    height: 22.0,
                },
            );
        }
    }

    fn render_detail_view(&self, renderer: &mut dyn Renderer, rect: Rect, system: &DesignSystem) {
        renderer.bifrost(rect, 30.0, 1.5, 0.9);
        renderer.fill_rounded_rect(rect, 12.0, [0.05, 0.05, 0.1, 0.95]);
        renderer.stroke_rounded_rect(rect, 12.0, [0.0, 0.8, 1.0, 0.6], 2.0);

        renderer.draw_text(
            &system.system.data.to_uppercase(),
            rect.x + 40.0,
            rect.y + 40.0,
            32.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        renderer.draw_text(
            &system.company.data,
            rect.x + 40.0,
            rect.y + 80.0,
            18.0,
            [0.0, 0.8, 1.0, 1.0],
        );

        let mut y = rect.y + 140.0;
        let attributes = [
            ("Code Depth", &system.code_depth.data),
            ("JS Framework", &system.js.data),
            ("TypeScript", &system.ts.data),
            ("CSS Approach", &system.css.data),
            ("Components", &system.components.data),
            ("Design Tokens", &system.design_tokens.data),
            ("Storybook", &system.storybook.data),
            ("Accessibility", &system.accessibility_guidelines.data),
        ];

        for (label, value) in attributes {
            renderer.draw_text(label, rect.x + 40.0, y, 14.0, [0.5, 0.5, 0.6, 1.0]);
            renderer.draw_text(value, rect.x + 200.0, y, 14.0, [1.0, 1.0, 1.0, 1.0]);
            y += 30.0;
        }

        // Action Buttons
        let mut btn_x = rect.x + 40.0;
        let btn_y = rect.y + rect.height - 80.0;

        let repo_btn = Button::new("View Repository", || {});
        repo_btn.render(
            renderer,
            Rect {
                x: btn_x,
                y: btn_y,
                width: 140.0,
                height: 40.0,
            },
        );
        btn_x += 160.0;

        let compare_btn = Button::new("Add to Comparison", || {});
        compare_btn.render(
            renderer,
            Rect {
                x: btn_x,
                y: btn_y,
                width: 160.0,
                height: 40.0,
            },
        );
    }

    fn render_comparison_view(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.bifrost(rect, 20.0, 1.2, 0.9);
        renderer.fill_rounded_rect(rect, 8.0, [0.05, 0.05, 0.1, 0.8]);

        let systems_to_compare: Vec<&DesignSystem> = self
            .systems
            .iter()
            .filter(|s| self.selected_systems.contains(&s.system.data))
            .collect();

        if systems_to_compare.is_empty() {
            renderer.draw_text(
                "No systems selected for comparison.",
                rect.x + 40.0,
                rect.y + 40.0,
                16.0,
                [0.5, 0.5, 0.6, 1.0],
            );
            renderer.draw_text(
                "Go back to catalog and add systems to compare them side-by-side.",
                rect.x + 40.0,
                rect.y + 70.0,
                14.0,
                [0.4, 0.4, 0.5, 1.0],
            );
            return;
        }

        // Matrix header
        let col_w = (rect.width - 200.0) / systems_to_compare.len() as f32;
        let mut x = rect.x + 200.0;
        for system in &systems_to_compare {
            renderer.draw_text(
                &system.system.data.to_uppercase(),
                x + 10.0,
                rect.y + 40.0,
                14.0,
                [0.0, 0.8, 1.0, 1.0],
            );
            x += col_w;
        }

        // Matrix rows
        let rows: Vec<(&str, Box<dyn Fn(&DesignSystem) -> &str>)> = vec![
            ("JS", Box::new(|s: &DesignSystem| &s.js.data)),
            ("TS", Box::new(|s: &DesignSystem| &s.ts.data)),
            ("CSS", Box::new(|s: &DesignSystem| &s.css.data)),
            (
                "A11Y",
                Box::new(|s: &DesignSystem| &s.accessibility_guidelines.data),
            ),
        ];

        let mut y = rect.y + 80.0;
        for (label, getter) in &rows {
            renderer.draw_text(label, rect.x + 40.0, y, 14.0, [0.5, 0.5, 0.6, 1.0]);
            let mut row_x = rect.x + 200.0;
            for system in &systems_to_compare {
                renderer.draw_text(getter(system), row_x + 10.0, y, 14.0, [1.0, 1.0, 1.0, 1.0]);
                row_x += col_w;
            }
            y += 40.0;
            renderer.draw_line(
                rect.x + 20.0,
                y - 15.0,
                rect.x + rect.width - 20.0,
                y - 15.0,
                [0.2, 0.2, 0.3, 0.3],
                1.0,
            );
        }
    }
}
