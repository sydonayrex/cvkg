// =============================================================================
// ARIA PROPERTIES
// =============================================================================

/// Semantic role for assistive technology (WCAG 2.1 §4.1.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AriaRole {
    Alert,
    Alertdialog,
    Article,
    Banner,
    Button,
    Checkbox,
    Columnheader,
    Combobox,
    Complementary,
    Contentinfo,
    Dialog,
    Form,
    Grid,
    Gridcell,
    Heading,
    Img,
    Link,
    List,
    Listbox,
    Listitem,
    Main,
    Menu,
    Menubar,
    Menuitem,
    Menuitemcheckbox,
    Menuitemradio,
    Navigation,
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

