//! Props panel — type-aware editors for live component manipulation.

use crate::state::{GalleryState, Registry, ComponentMeta};
use cvkg_components::{Button, CodeEditor, Divider, HStack, Input, Select, Slider, Text, Toggle, VStack};
use cvkg_core::{Rect, Renderer, View, Size, SizeProposal, AnyView};
use std::sync::{Arc, Mutex};

/// Props panel showing editable properties for the selected component.
#[derive(Clone)]
pub struct PropsPanel {
    state: Arc<Mutex<GalleryState>>,
}

impl PropsPanel {
    pub fn new(state: Arc<Mutex<GalleryState>>) -> Self {
        Self { state }
    }

    fn get_selected_meta(&self) -> Option<ComponentMeta> {
        let state = self.state.lock().unwrap();
        state.selected_component.as_ref().and_then(|name| Registry::find(name))
    }

    fn get_prop_value(&self, prop_name: &str) -> Option<String> {
        let state = self.state.lock().unwrap();
        state.selected_component.as_ref().and_then(|name| {
            Registry::find(name).and_then(|meta| {
                // For now, return placeholder - in Phase 5 this would be auto-generated
                Self::get_manual_prop(name, prop_name)
            })
        })
    }

    fn set_prop_value(&self, prop_name: &str, value: String) {
        let state = self.state.lock().unwrap();
        if let Some(name) = state.selected_component.as_ref() {
            self.set_manual_prop(name, prop_name, value);
        }
    }

    // Manual property getters/setters - will be replaced by macro-generated code in Phase 5
    fn get_manual_prop(name: &str, prop: &str) -> Option<String> {
        match (name, prop) {
            ("Button", "label") => Some("Click Me".into()),
            ("Button", "disabled") => Some("false".into()),
            ("Input", "placeholder") => Some("Enter text...".into()),
            ("Input", "value") => Some("".into()),
            ("Toggle", "label") => Some("Enable".into()),
            ("Toggle", "is_on") => Some("false".into()),
            ("Slider", "min") => Some("0".into()),
            ("Slider", "max") => Some("100".into()),
            ("Slider", "value") => Some("50".into()),
            ("Select", "options") => Some("Option A,Option B,Option C".into()),
            ("Select", "selected") => Some("0".into()),
            _ => None,
        }
    }

    fn set_manual_prop(&self, name: &str, prop: &str, value: String) {
        // In Phase 5, this will update actual component properties via reflection
        // For now, just log the change
        let mut state = self.state.lock().unwrap();
        state.log_event(&format!("Property '{}.{}' = {}", name, prop, value));
    }
}

impl View for PropsPanel {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let meta = self.get_selected_meta();

        // Panel background
        renderer.fill_rect(rect, [0.98, 0.98, 0.98, 1.0]);
        renderer.draw_line(rect.x, rect.y, rect.x + rect.width, rect.y, [0.7, 0.7, 0.7, 1.0], 1.0);

        let mut y = rect.y + 16.0;
        let pad = 16.0;
        let field_h = 36.0;
        let label_w = 100.0;

        if let Some(meta) = meta {
            // Title
            renderer.draw_text_raw(
                &format!("Properties: {}", meta.name),
                rect.x + pad,
                y,
                14.0,
                [0.2, 0.2, 0.2, 1.0],
            );
            y += 28.0;

            // Description
            renderer.draw_text_raw(
                meta.description,
                rect.x + pad,
                y,
                11.0,
                [0.5, 0.5, 0.5, 1.0],
            );
            y += 24.0;

            // Divider
            renderer.draw_line(rect.x + pad, y, rect.x + rect.width - pad, y, [0.7, 0.7, 0.7, 1.0], 1.0);
            y += 16.0;

            // Properties for this component type
            let props = Self::get_props_for_component(&meta.name);
            for (prop_name, prop_type, default) in props {
                let val = self.get_prop_value(prop_name).unwrap_or(default.into());

                // Label
                renderer.draw_text_raw(
                    prop_name,
                    rect.x + pad,
                    y + 8.0,
                    12.0,
                    [0.3, 0.3, 0.3, 1.0],
                );

                // Type badge
                renderer.draw_text_raw(
                    prop_type,
                    rect.x + rect.width - pad - 50.0,
                    y + 8.0,
                    10.0,
                    [0.5, 0.5, 0.5, 1.0],
                );

                // Editor based on type
                let editor_x = rect.x + pad + label_w;
                let editor_w = rect.width - pad * 2.0 - label_w - 60.0;
                let editor_rect = Rect { x: editor_x, y: y, width: editor_w, height: field_h };

                match prop_type {
                    "string" => {
                        let input = Input::new(default);
                        input.render(renderer, editor_rect);
                    }
                    "bool" => {
                        let is_on = val == "true";
                        let toggle = Toggle::new(prop_name, is_on, |_| {});
                        toggle.render(renderer, editor_rect);
                    }
                    "number" => {
                        let parts: Vec<&str> = default.split(',').collect();
                        let min = parts.get(0).unwrap_or(&"0").parse().unwrap_or(0.0);
                        let max = parts.get(1).unwrap_or(&"100").parse().unwrap_or(100.0);
                        let val_f = val.parse().unwrap_or(0.0);
                        let slider = Slider::new(val_f, min..=max, |_| {});
                        slider.render(renderer, editor_rect);
                    }
                    "enum" => {
                        let options: Vec<String> = default.split(',').map(|s| s.trim().into()).collect();
                        let selected_idx = options.iter().position(|o| o == &val).unwrap_or(0);
                        let mut select = Select::new("Select...");
                        for (idx, opt) in options.iter().enumerate() {
                            select = select.option(opt.clone(), idx);
                        }
                        select = select.selected(selected_idx);
                        select.render(renderer, editor_rect);
                    }
                    "callback" => {
                        let editor = CodeEditor::new(default).language("rust");
                        editor.render(renderer, editor_rect);
                    }
                    _ => {
                        let input = Input::new(default);
                        input.render(renderer, editor_rect);
                    }
                }

                y += field_h + 12.0;
            }
        } else {
            // Empty state
            renderer.draw_text_raw(
                "Select a component to edit properties",
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

impl cvkg_core::layout::LayoutView for PropsPanel {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) -> Size {
        Size { width: proposal.width.unwrap_or(300.0), height: proposal.height.unwrap_or(600.0) }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) {}
}

impl PropsPanel {
    fn get_props_for_component(name: &str) -> Vec<(&'static str, &'static str, &'static str)> {
        // (property_name, type, default_value)
        match name {
            "Button" => vec![
                ("label", "string", "Click Me"),
                ("disabled", "bool", "false"),
                ("on_click", "callback", "// fn() { println!(\"clicked\"); }"),
            ],
            "Input" => vec![
                ("placeholder", "string", "Enter text..."),
                ("value", "string", ""),
                ("on_change", "callback", "// fn(String) { println!(\"changed\"); }"),
            ],
            "Toggle" => vec![
                ("label", "string", "Enable"),
                ("is_on", "bool", "false"),
                ("on_change", "callback", "// fn(bool) { println!(\"toggled\"); }"),
            ],
            "Slider" => vec![
                ("min", "number", "0,100"),
                ("max", "number", "0,100"),
                ("value", "number", "50"),
                ("on_change", "callback", "// fn(f32) { println!(\"slider\"); }"),
            ],
            "Select" => vec![
                ("options", "enum", "Option A,Option B,Option C"),
                ("selected", "number", "0"),
                ("on_change", "callback", "// fn(usize) { println!(\"selected\"); }"),
            ],
            _ => vec![],
        }
    }
}