//! Gallery application state — shared across sidebar, canvas, toolbar, and commands.

use cvkg_components::{Button, Dialog, HStack, Input, MimirSpotlight, Select, Slider, Text, Toggle, VStack};
use cvkg_components::chrome::NiflheimSidebar;
use cvkg_core::{AnyView, Color, Rect, Size, View};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use urlencoding;

/// Component metadata for the registry and sidebar.
#[derive(Clone, Debug)]
pub struct ComponentMeta {
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    /// Factory function to create a live instance for the canvas.
    pub factory: fn(&GalleryState) -> AnyView,
}

impl ComponentMeta {
    pub const fn new(
        name: &'static str,
        category: &'static str,
        description: &'static str,
        factory: fn(&GalleryState) -> AnyView,
    ) -> Self {
        Self { name, category, description, factory }
    }
}

/// Theme variants for the gallery.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GalleryTheme {
    #[default]
    Light,
    Dark,
    HighContrast,
}

/// Viewport scale presets for responsive preview.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ViewportScale {
    #[default]
    Percent100 = 100,
    Percent150 = 150,
    Percent200 = 200,
    Percent75 = 75,
    Percent50 = 50,
}

impl ViewportScale {
    pub const ALL: &'static [Self] = &[
        Self::Percent50,
        Self::Percent75,
        Self::Percent100,
        Self::Percent150,
        Self::Percent200,
    ];

    pub fn factor(&self) -> f32 {
        *self as u32 as f32 / 100.0
    }
}

/// Viewport presets for responsive preview (mobile/tablet/desktop).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ViewportPreset {
    #[default]
    Desktop = 1440,
    Tablet = 768,
    Mobile = 375,
}

impl ViewportPreset {
    pub const ALL: &'static [Self] = &[
        Self::Desktop,
        Self::Tablet,
        Self::Mobile,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Desktop => "Desktop (1440)",
            Self::Tablet => "Tablet (768)",
            Self::Mobile => "Mobile (375)",
        }
    }

    pub fn width(&self) -> f32 {
        *self as u32 as f32
    }
}

/// Background styles for the canvas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CanvasBackground {
    #[default]
    Light,
    Dark,
    Checkered,
    Transparent,
}

/// Shared gallery state — single source of truth for the whole app.
#[derive(Clone, Debug, Default)]
pub struct GalleryState {
    /// Currently selected component (by registry name).
    pub selected_component: Option<String>,
    /// Expanded sidebar categories.
    pub expanded_categories: HashMap<String, bool>,
    /// Theme selection.
    pub theme: GalleryTheme,
    /// Viewport scale.
    pub scale: ViewportScale,
    /// Canvas background.
    pub canvas_bg: CanvasBackground,
    /// Viewport preset for responsive preview.
    pub viewport: ViewportPreset,
    /// Search text for spotlight.
    pub search_text: String,
    /// Whether spotlight is open.
    pub spotlight_open: bool,
    /// Selected command index in spotlight.
    pub spotlight_selected: usize,
    /// Props panel visibility.
    pub props_panel_open: bool,
    /// Event log for callback debugging.
    pub event_log: Vec<String>,
    /// Recent components (persisted).
    pub recent: Vec<String>,
    /// Persisted theme preference.
    pub persisted_theme: Option<GalleryTheme>,
    /// Persisted scale preference.
    pub persisted_scale: Option<ViewportScale>,
    /// Persisted canvas background.
    pub persisted_canvas_bg: Option<CanvasBackground>,
    /// Persisted sidebar width.
    pub persisted_sidebar_width: Option<f32>,
    /// Persisted expanded categories.
    pub persisted_expanded_categories: HashMap<String, bool>,
    /// Current sidebar width (for drag resize).
    pub sidebar_width: f32,
    /// Whether sidebar divider is being dragged.
    pub dragging_sidebar: bool,
}

impl GalleryState {
    /// Get or create the global state instance.
    pub fn global() -> Arc<Mutex<GalleryState>> {
        static INSTANCE: OnceLock<Arc<Mutex<GalleryState>>> = OnceLock::new();
        INSTANCE.get_or_init(|| Arc::new(Mutex::new(GalleryState::default()))).clone()
    }

    /// Toggle a sidebar category expanded state.
    pub fn toggle_category(&mut self, category: &str) {
        let entry = self.expanded_categories.entry(category.to_string()).or_insert(true);
        *entry = !*entry;
    }

    /// Select a component by name.
    pub fn select_component(&mut self, name: &str) {
        self.selected_component = Some(name.to_string());
        // Add to recent (max 5)
        if !self.recent.iter().any(|r| r == name) {
            self.recent.insert(0, name.to_string());
            if self.recent.len() > 5 { self.recent.pop(); }
        }
    }

    /// Log an event for the event log panel.
    pub fn log_event(&mut self, event: &str) {
        self.event_log.push(format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), event));
        if self.event_log.len() > 100 { self.event_log.remove(0); }
    }

    /// Load persisted state from localStorage (via cvkg-native bridge).
    pub fn load_persisted(&mut self) {
        // In a real implementation, this would call into the native layer
        // to read from localStorage or a config file.
        // For now, we restore from the in-memory persisted fields.
        if let Some(t) = self.persisted_theme {
            self.theme = t;
        }
        if let Some(s) = self.persisted_scale {
            self.scale = s;
        }
        if let Some(bg) = self.persisted_canvas_bg {
            self.canvas_bg = bg;
        }
        if let Some(w) = self.persisted_sidebar_width {
            // Sidebar width restoration handled in sidebar render
        }
        self.expanded_categories = self.persisted_expanded_categories.clone();
    }

    /// Save current state to persistence.
    pub fn save_persisted(&mut self) {
        self.persisted_theme = Some(self.theme);
        self.persisted_scale = Some(self.scale);
        self.persisted_canvas_bg = Some(self.canvas_bg);
        self.persisted_sidebar_width = Some(240.0); // TODO: get from sidebar render
        self.persisted_expanded_categories = self.expanded_categories.clone();
    }

    /// Serialize state for URL sharing.
    pub fn to_url_params(&self) -> String {
        let mut params = Vec::new();
        if let Some(comp) = &self.selected_component {
            params.push(format!("comp={}", urlencoding::encode(comp)));
        }
        params.push(format!("theme={:?}", self.theme));
        params.push(format!("scale={}", self.scale.factor()));
        params.push(format!("bg={:?}", self.canvas_bg));
        params.join("&")
    }

    /// Deserialize state from URL params.
    pub fn from_url_params(params: &str) -> Self {
        let mut state = Self::default();
        for pair in params.split('&') {
            let kv: Vec<&str> = pair.split('=').collect();
            if kv.len() == 2 {
                match kv[0] {
                    "comp" => state.selected_component = Some(urlencoding::decode(kv[1]).unwrap_or_default().into_owned()),
                    "theme" => state.theme = match kv[1] {
                        "Dark" => GalleryTheme::Dark,
                        "HighContrast" => GalleryTheme::HighContrast,
                        _ => GalleryTheme::Light,
                    },
                    "scale" => {
                        if let Ok(v) = kv[1].parse::<u32>() {
                            state.scale = match v {
                                50 => ViewportScale::Percent50,
                                75 => ViewportScale::Percent75,
                                100 => ViewportScale::Percent100,
                                150 => ViewportScale::Percent150,
                                200 => ViewportScale::Percent200,
                                _ => ViewportScale::Percent100,
                            };
                        }
                    }
                    "bg" => {
                        state.canvas_bg = match kv[1] {
                            "Dark" => CanvasBackground::Dark,
                            "Checkered" => CanvasBackground::Checkered,
                            "Transparent" => CanvasBackground::Transparent,
                            _ => CanvasBackground::Light,
                        };
                    }
                    _ => {}
                }
            }
        }
        state
    }
}

/// Component registry — hardcoded for Phase 1, auto-generated in Phase 5.
pub struct Registry;

impl Registry {
    /// Get all registered components.
    pub fn all() -> Vec<ComponentMeta> {
        vec![
            // Forms
            ComponentMeta::new(
                "Button", "Forms", "Interactive button with variants and states",
                |_| AnyView::new(crate::canvas::ButtonShowcase)
            ),
            ComponentMeta::new(
                "Checkbox", "Forms", "Checkbox with label and indeterminate state",
                |_| AnyView::new(crate::canvas::CheckboxShowcase)
            ),
            ComponentMeta::new(
                "Input", "Forms", "Text input with validation and states",
                |_| AnyView::new(crate::canvas::InputShowcase)
            ),
            ComponentMeta::new(
                "Select", "Forms", "Dropdown select with search",
                |_| AnyView::new(crate::canvas::SelectShowcase)
            ),
            ComponentMeta::new(
                "Toggle", "Forms", "Animated toggle switch",
                |_| AnyView::new(crate::canvas::ToggleShowcase)
            ),
            ComponentMeta::new(
                "Slider", "Forms", "Range slider with marks",
                |_| AnyView::new(crate::canvas::SliderShowcase)
            ),
            ComponentMeta::new(
                "DatePicker", "Forms", "Date picker with calendar",
                |_| AnyView::new(crate::canvas::DatePickerShowcase)
            ),
            ComponentMeta::new(
                "SearchField", "Forms", "Search input with suggestions",
                |_| AnyView::new(crate::canvas::SearchFieldShowcase)
            ),

            // Overlays
            ComponentMeta::new(
                "Dialog", "Overlays", "Modal dialog with actions",
                |_| AnyView::new(crate::canvas::DialogShowcase)
            ),
            ComponentMeta::new(
                "Popover", "Overlays", "Floating popover anchored to trigger",
                |_| AnyView::new(crate::canvas::PopoverShowcase)
            ),
            ComponentMeta::new(
                "Tooltip", "Overlays", "Hover tooltip with positioning",
                |_| AnyView::new(crate::canvas::TooltipShowcase)
            ),
            ComponentMeta::new(
                "ContextMenu", "Overlays", "Right-click context menu",
                |_| AnyView::new(crate::canvas::ContextMenuShowcase)
            ),

            // Layout
            ComponentMeta::new(
                "VStack", "Layout", "Vertical stack with alignment",
                |_| AnyView::new(crate::canvas::VStackShowcase)
            ),
            ComponentMeta::new(
                "HStack", "Layout", "Horizontal stack with alignment",
                |_| AnyView::new(crate::canvas::HStackShowcase)
            ),
            ComponentMeta::new(
                "Grid", "Layout", "Responsive grid layout",
                |_| AnyView::new(crate::canvas::GridShowcase)
            ),
            ComponentMeta::new(
                "Divider", "Layout", "Visual separator",
                |_| AnyView::new(crate::canvas::DividerShowcase)
            ),

            // Data Display
            ComponentMeta::new(
                "Table", "Data Display", "Sortable, selectable data table",
                |_| AnyView::new(crate::canvas::TableShowcase)
            ),
            ComponentMeta::new(
                "Badge", "Data Display", "Status badges and labels",
                |_| AnyView::new(crate::canvas::BadgeShowcase)
            ),
            ComponentMeta::new(
                "Avatar", "Data Display", "User avatar with fallback",
                |_| AnyView::new(crate::canvas::AvatarShowcase)
            ),
            ComponentMeta::new(
                "Progress", "Data Display", "Progress bars and spinners",
                |_| AnyView::new(crate::canvas::ProgressShowcase)
            ),

            // Feedback
            ComponentMeta::new(
                "Toast", "Feedback", "Transient notifications",
                |_| AnyView::new(crate::canvas::ToastShowcase)
            ),
            ComponentMeta::new(
                "Alert", "Feedback", "Inline alert messages",
                |_| AnyView::new(crate::canvas::AlertShowcase)
            ),
            ComponentMeta::new(
                "Skeleton", "Feedback", "Loading placeholders",
                |_| AnyView::new(crate::canvas::SkeletonShowcase)
            ),

            // Navigation
            ComponentMeta::new(
                "Tabs", "Navigation", "Tab navigation with panels",
                |_| AnyView::new(crate::canvas::TabsShowcase)
            ),
            ComponentMeta::new(
                "Breadcrumb", "Navigation", "Hierarchical breadcrumb trail",
                |_| AnyView::new(crate::canvas::BreadcrumbShowcase)
            ),
            ComponentMeta::new(
                "Pagination", "Navigation", "Page navigation controls",
                |_| AnyView::new(crate::canvas::PaginationShowcase)
            ),

            // Advanced
            ComponentMeta::new(
                "ColorPicker", "Advanced", "Color picker with multiple formats",
                |_| AnyView::new(crate::canvas::ColorPickerShowcase)
            ),
            ComponentMeta::new(
                "CommandPalette", "Advanced", "Full command palette (MimirSpotlight)",
                |_| AnyView::new(crate::canvas::CommandPaletteShowcase)
            ),
            ComponentMeta::new(
                "Sidebar", "Advanced", "Collapsible navigation sidebar (NiflheimSidebar)",
                |_| AnyView::new(crate::canvas::SidebarShowcase)
            ),
        ]
    }

    /// Get components grouped by category.
    pub fn by_category() -> HashMap<&'static str, Vec<ComponentMeta>> {
        let mut map = HashMap::new();
        for meta in Self::all() {
            map.entry(meta.category).or_insert_with(Vec::new).push(meta);
        }
        map
    }

    /// Find a component by name.
    pub fn find(name: &str) -> Option<ComponentMeta> {
        Self::all().into_iter().find(|m| m.name.eq_ignore_ascii_case(name))
    }
}