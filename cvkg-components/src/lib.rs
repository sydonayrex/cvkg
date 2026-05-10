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
pub mod calendar;
pub mod card;
pub mod command;
pub mod container;
pub mod data_grid;
pub mod docking_workspace;
pub mod command_palette;
pub mod advanced_forms;
pub mod devtools;
pub mod error;
pub mod grid;
pub mod image;
pub mod interactive;
pub mod memory;
pub mod navigation;
pub mod niflheim_demo;
pub mod primitive;
pub mod richtext;
pub mod virtual_list;
pub mod virtual_table;
pub mod shapes;
pub mod effects;
pub mod hud;
pub mod visual;
pub mod shield_wall;
pub mod mjolnir_slider;
pub mod oracle_orb;
pub mod runestone_editor;
pub mod raven_messenger;
pub mod valkyrie_indicator;
pub mod scribing_stone;
pub mod bifrost_tabs;
pub mod clipped_corner;
pub mod holographic_runestone;
pub mod wyrd_hud;
pub mod mjolnir_frame;
pub mod window;
pub mod runestone_decoder;
pub mod file_tree;
pub mod infinite_canvas;
pub mod node_graph_editor;
pub mod gpu_charts;
pub mod collaboration;
pub mod semantic_memory_explorer;
pub mod multi_agent_orchestrator;
pub mod ai_workflow_builder;

pub mod timeline_editor;
pub mod asset_browser;
pub mod layer_system;
pub mod freyr_inspector;
pub mod njord_theme;
pub mod gullveig_inspector;
pub mod skadi_scripting;
pub mod gerd_telemetry;
pub mod idunn_persistence;

pub use calendar::*;
pub use card::RunesCard;
pub use command::*;
pub use container::{NavigationStack, NavigationSplitView, TabView, Sheet, SheetModifier, Dialog, DialogAction, AlertDialog, Menu, ScrollView, Table, Form, VStack, LazyVStack, HStack, GjallarSplitter, SagaAccordion};
pub use data_grid::RunesTable;
pub use docking_workspace::*;
pub use command_palette::{MimirSpotlight, BifrostLauncher};
pub use advanced_forms::{EikonaForm, Calendar};
pub use devtools::*;
pub use error::*;
pub use grid::*;
pub use shield_wall::ShieldWall;
pub use mjolnir_slider::MjolnirSlider;
pub use oracle_orb::OracleOrb;
pub use runestone_editor::RunestoneEditor;
pub use raven_messenger::RavenMessenger;
pub use valkyrie_indicator::ValkyrieIndicator;
pub use scribing_stone::ScribingStone;
pub use bifrost_tabs::BifrostTabs;
pub use holographic_runestone::HolographicRunestone;
pub use wyrd_hud::WyrdHUD;
pub use mjolnir_frame::MjolnirFrame;
pub use runestone_decoder::RunestoneDecoder;
pub use file_tree::{YggdrasilTree, FileItem, FileKind};
pub use window::{YggdrasilWindow, GinnungagapWindow, HiminnModal};
pub use clipped_corner::*;
pub use image::*;
pub use interactive::{Button, Toggle, Slider, Input, SecureField, Textarea, ValkyrSelect, HringrPagination, ValhallaRating, BifrostColorPicker, Stepper, Picker, Checkbox};
pub use memory::*;
pub use navigation::*;
pub use niflheim_demo::*;
pub use primitive::*;
pub use richtext::*;
pub use virtual_list::*;
pub use virtual_table::*;
pub use shapes::*;
pub use effects::*;
pub use hud::*;
pub use hud::{Vegvísir, TacticalGauge, GjallarAlert, AlertKind};
pub use visual::{Progress, Gauge, StatusBar, ValkyrieAnalytics, TelemetryView, MimirsWell, RuneScript, SleipnirGait, VölvaScan, RunicTooltip, DraumaSkeleton, UrdrTimeline, EikonaAvatar, AvatarStatus, MerkiBadge};
pub use collaboration::*;
pub use semantic_memory_explorer::*;
pub use multi_agent_orchestrator::*;
pub use ai_workflow_builder::*;
pub use timeline_editor::*;
pub use asset_browser::*;
pub use layer_system::*;
pub use freyr_inspector::*;
pub use njord_theme::*;
pub use gullveig_inspector::*;
pub use skadi_scripting::*;
pub use gerd_telemetry::*;
pub use idunn_persistence::*;
pub mod bragi_creative;
pub mod hlin_accessibility;
pub mod eir_motion;
pub mod tyr_security;

pub use bragi_creative::*;
pub use hlin_accessibility::*;
pub use eir_motion::*;
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
