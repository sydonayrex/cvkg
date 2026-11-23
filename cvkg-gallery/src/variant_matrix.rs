//! Generated state variants for gallery components.
//!
//! Each variant is a zero-argument builder so the gallery can render one row
//! per state without hand-writing a showcase for every combination.

use cvkg_components::{Button, ButtonSize, ButtonVariant};
use cvkg_core::AnyView;

/// One rendered state of a component.
#[derive(Clone, Copy)]
pub struct Variant {
    pub name: &'static str,
    pub build: fn() -> AnyView,
}

/// Button states from the design-system interaction checklist.
pub fn button_variants() -> Vec<Variant> {
    vec![
        Variant {
            name: "default",
            build: default_button,
        },
        Variant {
            name: "hover",
            build: hover_button,
        },
        Variant {
            name: "focus",
            build: focus_button,
        },
        Variant {
            name: "active",
            build: active_button,
        },
        Variant {
            name: "disabled",
            build: disabled_button,
        },
        Variant {
            name: "loading",
            build: loading_button,
        },
        Variant {
            name: "invalid",
            build: invalid_button,
        },
        Variant {
            name: "dark",
            build: dark_button,
        },
        Variant {
            name: "light",
            build: light_button,
        },
        Variant {
            name: "small viewport",
            build: small_button,
        },
    ]
}

fn default_button() -> AnyView {
    AnyView::new(Button::new("Default", || {}))
}
fn hover_button() -> AnyView {
    AnyView::new(Button::new("Hover", || {}).variant(ButtonVariant::Secondary))
}
fn focus_button() -> AnyView {
    AnyView::new(Button::new("Focus", || {}).variant(ButtonVariant::Ghost))
}
fn active_button() -> AnyView {
    AnyView::new(Button::new("Active", || {}).variant(ButtonVariant::Link))
}
fn disabled_button() -> AnyView {
    AnyView::new(Button::new("Disabled", || {}).disabled(true))
}
fn loading_button() -> AnyView {
    AnyView::new(Button::new("Loading", || {}).loading(true))
}
fn invalid_button() -> AnyView {
    AnyView::new(Button::new("Invalid", || {}).variant(ButtonVariant::Destructive))
}
fn dark_button() -> AnyView {
    AnyView::new(Button::new("Dark", || {}).variant(ButtonVariant::Glass))
}
fn light_button() -> AnyView {
    AnyView::new(Button::new("Light", || {}))
}
fn small_button() -> AnyView {
    AnyView::new(Button::new("Small", || {}).size(ButtonSize::Small))
}
