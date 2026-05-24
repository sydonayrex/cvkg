//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     — State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     — Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     — Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    — Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     — Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     — Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   — Check every tool call / command for progress every 30 seconds.
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
    clippy::unusual_byte_groupings
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
// TYPOGRAPHY SYSTEM — Centralized design tokens for text sizing
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
// SPACING SCALE — Consistent spacing tokens
// =============================================================================

/// Spacing scale for layout consistency.
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 16.0;
pub const SPACE_LG: f32 = 24.0;
pub const SPACE_XL: f32 = 32.0;

// =============================================================================
// BORDER RADIUS SCALE — Consistent corner radii
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
// FOCUS RING SYSTEM — WCAG 2.4.7 compliant focus indicators
// =============================================================================

/// Focus ring width in logical pixels.
pub const FOCUS_RING_WIDTH: f32 = 2.0;

/// Focus ring offset from the element bounds.
pub const FOCUS_RING_OFFSET: f32 = 2.0;

/// Default focus ring color (cyan accent).
pub const FOCUS_RING_COLOR: [f32; 4] = [0.0, 0.8, 1.0, 0.8];

/// Draws a focus ring around the given rectangle.
/// This function should be called by every interactive component when it has focus.
///
/// # Contract
/// - Uses `FOCUS_RING_WIDTH` for stroke width
/// - Uses `FOCUS_RING_OFFSET` to expand outward from the element bounds
/// - Uses `FOCUS_RING_COLOR` or the theme's focus ring color
pub fn draw_focus_ring(renderer: &mut dyn cvkg_core::Renderer, rect: cvkg_core::Rect) {
    let outline_rect = cvkg_core::Rect {
        x: rect.x - FOCUS_RING_OFFSET,
        y: rect.y - FOCUS_RING_OFFSET,
        width: rect.width + FOCUS_RING_OFFSET * 2.0,
        height: rect.height + FOCUS_RING_OFFSET * 2.0,
    };
    renderer.stroke_rounded_rect(outline_rect, RADIUS_SM, FOCUS_RING_COLOR, FOCUS_RING_WIDTH);
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
// BUTTON VARIANTS — Standard button styles
// =============================================================================

/// Button visual variants matching the shadcn/ui pattern.
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
pub mod gpu_charts;
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
pub mod primitive;
pub mod raven_messenger;
pub mod richtext;
pub mod runestone_decoder;
pub mod runestone_editor;
pub mod scribing_stone;
pub mod semantic_memory_explorer;
pub mod shapes;
pub mod shield_wall;
pub mod valkyrie_indicator;
pub mod virtual_list;
pub mod virtual_table;
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
pub mod theme;
pub mod skadi_scripting;
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
pub use command_palette::{BifrostLauncher, MimirSpotlight};
pub use container::{
    GarmAlert, GeriDialog, DialogAction, Form, GjallarSplitter, HStack, LazyVStack, Menu,
    NavigationSplitView, NavigationStack, SagaAccordion, ScrollView, GraniSheet, SheetPosition,
    SheetModifier, TabView, Table, VStack,
};
pub use dropdown_menu::{DropdownItem, DropdownMenu};
pub use data_grid::RunesTable;
pub use devtools::*;
pub use docking_workspace::*;
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
    BifrostColorPicker, Button, Checkbox, Dropdown, HringrPagination, Input, Picker,
    SecureField, Select, Slider, Stepper, Textarea, Toggle, ValhallaRating, GeriTransfer,
};
pub use layer_system::*;
pub use memory::*;
pub use mjolnir_frame::MjolnirFrame;
pub use mjolnir_slider::MjolnirSlider;
pub use multi_agent_orchestrator::*;
pub use navigation::*;
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
pub use virtual_table::*;
pub use visual::{
    AvatarStatus, ChartType, DraumaSkeleton, MuninAvatar, EmptyState, Gauge, MerkiBadge,
    MimirsWell, SkollProgress, RuneScript, RunicTooltip, SleipnirGait, HatiSpinner, SpinnerVariant,
    StatusBar, TelemetryView, UrdrTimeline, ValkyrieAnalytics, VölvaScan, HatiCarousel,
};
pub use window::{GinnungagapWindow, HiminnModal, YggdrasilWindow};
pub use wyrd_hud::WyrdHUD;
pub mod autocomplete;
pub mod bragi_creative;
pub mod combobox;
pub mod new_components;
pub use new_components::*;
pub mod ai_components;
pub use ai_components::*;
pub mod datepicker;
pub mod eir_motion;
pub mod form_validation;
pub mod hlin_accessibility;
pub mod keyboard_nav;
pub mod popover;
pub mod toast;
pub mod tooltip;
pub mod transitions;
pub mod tyr_security;

pub use autocomplete::*;
pub use bragi_creative::*;
pub use combobox::*;
pub use eir_motion::*;
pub use hlin_accessibility::*;
pub use popover::*;
pub use toast::*;
pub use tooltip::*;
pub use transitions::*;
pub use tyr_security::*;
// Re-export layout components
pub use cvkg_layout as layout;

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
