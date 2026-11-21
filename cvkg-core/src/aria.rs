use serde::{Deserialize, Serialize};

// =============================================================================
// ARIA PROPERTIES
// =============================================================================

/// Semantic role for assistive technology (WCAG 2.1 §4.1.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AriaRole {
    Button,
    Checkbox,
    Combobox,
    Dialog,
    Grid,
    Gridcell,
    Heading,
    Image,
    Link,
    Listbox,
    Listitem,
    Menu,
    Menubar,
    Menuitem,
    None,
    Note,
    Option,
    Presentation,
    Progressbar,
    Radio,
    Radiogroup,
    Region,
    Row,
    Rowgroup,
    Rowheader,
    Search,
    Separator,
    Slider,
    Spinbutton,
    Status,
    Switch,
    Tab,
    Table,
    Tablist,
    Tabpanel,
    Textbox,
    Toolbar,
    Tooltip,
    Tree,
    Treeitem,
    Img,
    Navigation,
    List,
    Form,
    Main,
    Banner,
    Contentinfo,
}

/// Accessible properties for a view, describing its semantic role and state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AriaProperties {
    pub role: AriaRole,
    pub label: String,
    pub description: Option<String>,
    pub value: Option<String>,
    pub pressed: Option<bool>,
    pub checked: Option<bool>,
    /// Mirrors the `aria-invalid` state (validation error, not parse failure).
    pub invalid: Option<bool>,
    pub expanded: Option<bool>,
    pub disabled: bool,
    pub hidden: bool,
    pub level: Option<u8>,
    pub shortcut: Option<String>,
    pub focused: bool,
    pub live: Option<String>,
    pub atomic: bool,
}

impl AriaProperties {
    pub fn new(role: AriaRole, label: impl Into<String>) -> Self {
        Self {
            role,
            label: label.into(),
            description: None,
            value: None,
            pressed: None,
            checked: None,
            invalid: None,
            expanded: None,
            disabled: false,
            hidden: false,
            level: None,
            shortcut: None,
            focused: false,
            live: None,
            atomic: false,
        }
    }

    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = Some(v.into());
        self
    }
    pub fn checked(mut self, c: bool) -> Self {
        self.checked = Some(c);
        self
    }
    /// Set the `aria-pressed` state for toggle-style buttons.
    pub fn pressed(mut self, p: bool) -> Self {
        self.pressed = Some(p);
        self
    }
    /// Set the `aria-invalid` state. `None` (the default) means the attribute
    /// is not exposed; `Some(false)` explicitly marks a field as valid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = Some(invalid);
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn expanded(mut self, e: bool) -> Self {
        self.expanded = Some(e);
        self
    }
    pub fn level(mut self, l: u8) -> Self {
        self.level = Some(l.clamp(1, 6));
        self
    }
    pub fn shortcut(mut self, s: impl Into<String>) -> Self {
        self.shortcut = Some(s.into());
        self
    }
    pub fn focused(mut self, f: bool) -> Self {
        self.focused = f;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_state_defaults_to_absent() {
        let props = AriaProperties::new(AriaRole::Textbox, "value");
        assert_eq!(props.invalid, None);
    }

    #[test]
    fn pressed_state_defaults_to_absent_and_can_be_set() {
        let plain = AriaProperties::new(AriaRole::Button, "ok");
        assert_eq!(plain.pressed, None);
        let pressed = AriaProperties::new(AriaRole::Button, "ok").pressed(true);
        assert_eq!(pressed.pressed, Some(true));
    }

    #[test]
    fn invalid_builder_sets_explicit_state() {
        let props = AriaProperties::new(AriaRole::Textbox, "value").invalid(true);
        assert_eq!(props.invalid, Some(true));
        let valid = AriaProperties::new(AriaRole::Textbox, "value").invalid(false);
        assert_eq!(valid.invalid, Some(false));
    }
}
