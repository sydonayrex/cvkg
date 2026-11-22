//! API Docs tab — shows component props, usage examples, and copy-to-clipboard.

use crate::state::{ComponentMeta, GalleryState, Registry};
use cvkg_components::{
    Badge, BadgeVariant, Button, Codeblock, CopyButton, Divider, HStack, ScrollView, Text, VStack,
};
use cvkg_core::{Rect, Renderer, Size, SizeProposal, View};
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
        state
            .selected_component
            .as_ref()
            .and_then(|name| Registry::find(name))
    }

    /// Whether the API docs panel has both props and a usage example for this
    /// component. Used by the gallery coverage test.
    pub fn has_docs(name: &str) -> bool {
        let has_props = !Self::get_props_for_component(name).is_empty()
            || !Self::get_props_extra(name).is_empty();
        let has_example = Self::get_usage_example(name) != "// No example available"
            || !Self::get_usage_example_extra(name).is_empty();
        has_props && has_example
    }

    /// Whether the registry metadata generated a docs summary for `name`.
    pub fn has_generated_summary(name: &str) -> bool {
        crate::generated_docs::has_summary(name)
    }
}

impl View for DocsPanel {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let meta = self.get_selected_meta();

        // Panel background
        renderer.fill_rect(rect, [0.98, 0.98, 0.98, 1.0]);
        renderer.draw_line(
            rect.x,
            rect.y,
            rect.x + rect.width,
            rect.y,
            [0.7, 0.7, 0.7, 1.0],
            1.0,
        );

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

            // Description — generated from registry metadata, with the curated detail below.
            let description = match crate::generated_docs::summary(meta.name) {
                Some(summary) => format!("{summary}\n{}", meta.description),
                None => meta.description.to_string(),
            };
            renderer.draw_text_raw(&description, rect.x + pad, y, 13.0, [0.4, 0.4, 0.4, 1.0]);
            y += 24.0;

            // Divider
            renderer.draw_line(
                rect.x + pad,
                y,
                rect.x + rect.width - pad,
                y,
                [0.85, 0.85, 0.85, 1.0],
                1.0,
            );
            y += 16.0;

            // Category badge
            let badge = Badge::new(meta.category).variant(BadgeVariant::Secondary);
            badge.render(
                renderer,
                Rect {
                    x: rect.x + pad,
                    y,
                    width: 100.0,
                    height: 22.0,
                },
            );
            y += 30.0;

            // Props table
            renderer.draw_text_raw("Props", rect.x + pad, y, 14.0, [0.2, 0.2, 0.2, 1.0]);
            y += 22.0;

            let mut props = Self::get_props_for_component(&meta.name);
            props.extend(Self::get_props_extra(&meta.name));
            for (prop_name, prop_type, default, desc) in props {
                // Prop row background
                let row_h = 32.0;
                let row_rect = Rect {
                    x: rect.x + pad,
                    y,
                    width: content_w,
                    height: row_h,
                };
                renderer.fill_rect(row_rect, [0.95, 0.95, 0.95, 1.0]);
                renderer.draw_line(
                    rect.x + pad,
                    y + row_h,
                    rect.x + rect.width - pad,
                    y + row_h,
                    [0.85, 0.85, 0.85, 1.0],
                    1.0,
                );

                // Name + type badge
                renderer.draw_text_raw(
                    prop_name,
                    rect.x + pad + 8.0,
                    y + 8.0,
                    12.0,
                    [0.2, 0.2, 0.2, 1.0],
                );
                let type_badge = Badge::new(prop_type).variant(BadgeVariant::Outline);
                type_badge.render(
                    renderer,
                    Rect {
                        x: rect.x + pad + 140.0,
                        y: y + 4.0,
                        width: 80.0,
                        height: 22.0,
                    },
                );

                // Default
                if !default.is_empty() {
                    renderer.draw_text_raw(
                        &format!("default: {}", default),
                        rect.x + pad + 230.0,
                        y + 8.0,
                        11.0,
                        [0.5, 0.5, 0.5, 1.0],
                    );
                }

                // Description
                if !desc.is_empty() {
                    renderer.draw_text_raw(
                        desc,
                        rect.x + pad + 8.0,
                        y + row_h - 14.0,
                        10.0,
                        [0.6, 0.6, 0.6, 1.0],
                    );
                }

                y += row_h + 8.0;
            }

            y += 20.0;

            // Divider
            renderer.draw_line(
                rect.x + pad,
                y,
                rect.x + rect.width - pad,
                y,
                [0.85, 0.85, 0.85, 1.0],
                1.0,
            );
            y += 16.0;

            // Usage Example
            renderer.draw_text_raw("Usage Example", rect.x + pad, y, 14.0, [0.2, 0.2, 0.2, 1.0]);
            y += 22.0;

            let example = Self::get_usage_example(&meta.name);
            let extra = Self::get_usage_example_extra(&meta.name);
            let example = if extra.is_empty() { example } else { extra };
            let code_block = Codeblock::new(example).language("rust");
            let code_rect = Rect {
                x: rect.x + pad,
                y,
                width: content_w,
                height: 160.0,
            };
            code_block.render(renderer, code_rect);
            y += 170.0;

            // Copy to clipboard buttons
            renderer.draw_text_raw(
                "Copy to Clipboard",
                rect.x + pad,
                y,
                13.0,
                [0.2, 0.2, 0.2, 1.0],
            );
            y += 20.0;

            let copy_usage = CopyButton::new("Copy Usage", || {});
            copy_usage.render(
                renderer,
                Rect {
                    x: rect.x + pad,
                    y,
                    width: 140.0,
                    height: 32.0,
                },
            );

            let copy_props = CopyButton::new("Copy Props Table", || {});
            copy_props.render(
                renderer,
                Rect {
                    x: rect.x + pad + 150.0,
                    y,
                    width: 150.0,
                    height: 32.0,
                },
            );
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
        Size {
            width: proposal.width.unwrap_or(300.0),
            height: proposal.height.unwrap_or(600.0),
        }
    }
}

impl cvkg_core::layout::LayoutView for DocsPanel {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) -> Size {
        Size {
            width: proposal.width.unwrap_or(300.0),
            height: proposal.height.unwrap_or(600.0),
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

impl DocsPanel {
    fn get_props_for_component(
        name: &str,
    ) -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
        match name {
            "Button" => vec![
                ("label", "string", "\"Click Me\"", "Button text"),
                ("on_click", "fn()", "|| {}", "Click handler"),
                (
                    "variant",
                    "enum",
                    "Primary",
                    "Primary|Secondary|Destructive|Ghost",
                ),
                ("disabled", "bool", "false", "Disable interaction"),
                (
                    "world",
                    "WorldSpaceConfig",
                    "default",
                    "3D world-space config",
                ),
            ],
            "Input" => vec![
                (
                    "placeholder",
                    "string",
                    "\"Enter text...\"",
                    "Placeholder text",
                ),
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
                (
                    "options",
                    "Vec<(String, V)>",
                    "vec![...]",
                    "Display/value pairs",
                ),
                ("selected", "usize", "0", "Selected index"),
                ("placeholder", "string", "\"Choose...\"", "Placeholder"),
                ("on_change", "fn(V)", "|v| {}", "Selection handler"),
            ],
            _ => vec![],
        }
    }

    fn get_props_extra(
        name: &str,
    ) -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
        match name {
            "Checkbox" => vec![
                ("label", "string", "\"Remember me\"", "Label text"),
                ("checked", "bool", "false", "Checked state"),
                ("on_change", "fn(bool)", "|b| {}", "Change handler"),
            ],
            "DatePicker" => vec![
                (
                    "on_change",
                    "fn(u32,u32,u32)",
                    "|_,_,_| {}",
                    "Date selection handler",
                ),
                ("range_mode", "bool", "false", "Range selection"),
            ],
            "SearchField" => vec![
                ("text", "string", "\"\"", "Current query"),
                ("placeholder", "string", "\"Search...\"", "Placeholder text"),
                ("width", "f32", "220.0", "Field width"),
            ],
            "Dialog" => vec![
                ("content", "View", "Text::new(...)", "Dialog body"),
                ("title", "string", "\"\"", "Header title"),
                ("presented", "bool", "false", "Visibility"),
            ],
            "Popover" => vec![
                ("trigger", "View", "Button::new(...)", "Anchor view"),
                ("content", "View", "Text::new(...)", "Popover body"),
            ],
            "Tooltip" => vec![
                ("content", "View", "Button::new(...)", "Hover target"),
                ("text", "string", "\"\"", "Tooltip text"),
            ],
            "ContextMenu" => vec![("items", "Vec<ContextMenuItem>", "vec![...]", "Menu entries")],
            "Grid" => vec![
                (
                    "columns",
                    "Vec<GridTrack>",
                    "vec![Flex(1.0)]",
                    "Column tracks",
                ),
                ("rows", "Vec<GridTrack>", "vec![Auto]", "Row tracks"),
            ],
            "Avatar" => vec![
                ("fallback", "string", "\"AL\"", "Initials fallback"),
                ("status", "AvatarStatus", "Offline", "Presence indicator"),
            ],
            "Progress" => vec![
                ("value", "f32", "0.65", "Completion fraction"),
                ("variant", "ProgressVariant", "Default", "Visual style"),
            ],
            "Toast" => vec![
                ("title", "string", "\"Saved\"", "Toast title"),
                ("message", "string", "\"...\"", "Body text"),
                ("kind", "ToastKind", "Info", "Severity"),
                ("duration", "f32", "5.0", "Auto-dismiss seconds"),
            ],
            "Alert" => vec![
                ("title", "string", "\"Heads up\"", "Alert title"),
                ("message", "string", "\"...\"", "Body text"),
                ("kind", "AlertKind", "Information", "Severity"),
            ],
            "Skeleton" => vec![("(none)", "-", "-", "Create with DraumaSkeleton::new()")],
            "Tabs" => vec![
                ("options", "Vec<String>", "vec![...]", "Tab labels"),
                ("selected", "usize", "0", "Selected tab"),
                ("on_select", "fn(usize)", "|_| {}", "Selection handler"),
            ],
            "Breadcrumb" => vec![
                ("items", "Vec<BreadcrumbItem>", "vec![...]", "Trail entries"),
                ("on_select", "fn(usize)", "|_| {}", "Item handler"),
            ],
            "Pagination" => vec![
                ("total_pages", "usize", "5", "Page count"),
                ("on_change", "fn(usize)", "|_| {}", "Page handler"),
            ],
            "ColorPicker" => vec![("color", "[f32;4]", "[0.2,0.4,0.8,1.0]", "Initial RGBA")],
            "CommandPalette" => vec![
                ("search", "string", "\"\"", "Initial query"),
                ("open", "flag", "closed", "Visibility"),
                ("select", "usize", "0", "Selected result"),
            ],
            "Sidebar" => vec![(
                "items",
                "Vec<SidebarItem>",
                "vec![...]",
                "Navigation entries",
            )],
            "DockingWorkspace" => vec![
                (
                    "panels",
                    "Vec<Panel>",
                    "vec![...]",
                    "Docked and floating panels",
                ),
                ("active", "usize", "None", "Active panel id"),
            ],
            "VirtualList" => vec![
                ("data", "Vec<D>", "vec![...]", "Backing dataset"),
                ("item_height", "f32", "24.0", "Row height"),
                ("on_select", "fn(usize)", "None", "Selection handler"),
            ],
            "VStack" => vec![
                ("spacing", "f32", "8.0", "Gap between children"),
                ("alignment", "Alignment", "Start", "Cross-axis alignment"),
                (
                    "distribution",
                    "Distribution",
                    "Start",
                    "Main-axis distribution",
                ),
            ],
            "HStack" => vec![
                ("spacing", "f32", "8.0", "Gap between children"),
                ("alignment", "Alignment", "Start", "Cross-axis alignment"),
                (
                    "distribution",
                    "Distribution",
                    "Start",
                    "Main-axis distribution",
                ),
            ],
            "Divider" => vec![
                (
                    "orientation",
                    "enum",
                    "horizontal",
                    "horizontal() or vertical()",
                ),
                ("width", "f32", "1.0", "Stroke thickness"),
            ],
            "Table" => vec![
                ("data", "Vec<D>", "vec![...]", "Rows to display"),
                ("column", "fn(&str,f32,bool,fn)", "-", "Column definition"),
                ("row_height", "f32", "32.0", "Row height"),
            ],
            "Badge" => vec![
                ("text", "string", "\"New\"", "Badge label"),
                ("variant", "BadgeVariant", "Default", "Visual style"),
            ],
            _ => vec![],
        }
    }

    fn get_usage_example(name: &str) -> &'static str {
        match name {
            "Button" => {
                r#"use cvkg_components::Button;

let btn = Button::new("Click Me", || {
    println!("Clicked!");
})
.variant(ButtonVariant::Primary)
.disabled(false);"#
            }
            "Input" => {
                r#"use cvkg_components::Input;

let input = Input::new("Type here...")
    .placeholder("Enter text...")
    .on_change(|text| println!("Changed: {}", text));"#
            }
            "Toggle" => {
                r#"use cvkg_components::Toggle;

let toggle = Toggle::new("Enable Feature", false, |enabled| {
    println!("Feature: {}", enabled);
});"#
            }
            "Slider" => {
                r#"use cvkg_components::Slider;

let slider = Slider::new(0.5, 0.0..=100.0, |val| {
    println!("Value: {}", val);
})
.step(1.0);"#
            }
            "Select" => {
                r#"use cvkg_components::Select;

let options = vec![
    ("Option A".to_string(), "a"),
    ("Option B".to_string(), "b"),
];
let select = Select::new(options, 0)
    .placeholder("Choose...");"#
            }
            _ => "// No example available",
        }
    }

    fn get_usage_example_extra(name: &str) -> &'static str {
        match name {
            "Checkbox" => {
                r#"use cvkg_components::*;
let c = Checkbox::new("Remember me", false, |v| {});"#
            }
            "DatePicker" => {
                r#"use cvkg_components::*;
let d = DatePicker::new(|y, m, day| println!("{y}-{m}-{day}"));"#
            }
            "SearchField" => {
                r#"use cvkg_components::*;
let s = SearchField::new().text("cvkg");"#
            }
            "Dialog" => {
                r#"use cvkg_components::*;
let d = Dialog::new(Text::new("Body")).title("Details").presented(true);"#
            }
            "Popover" => {
                r#"use cvkg_components::*;
let p = Popover::new(Button::new("Open", || {}), Text::new("Content"));"#
            }
            "Tooltip" => {
                r#"use cvkg_components::*;
let t = Tooltip::new(Button::new("Hover", || {}).erase(), "Tip");"#
            }
            "ContextMenu" => {
                r#"use cvkg_components::*;
let m = ContextMenu::new(vec![ContextMenuItem::new("Open"), ContextMenuItem::new("Copy")]);"#
            }
            "Grid" => {
                r#"use cvkg_components::*;
let g = Grid::new(vec![GridTrack::Flex(1.0)], vec![GridTrack::Auto]).child(Text::new("A"));"#
            }
            "Avatar" => {
                r#"use cvkg_components::*;
let a = Avatar::new("AL");"#
            }
            "Progress" => {
                r#"use cvkg_components::*;
let p = Progress::new(0.65);"#
            }
            "Toast" => {
                r#"use cvkg_components::*;
let mut m = ToastManager::new();
m.info("Saved", "Changes saved");"#
            }
            "Alert" => {
                r#"use cvkg_components::*;
let a = Alert::new("Heads up", "Needs attention").kind(AlertKind::Warning);"#
            }
            "Skeleton" => {
                r#"use cvkg_components::*;
let s = DraumaSkeleton::new();"#
            }
            "Tabs" => {
                r#"use cvkg_components::*;
let t = Tabs::new(vec!["One".into(), "Two".into()], 0, |_| {});"#
            }
            "Breadcrumb" => {
                r#"use cvkg_components::*;
let b = Breadcrumb::new(vec![BreadcrumbItem::new("Home"), BreadcrumbItem::new("Gallery")]);"#
            }
            "Pagination" => {
                r#"use cvkg_components::*;
let p = Pagination::new(5, |page| println!("{page}"));"#
            }
            "ColorPicker" => {
                r#"use cvkg_components::*;
let c = ColorPicker::new([0.2, 0.4, 0.8, 1.0]);"#
            }
            "CommandPalette" => {
                r#"use cvkg_components::*;
let c = CommandPalette::new().open();"#
            }
            "Sidebar" => {
                r#"use cvkg_components::*;
let s = NiflheimSidebar::new(vec![SidebarItem::new("home", "Home")]);"#
            }
            "DockingWorkspace" => {
                r#"use cvkg_components::*;
let d = DockingWorkspace::new()
    .panel("main", "Main", Text::new("Body"), PanelPosition::Center)
    .active("main");"#
            }
            "VirtualList" => {
                r#"use cvkg_components::*;
let v = VirtualList::new(vec![1, 2, 3], 24.0, |n: &i32, _| Text::new(n.to_string()));"#
            }
            "VStack" => {
                r#"use cvkg_components::*;
let s = VStack::new(8.0).child(Text::new("First")).child(Text::new("Second"));"#
            }
            "HStack" => {
                r#"use cvkg_components::*;
let s = HStack::new(8.0).child(Text::new("Left")).child(Text::new("Right"));"#
            }
            "Divider" => {
                r#"use cvkg_components::Divider;
let d = Divider::horizontal();"#
            }
            "Table" => {
                r#"use cvkg_components::*;
let t = RunesTable::new(vec!["Alice".to_string()])
    .column("Name", 160.0, true, |name: &String| Text::new(name.clone()));"#
            }
            "Badge" => {
                r#"use cvkg_components::Badge;
let b = Badge::new("New");"#
            }
            _ => "",
        }
    }
}
