use std::any::Any;

/// A companion state that should be auto-initialized when a View component
/// is instantiated. Similar to Bevy's `#[require(Component)]`.
///
/// # Contract
/// - `Default::default()` produces a valid initial state.
/// - The state is stored in the VNode and persists across frames.
/// - Must implement `type_name()` for debug/inspector display.
/// - Must be object-safe (dyn compatible) for storage as `Box<dyn Companion>`.
/// - Must implement `as_any()` for downcasting from `Box<dyn Companion>`.
pub trait Companion: Send + Sync + 'static {
    /// Human-readable name for debug/inspector display.
    fn type_name(&self) -> &'static str;

    /// Downcast to concrete type for retrieval from VNode state.
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Standard companion for focusable views.
#[derive(Clone, Debug, Default)]
pub struct FocusableCompanion {
    pub state: FocusState,
    pub tab_index: i32,
}

impl Companion for FocusableCompanion {
    fn type_name(&self) -> &'static str {
        "FocusableCompanion"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl FocusableCompanion {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Standard companion for accessibility properties.
#[derive(Clone, Debug, Default)]
pub struct A11yCompanion {
    pub role: String,
    pub label: String,
    pub description: String,
    pub disabled: bool,
}

impl Companion for A11yCompanion {
    fn type_name(&self) -> &'static str {
        "A11yCompanion"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl A11yCompanion {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_role(mut self, role: &str) -> Self {
        self.role = role.to_string();
        self
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }
}

/// Focus state for interactive elements.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FocusState {
    #[default]
    Unfocused,
    Focused,
    FocusVisible,
}