//! CVKG prelude: Standard English names for all common components.
//!
//! This module re-exports the most common components under standard English names,
//! enabling AI agents and developers to discover components without knowledge of
//! the Norse naming convention.
//!
//! Both names work: `cvkg::prelude::Tabs` and `cvkg_components::bifrost_tabs::BifrostTabs`
//! resolve to the same type.

// === Layout primitives ===
pub use crate::container::flex::FlexBox;
pub use crate::container::modal::GeriDialog as Dialog;
pub use crate::container::scroll::ScrollView;
pub use crate::container::stacks::HStack;
pub use crate::container::stacks::LazyVStack;
pub use crate::container::stacks::VStack;
pub use crate::grid::Grid;
pub use crate::responsive::{Breakpoint, Responsive};

// === Interactive components ===
pub use crate::interactive::button::Button;
pub use crate::interactive::button::Slider;
pub use crate::interactive::checkbox::Checkbox;
pub use crate::interactive::input::Input;
pub use crate::interactive::select::Select;
pub use crate::interactive::textarea::Textarea;
pub use crate::primitive::Text;

// === Layout primitives (additional) ===
pub use crate::layout_primitives::{
    AspectRatio, Group, GroupBox, LazyHGrid, LazyHStack, LazyVGrid, Resizable, Separator, ZStack,
};

// === Norse-named component aliases (English names) ===
pub use crate::advanced_forms::Calendar;
pub use crate::bifrost_tabs::BifrostTabs as Tabs;
pub use crate::card::RunesCard as Card;
pub use crate::data_grid::RunesTable as DataGrid;
pub use crate::datepicker::DatePicker;
pub use crate::docking_workspace::DockingWorkspace;
pub use crate::hover_card::HoverCard;
pub use crate::interactive::hringrpagination::HringrPagination as Pagination;
pub use crate::interactive::hrungnirsegmented::HrungnirSegmented as SegmentedControl;
pub use crate::mjolnir_frame::MjolnirFrame as Frame;
pub use crate::popover::Popover;
pub use crate::radial_menu::RadialMenu;
pub use crate::toast::ToastManager as Toast;
pub use crate::tree_view::RichTreeView as TreeView;

// === English aliases from lib.rs ===
pub use crate::{
    AccessibilityTree, Accordion, Alert, Analytics, Avatar, CodeEditor, ColorPicker,
    CommandPalette, CreativeTools, Decoder, DevToolsInspector, HUD, HolographicDisplay, Indicator,
    Inspector, Markdown, Messenger, Orb, Persistence, Progress, PromptBuilder, Rating,
    ScribingNote, Scripting, SecurityGate, Sheet, Spinner, Splitter, StepIndicator, Telemetry,
    ThemeConfig, Timeline, Tooltip, Well, Window,
};

// === Landing page components ===
pub use crate::{
    FeatureGrid, FeatureItem, Hero, PricingCard, PricingTable, TestimonialCard, TestimonialItem,
};

// === P4.2 Motion preset library ===
pub use crate::{Motion, MotionPreset};

// === P4.3 Skeleton loading component ===
pub use crate::Skeleton;

// === P5.1 Game UI primitives ===
pub use crate::{DPadControl, DPadDirection, HealthBar, MapMarker, MiniMap};

// === P1.2 Integrated form validation ===
pub use crate::{Form, FormBinder, FormBinding, FormField, ValidationRule};
