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
    dead_code,
    clippy::type_complexity,
    clippy::unusual_byte_groupings
)]

//! Built-in component library for CVKG
//!
//! This crate implements standard CVKG components using public CVKG APIs.

// --- Shared Types ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Regular,
    Bold,
    Italic,
}

pub use cvkg_core::Color;

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
    AlertDialog, Dialog, DialogAction, Form, GjallarSplitter, HStack, LazyVStack, Menu,
    NavigationSplitView, NavigationStack, SagaAccordion, ScrollView, Sheet, SheetModifier, TabView,
    Table, VStack,
};
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
    BifrostColorPicker, Button, Checkbox, HringrPagination, Input, Picker, SecureField, Slider,
    Stepper, Textarea, Toggle, ValhallaRating, ValkyrSelect,
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
    AvatarStatus, ChartType, DraumaSkeleton, EikonaAvatar, Gauge, MerkiBadge, MimirsWell, Progress,
    RuneScript, RunicTooltip, SleipnirGait, StatusBar, TelemetryView, UrdrTimeline,
    ValkyrieAnalytics, VölvaScan,
};
pub use window::{GinnungagapWindow, HiminnModal, YggdrasilWindow};
pub use wyrd_hud::WyrdHUD;
pub mod autocomplete;
pub mod bragi_creative;
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
pub use eir_motion::*;
pub use hlin_accessibility::*;
pub use toast::*;
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
