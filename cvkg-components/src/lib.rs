//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     -- State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     -- Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     -- Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    -- Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     -- Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     -- Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   -- Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//!   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//!   CVKG Extended: Section 2 of the CVKG Design Specification
#![allow(
    clippy::too_many_arguments,
    clippy::new_without_default,
    clippy::needless_range_loop,
    clippy::large_enum_variant,
    clippy::manual_clamp,
    clippy::doc_lazy_continuation,
    clippy::needless_borrow,
    ambiguous_glob_reexports,
    clippy::type_complexity,
    clippy::unusual_byte_groupings,
    clippy::collapsible_if,
    clippy::manual_range_contains,
    clippy::derivable_impls,
    clippy::match_like_matches_macro,
    clippy::collapsible_match
)]

//! Built-in component library for CVKG
//!
//! This crate implements standard CVKG components using public CVKG APIs.

// --- Shared Types ---

/// Font weight for text rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Regular,
    Bold,
    Italic,
}

pub use cvkg_core::Color;

// =============================================================================
// TYPOGRAPHY SYSTEM -- Centralized design tokens for text sizing
// =============================================================================

/// Typography scale providing consistent font sizes across all components.
/// Replace all magic font numbers with these tokens.
pub const FONT_XS: f32 = 10.0;
pub const FONT_SM: f32 = 12.0;
pub const FONT_BASE: f32 = 14.0;
pub const FONT_MD: f32 = 16.0;
pub const FONT_LG: f32 = 20.0;
pub const FONT_XL: f32 = 24.0;
pub const FONT_2XL: f32 = 32.0;
pub const FONT_3XL: f32 = 48.0;

/// Line height multipliers for each font size.
pub const LINE_HEIGHT_XS: f32 = 1.4;
pub const LINE_HEIGHT_SM: f32 = 1.4;
pub const LINE_HEIGHT_BASE: f32 = 1.5;
pub const LINE_HEIGHT_MD: f32 = 1.5;
pub const LINE_HEIGHT_LG: f32 = 1.5;
pub const LINE_HEIGHT_XL: f32 = 1.4;
pub const LINE_HEIGHT_2XL: f32 = 1.3;
pub const LINE_HEIGHT_3XL: f32 = 1.2;

// =============================================================================
// SPACING SCALE -- Consistent spacing tokens
// =============================================================================

/// Spacing scale for layout consistency.
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 16.0;
pub const SPACE_LG: f32 = 24.0;
pub const SPACE_XL: f32 = 32.0;

// =============================================================================
// BORDER RADIUS SCALE -- Consistent corner radii
// =============================================================================

/// Border radius scale for consistent corner rounding.
pub const RADIUS_XS: f32 = 2.0;
pub const RADIUS_SM: f32 = 4.0;
pub const RADIUS_MD: f32 = 6.0;
pub const RADIUS_LG: f32 = 8.0;
pub const RADIUS_XL: f32 = 12.0;
pub const RADIUS_2XL: f32 = 16.0;
pub const RADIUS_FULL: f32 = 9999.0;

// =============================================================================
// FOCUS RING SYSTEM -- WCAG 2.4.7 compliant focus indicators
// =============================================================================

/// Focus ring width in logical pixels.
pub const FOCUS_RING_WIDTH: f32 = 2.0;

/// Focus ring offset from the element bounds.
pub const FOCUS_RING_OFFSET: f32 = 2.0;

/// Default focus ring color (cyan accent).
///
/// **Deprecated:** Use `crate::theme::focus_ring()` instead, which respects the
/// active theme and avoids hardcoded values.
#[deprecated(since = "0.2.0", note = "use `theme::focus_ring()` instead")]
pub const FOCUS_RING_COLOR: [f32; 4] = [0.0, 0.8, 1.0, 0.8];

/// Draws a focus ring around the given rectangle.
/// This function should be called by every interactive component when it has focus.
///
/// # Contract
/// - Uses `FOCUS_RING_WIDTH` for stroke width
/// - Uses `FOCUS_RING_OFFSET` to expand outward from the element bounds
/// - Uses `crate::theme::focus_ring()` for the color (theme-aware)
pub fn draw_focus_ring(renderer: &mut dyn cvkg_core::Renderer, rect: cvkg_core::Rect) {
    let outline_rect = cvkg_core::Rect {
        x: rect.x - FOCUS_RING_OFFSET,
        y: rect.y - FOCUS_RING_OFFSET,
        width: rect.width + FOCUS_RING_OFFSET * 2.0,
        height: rect.height + FOCUS_RING_OFFSET * 2.0,
    };
    let ring_color = crate::theme::focus_ring();
    renderer.stroke_rounded_rect(outline_rect, RADIUS_SM, ring_color, FOCUS_RING_WIDTH);
}

/// Draws a focus ring with a custom color.
pub fn draw_focus_ring_color(
    renderer: &mut dyn cvkg_core::Renderer,
    rect: cvkg_core::Rect,
    color: [f32; 4],
) {
    let outline_rect = cvkg_core::Rect {
        x: rect.x - FOCUS_RING_OFFSET,
        y: rect.y - FOCUS_RING_OFFSET,
        width: rect.width + FOCUS_RING_OFFSET * 2.0,
        height: rect.height + FOCUS_RING_OFFSET * 2.0,
    };
    renderer.stroke_rounded_rect(outline_rect, RADIUS_SM, color, FOCUS_RING_WIDTH);
}

// =============================================================================
// BUTTON VARIANTS -- Standard button styles
// =============================================================================

/// Button visual variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Primary action button.
    Default,
    /// Destructive/danger action button.
    Destructive,
    /// Outlined/ghost button.
    Secondary,
    /// Subtle background button.
    Ghost,
    /// Link-styled button.
    Link,
    /// Glass button: frosted background, no border, subtle backdrop.
    Glass,
    /// Tinted glass: glass base with accent color tint.
    TintedGlass,
    /// Capsule button: pill-shaped, solid fill, high contrast.
    Capsule,
}

/// Button size variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Small,
    Default,
    Large,
    Icon,
}

// Re-export submodules
pub mod advanced_forms;
pub mod ai_workflow_builder;
pub mod bifrost_tabs;
pub mod calendar;
pub mod card;
pub mod chrome;
pub mod clipped_corner;
pub mod collaboration;
pub mod command;
pub mod command_palette;
pub mod container;
pub mod data_grid;
pub mod devtools;
pub mod docking_workspace;
pub mod dropdown_menu;
pub mod effects;
pub mod error;
pub mod file_tree;
pub mod flexiscope;
pub mod flux_layout;
pub mod gpu_charts;
pub mod gradient;
pub mod grid;
pub mod holographic_runestone;
pub mod hud;
pub mod image;
pub mod infinite_canvas;
pub mod interactive;
pub mod memory;
pub mod mjolnir_frame;
pub mod mjolnir_slider;
pub mod multi_agent_orchestrator;
pub mod navigation;
pub mod niflheim_demo;
pub mod node_graph_editor;
pub mod oracle_orb;
pub mod ornamental;
pub mod primitive;
pub mod radial_menu;
pub mod raven_messenger;
pub mod richtext;
pub mod runestone_decoder;
pub mod runestone_editor;
pub use radial_menu::RadialMenu;
pub mod scribing_stone;
pub mod semantic_memory_explorer;
pub mod shapes;
pub mod shield_wall;
pub mod valkyrie_indicator;
pub mod virtual_list;
pub mod visual;
pub mod window;
pub mod wyrd_hud;

pub mod asset_browser;
pub mod freyr_inspector;
pub mod gerd_telemetry;
pub mod gullveig_inspector;
pub mod idunn_persistence;
pub mod layer_system;
pub mod njord_theme;
pub mod skadi_scripting;
pub mod theme;
pub mod theme_switch;
pub mod timeline_editor;

pub use advanced_forms::{Calendar, EikonaForm};
pub use ai_workflow_builder::*;
pub use asset_browser::*;
pub use bifrost_tabs::BifrostTabs;
pub use calendar::*;
pub use card::RunesCard;
pub use clipped_corner::*;
pub use collaboration::*;
pub use command::*;
pub use command_palette::{Launcher, MimirSpotlight, PaletteCommand};

/// Deprecated: Use `Launcher` instead.
#[deprecated(note = "Use `Launcher` instead")]
pub type BifrostLauncher = Launcher;
pub use container::{
    Collapsible, DialogAction, FlexBox, GeriDialog, GjallarSplitter, GraniSheet, HStack,
    LazyVStack, NavigationSplitView, NavigationStack, SagaAccordion, ScrollView, SettingsForm,
    SheetModifier, SheetPosition, VStack,
};
pub use data_grid::RunesTable;
pub use devtools::*;
pub use docking_workspace::*;
pub use dropdown_menu::{DropdownItem, DropdownMenu};

pub use effects::*;
pub use error::*;
pub use file_tree::{FileItem, FileKind, YggdrasilTree};
pub use freyr_inspector::*;
pub use gerd_telemetry::*;
pub use grid::*;
pub use gullveig_inspector::*;
pub use holographic_runestone::HolographicRunestone;
pub use hud::*;
pub use hud::{AlertKind, GjallarAlert, TacticalGauge, Vegvísir};
pub use idunn_persistence::*;
pub use image::*;
pub use interactive::{
    BifrostColorPicker, Button, Checkbox, GeriTransfer, HringrPagination, Input, Picker,
    SecureField, Select, Slider, Stepper, Textarea, Toggle, ValhallaRating,
};
pub use layer_system::*;
pub use memory::*;
pub use mjolnir_frame::MjolnirFrame;
pub use mjolnir_slider::MjolnirSlider;
pub use multi_agent_orchestrator::*;
pub use niflheim_demo::*;
pub use njord_theme::*;
pub use oracle_orb::OracleOrb;
pub use primitive::*;
pub use raven_messenger::RavenMessenger;
pub use richtext::*;
pub use runestone_decoder::RunestoneDecoder;
pub use runestone_editor::RunestoneEditor;
pub use scribing_stone::ScribingStone;
pub use semantic_memory_explorer::*;
pub use shapes::*;
pub use shield_wall::ShieldWall;
pub use skadi_scripting::*;
pub use timeline_editor::*;
pub use valkyrie_indicator::ValkyrieIndicator;
pub use virtual_list::*;
pub use visual::{
    AvatarStatus, ChartType, DraumaSkeleton, EmptyState, HatiCarousel, HatiSpinner, MerkiBadge,
    MimirsWell, MuninAvatar, ProgressVariant, RuneScript, RunicTooltip, SkollProgress, SleipnirGait, SpinnerVariant,
    StatusBar, TelemetryView, UrdrTimeline, ValkyrieAnalytics, VölvaScan,
};
pub use window::{GinnungagapWindow, HiminnModal, YggdrasilWindow};
pub use wyrd_hud::WyrdHUD;
pub mod advanced;
pub mod autocomplete;
pub mod bragi_creative;
pub mod combobox;
pub use advanced::*;
pub mod ai_components;
pub use ai_components::*;
pub mod a11y_inspector;
pub mod datepicker;

pub mod font_axis_panel;
pub mod form_validation;
pub use font_axis_panel::FontAxisPanel;
pub mod a11y_beacon;
pub mod await_veil;
pub mod computed_signal;
pub mod consent_gate;
pub mod drop_vault;
pub mod hlin_accessibility;
pub mod keyboard_nav;
pub mod lingua_tong;
pub mod morph_bridge;
pub mod notification_center;
pub mod outline_view;
pub mod perf_overlay;
pub mod phasegate;
pub mod popover;
pub mod prompt_forge;
pub mod radio_group;
pub mod sync_weave;
pub mod text_editor;
pub mod toast;
pub mod token_stream;

pub mod anim;
pub mod animation_triggers;
pub mod trustmark;
pub mod tyr_security;
pub mod vtree;

// ── New UI parity components ──
pub mod breadcrumb;
pub mod button_group;
pub mod context_menu;
pub mod direction;
pub mod editable;
pub mod form_binder;
pub mod hover_card;
pub mod input_group;
pub mod input_otp;
pub mod item;
pub mod kbd;
pub mod mention_input;
pub mod native_select;
pub mod phone_input;
pub mod popconfirm;
pub mod qrcode;
pub mod sonner;
pub mod toggle_group;

// ── New HIGH priority components from cvkg-com-pool ──
pub mod agent_chat;
pub mod dialog;
pub mod display;
pub mod form_controls;
pub mod layout_components;
pub mod layout_primitives;
pub mod m3_components;
pub mod morph;
pub mod multimedia;
pub mod patterns;
pub mod scheduler;
pub mod text_anim;
pub mod tree_view;

pub use a11y_beacon::{A11yBeacon, A11yBeaconExt};
pub use a11y_inspector::{A11yInspector, A11yNode};
pub use autocomplete::*;
pub use await_veil::AwaitVeil;
pub use bragi_creative::*;
pub use breadcrumb::{Breadcrumb, BreadcrumbItem};
pub use button_group::ButtonGroup;
pub use combobox::*;
pub use computed_signal::{ComputedSignal, InputRef};
pub use consent_gate::{ConsentGate, DataTrail, TrailKind};
pub use context_menu::{ContextMenu, ContextMenuItem};
pub use cvkg_layout as layout;
pub use datepicker::*;
pub use direction::DirectionProvider;
pub use drop_vault::{DropVault, VaultEntry, VaultFile, VaultStatus};
pub use editable::Editable;
pub use form_binder::{FormBinder, FormBinding};
pub use form_validation::FormField;
pub use hover_card::{HoverCard, HoverCardPosition};
pub use input_group::InputGroup;
pub use input_otp::InputOTP;
pub use item::Item;
pub use kbd::Kbd;
pub use mention_input::MentionInput;
pub use native_select::NativeSelect;
pub use phone_input::PhoneInput;
pub use popconfirm::Popconfirm;
pub use popover::Popover;
pub use qrcode::QRCode;
pub use radio_group::RadioGroup;
pub use sonner::{Sonner, SonnerPosition, SonnerToast, SonnerType};
pub use toast::ToastManager;
pub use toggle_group::ToggleGroup;

pub use flexiscope::{ContainerLayout, FlexiScope, ScopeThreshold, fluid_typography};
pub use flux_layout::FluxState;
pub use hlin_accessibility::*;
pub use lingua_tong::{
    current_locale, init_english, is_rtl, load_translations, set_locale, t, t_with,
};
pub use morph_bridge::{MorphBridge, lerp_rect};
pub use notification_center::*;
pub use outline_view::{OutlineNode, OutlineView};
pub use perf_overlay::PerfOverlay;
pub use phasegate::{GateTier, PhaseGate};
pub use popover::*;
pub use prompt_forge::{ForgeSegment, PromptForge};
pub use radio_group::*;
pub use sync_weave::{PeerCursor, SyncEditor, SyncWeave, WeaveOp};
pub use text_editor::TextEditor;
pub use toast::*;
pub use token_stream::TokenStream;

pub use anim::*;
pub use trustmark::*;
pub use tyr_security::*;
pub use vtree::{VTree, VTreeNode};

// ── New HIGH priority component exports ──
pub use dialog::{AlertDialog, AlertVariant, ConfirmationDialog, FullScreenCover};
pub use display::{BackgroundPattern, BgPattern, Icon, ScrollArea, Typography, TypographyVariant};
pub use form_controls::{DateTimePicker, Label, Link, SearchField, SearchSuggestions, Tag};
pub use gpu_charts::{
    BarChart, Candle, CandlestickChart, FunnelChart, GaugeChart, HeatmapChart, Histogram,
    LineChart, PieChart, RadarChart, RangeChart, SankeyChart, ScatterPlot, SparkLineChart,
    TreemapChart, TreemapNode,
};
pub use layout_components::{
    BentoGrid, Carousel, FloatingNavbar, Loader, LoaderVariant, Marquee, MultiStepLoader,
    NavbarMenu,
};
pub use layout_primitives::{
    AspectRatio, Group, GroupBox, LazyHGrid, LazyHStack, LazyVGrid, Resizable, Separator, ZStack,
};
pub use m3_components::{
    BgMediaHero, Codeblock, DateRangePicker, DynamicIsland, ExtendedFAB, FAB, HeroColorPanels,
    Kanban, KanbanCard, KanbanColumn, LogoCarousel, SidePanel, TimePicker,
};
pub use morph::{MorphState, Morphed, ViewMorphExt};
pub use multimedia::{Audio, Map, Video};
pub use navigation::{DisclosureGroup, Drawer, List, Menubar, NavigationMenu, Section};
pub use patterns::{Gallery, Login, Settings, Wizard};
pub use scheduler::{Gantt, GanttTask, Scheduler, SchedulerEvent};
pub use text_anim::{
    CardHoverEffect, CardStack, DraggableCard, ExpandableCard, NumberTicker, RippleButton,
    ShimmerButton, StatefulButton, TextAnimEffect, TextAnimate, TypewriterEffect,
};
pub use tree_view::{RichTreeView, TreeViewNode};

// Internal Never type for primitive views
#[doc(hidden)]
pub use cvkg_core::Never;
pub use cvkg_core::Orientation;

/// Extension trait for all views to add component-level modifiers like .sheet()
pub trait ViewExt: cvkg_core::View + Sized {
    /// Present a modal sheet over this view.
    /// The modal uses a glassmorphic rounded rectangle with a mostly clear center and frosted edges.
    fn sheet<V: cvkg_core::View + Clone + 'static>(
        self,
        is_presented: bool,
        content: V,
    ) -> cvkg_core::ModifiedView<Self, container::SheetModifier<V>> {
        self.modifier(container::SheetModifier {
            is_presented,
            content,
        })
    }
}

// Blanket implementation for all Views
impl<T: cvkg_core::View + Sized> ViewExt for T {}

// =============================================================================
// MICRO-FEEDBACK -- Wrapper view that provides haptic/audio feedback hooks
// =============================================================================

/// A wrapper view that provides haptic and audio feedback for any content.
///
/// `MicroFeedback` wraps any view and injects haptic/audio feedback hooks
/// that child views can invoke through the renderer. This is useful as a
/// top-level wrapper for screens that want consistent feedback behavior.
///
/// The haptic and audio engines can be overridden at construction time;
/// by default they use the global system engines.
pub struct MicroFeedback<V: cvkg_core::View> {
    /// The wrapped content view.
    pub content: V,
    /// Haptic engine reference for tactile feedback.
    pub haptic: std::sync::Arc<dyn cvkg_core::HapticEngine>,
    /// Audio engine reference for sound feedback.
    pub audio: std::sync::Arc<dyn cvkg_core::AudioEngine>,
}

impl<V: cvkg_core::View> MicroFeedback<V> {
    /// Create a new `MicroFeedback` wrapper around `content` using the
    /// global default haptic and audio engines.
    pub fn new(content: V) -> Self {
        Self {
            content,
            haptic: std::sync::Arc::new(cvkg_core::NullHapticEngine),
            audio: std::sync::Arc::new(cvkg_core::NullAudioEngine),
        }
    }

    /// Set a custom haptic engine.
    pub fn with_haptic(mut self, engine: std::sync::Arc<dyn cvkg_core::HapticEngine>) -> Self {
        self.haptic = engine;
        self
    }

    /// Set a custom audio engine.
    pub fn with_audio(mut self, engine: std::sync::Arc<dyn cvkg_core::AudioEngine>) -> Self {
        self.audio = engine;
        self
    }

    /// Trigger a haptic impact through this wrapper's engine.
    pub fn trigger_haptic(&self, intensity: cvkg_core::HapticIntensity) {
        self.haptic.impact(intensity);
    }

    /// Trigger a selection haptic through this wrapper's engine.
    pub fn trigger_selection(&self) {
        self.haptic.selection();
    }

    /// Play a named sound through this wrapper's engine.
    pub fn play_audio(&self, name: &str, volume: f32) {
        self.audio.play_sound(name, volume);
    }
}

impl<V: cvkg_core::View> cvkg_core::View for MicroFeedback<V> {
    type Body = V::Body;

    fn body(self) -> Self::Body {
        self.content.body()
    }

    fn render(&self, renderer: &mut dyn cvkg_core::Renderer, rect: cvkg_core::Rect) {
        self.content.render(renderer, rect);
    }
}

// =============================================================================
// ENGLISH API ALIASES
// Standard names for Norse-named components. These are type aliases that
// point to the canonical (Norse) implementation. Both names are valid.
// See `uiux.md` audit for naming analysis and discoverability metrics.
// =============================================================================

pub type Accordion = SagaAccordion<cvkg_core::AnyView>;
pub type Alert = GjallarAlert;
pub type Analytics = ValkyrieAnalytics;
pub type Avatar = MuninAvatar;
pub type ColorPicker = BifrostColorPicker;
pub type CommandPalette = MimirSpotlight;
pub type CreativeTools = BragiCreative;
pub type Decoder = RunestoneDecoder;
pub type Dialog = GeriDialog<cvkg_core::AnyView>;
pub type HolographicDisplay = HolographicRunestone;
pub type HUD = WyrdHUD;
pub type Indicator = ValkyrieIndicator;
pub type Messenger = RavenMessenger;
pub type Orb = OracleOrb;
pub type Pagination = HringrPagination;
pub type Progress = SkollProgress;
pub type PromptBuilder = PromptForge;
pub type Rating = ValhallaRating;
pub type ScribingNote = ScribingStone;
pub type Sheet = GraniSheet<cvkg_core::AnyView>;
pub type Spinner = HatiSpinner;
pub type Splitter = GjallarSplitter<cvkg_core::AnyView, cvkg_core::AnyView>;
pub type StepIndicator = SleipnirGait;
pub type Tabs = BifrostTabs;
pub type Timeline = UrdrTimeline;
pub type Tooltip = RunicTooltip<cvkg_core::AnyView>;
pub type TreeView = YggdrasilTree;
pub type Well = MimirsWell;
pub type Window = YggdrasilWindow<cvkg_core::AnyView>;
