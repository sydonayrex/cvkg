//! API Docs tab — shows component props, usage examples, and copy-to-clipboard.

use crate::state::{GalleryState, Registry, ComponentMeta};
use cvkg_components::{Button, Codeblock, CopyButton, ScrollView, Text, VStack, HStack, Divider, Badge, BadgeVariant};
use cvkg_core::{Rect, Renderer, View, Size, SizeProposal};
use std::sync::{Arc, Mutex};

/// API Documentation panel — shows props table, usage examples, and copy buttons.
#[derive(Clone)]
pub struct DocsPanel {
    state: Arc<Mutex<GalleryState>>,
}

impl DocsPanel {
    pub fn new(state: Arc<Mutex<GalleryState>>) -> Self {
        Self { state }
    }

    fn get_selected_meta(&self) -> Option<ComponentMeta> {
        let state = self.state.lock().unwrap();
        state.selected_component.as_ref().and_then(|name| Registry::find(name))
    }
}

impl View for DocsPanel {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let meta = self.get_selected_meta();

        // Panel background
        renderer.fill_rect(rect, [0.98, 0.98, 0.98, 1.0]);
        renderer.draw_line(rect.x, rect.y, rect.x + rect.width, rect.y, [0.7, 0.7, 0.7, 1.0], 1.0);

        let mut y = rect.y + 16.0;
        let pad = 16.0;
        let content_w = rect.width - pad * 2.0;

        if let Some(meta) = meta {
            // Title
            renderer.draw_text_raw(
                &format!("API: {}", meta.name),
                rect.x + pad,
                y,
                18.0,
                [0.15, 0.15, 0.15, 1.0],
            );
            y += 28.0;

            // Description
            renderer.draw_text_raw(
                meta.description,
                rect.x + pad,
                y,
                13.0,
                [0.4, 0.4, 0.4, 1.0],
            );
            y += 24.0;

            // Divider
            renderer.draw_line(rect.x + pad, y, rect.x + rect.width - pad, y, [0.85, 0.85, 0.85, 1.0], 1.0);
            y += 16.0;

            // Category badge
            let badge = Badge::new(meta.category).variant(BadgeVariant::Secondary);
            badge.render(renderer, Rect { x: rect.x + pad, y, width: 100.0, height: 22.0 });
            y += 30.0;

            // Props table
            renderer.draw_text_raw("Props", rect.x + pad, y, 14.0, [0.2, 0.2, 0.2, 1.0]);
            y += 22.0;

            let props = Self::get_props_for_component(&meta.name);
            for (prop_name, prop_type, default, desc) in props {
                // Prop row background
                let row_h = 32.0;
                let row_rect = Rect { x: rect.x + pad, y, width: content_w, height: row_h };
                renderer.fill_rect(row_rect, [0.95, 0.95, 0.95, 1.0]);
                renderer.draw_line(rect.x + pad, y + row_h, rect.x + rect.width - pad, y + row_h, [0.85, 0.85, 0.85, 1.0], 1.0);

                // Name + type badge
                renderer.draw_text_raw(prop_name, rect.x + pad + 8.0, y + 8.0, 12.0, [0.2, 0.2, 0.2, 1.0]);
                let type_badge = Badge::new(prop_type).variant(BadgeVariant::Outline);
                type_badge.render(renderer, Rect { x: rect.x + pad + 140.0, y: y + 4.0, width: 80.0, height: 22.0 });

                // Default
                if !default.is_empty() {
                    renderer.draw_text_raw(&format!("default: {}", default), rect.x + pad + 230.0, y + 8.0, 11.0, [0.5, 0.5, 0.5, 1.0]);
                }

                // Description
                if !desc.is_empty() {
                    renderer.draw_text_raw(desc, rect.x + pad + 8.0, y + row_h - 14.0, 10.0, [0.6, 0.6, 0.6, 1.0]);
                }

                y += row_h + 8.0;
            }

            y += 20.0;

            // Divider
            renderer.draw_line(rect.x + pad, y, rect.x + rect.width - pad, y, [0.85, 0.85, 0.85, 1.0], 1.0);
            y += 16.0;

            // Usage Example
            renderer.draw_text_raw("Usage Example", rect.x + pad, y, 14.0, [0.2, 0.2, 0.2, 1.0]);
            y += 22.0;

            let example = Self::get_usage_example(&meta.name);
            let code_block = Codeblock::new(example).language("rust");
            let code_rect = Rect { x: rect.x + pad, y, width: content_w, height: 160.0 };
            code_block.render(renderer, code_rect);
            y += 170.0;

            // Copy to clipboard buttons
            renderer.draw_text_raw("Copy to Clipboard", rect.x + pad, y, 13.0, [0.2, 0.2, 0.2, 1.0]);
            y += 20.0;

            let copy_usage = CopyButton::new("Copy Usage", || {});
            copy_usage.render(renderer, Rect { x: rect.x + pad, y, width: 140.0, height: 32.0 });

            let copy_props = CopyButton::new("Copy Props Table", || {});
            copy_props.render(renderer, Rect { x: rect.x + pad + 150.0, y, width: 150.0, height: 32.0 });

        } else {
            // Empty state
            renderer.draw_text_raw(
                "Select a component to view API docs",
                rect.x + pad,
                rect.y + rect.height / 2.0,
                14.0,
                [0.5, 0.5, 0.5, 1.0],
            );
        }
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size { width: proposal.width.unwrap_or(300.0), height: proposal.height.unwrap_or(600.0) }
    }
}

impl cvkg_core::layout::LayoutView for DocsPanel {
    fn size_that_fits(&self, proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: proposal.width.unwrap_or(300.0), height: proposal.height.unwrap_or(600.0) }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

impl DocsPanel {
    fn get_props_for_component(name: &str) -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
        match name {
            "Button" => vec![
                ("label", "string", "\"Click Me\"", "Button text"),
                ("on_click", "fn()", "|| {}", "Click handler"),
                ("variant", "enum", "Primary", "Primary|Secondary|Destructive|Ghost"),
                ("disabled", "bool", "false", "Disable interaction"),
                ("world", "WorldSpaceConfig", "default", "3D world-space config"),
            ],
            "Input" => vec![
                ("placeholder", "string", "\"Enter text...\"", "Placeholder text"),
                ("value", "string", "\"\"", "Current value"),
                ("on_change", "fn(String)", "|s| {}", "Change handler"),
                ("disabled", "bool", "false", "Disable input"),
            ],
            "Toggle" => vec![
                ("label", "string", "\"Enable\"", "Label text"),
                ("is_on", "bool", "false", "Checked state"),
                ("on_change", "fn(bool)", "|b| {}", "Change handler"),
                ("disabled", "bool", "false", "Disable toggle"),
            ],
            "Slider" => vec![
                ("value", "f32", "0.5", "Current value"),
                ("range", "RangeInclusive<f32>", "0.0..=1.0", "Min/max"),
                ("step", "Option<f32>", "None", "Step increment"),
                ("on_change", "fn(f32)", "|v| {}", "Change handler"),
            ],
            "Select" => vec![
                ("options", "Vec<(String, V)>", "vec![...]", "Display/value pairs"),
                ("selected", "usize", "0", "Selected index"),
                ("placeholder", "string", "\"Choose...\"", "Placeholder"),
                ("on_change", "fn(V)", "|v| {}", "Selection handler"),
            ],
            _ => vec![],
        }
    }

    fn get_usage_example(name: &str) -> &'static str {
        match name {
            "Button" => r#"use cvkg_components::Button;

let btn = Button::new("Click Me", || {
    println!("Clicked!");
})
.variant(ButtonVariant::Primary)
.disabled(false);"#,
            "Input" => r#"use cvkg_components::Input;

let input = Input::new("Type here...")
    .placeholder("Enter text...")
    .on_change(|text| println!("Changed: {}", text));"#,
            "Toggle" => r#"use cvkg_components::Toggle;

let toggle = Toggle::new("Enable Feature", false, |enabled| {
    println!("Feature: {}", enabled);
});"#,
            "Slider" => r#"use cvkg_components::Slider;

let slider = Slider::new(0.5, 0.0..=100.0, |val| {
    println!("Value: {}", val);
})
.step(1.0);"#,
            "Select" => r#"use cvkg_components::Select;

let options = vec![
    ("Option A".to_string(), "a"),
    ("Option B".to_string(), "b"),
];
let select = Select::new(options, 0)
    .placeholder("Choose...");"#,
            _ => "// No example available",
        }
    }
}