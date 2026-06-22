/// Rendering mode for a widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderingMode {
    /// Use native platform controls (buttons, text fields, etc.).
    Native,
    /// Use CVKG's GPU renderer for custom-drawn content.
    Custom,
    /// Hybrid: native container with custom rendering inside.
    Hybrid,
}

/// Translation contract for a CVKG widget to its native representation.
#[derive(Debug, Clone)]
pub struct TranslationContract {
    /// CVKG widget type name.
    pub cvkg_type: &'static str,
    /// Platform-specific type name (e.g., "NSView", "HWND", "GTKWidget").
    pub platform_type: &'static str,
    /// Whether this widget uses native controls or custom rendering.
    pub rendering_mode: RenderingMode,
    /// Whether accessibility is handled natively.
    pub native_accessibility: bool,
}

/// Registry of translation contracts for all widget types.
pub struct TranslationContractRegistry {
    contracts: Vec<TranslationContract>,
}

impl TranslationContractRegistry {
    /// Creates a new translation contract registry.
    pub fn new() -> Self {
        Self {
            contracts: vec![
                TranslationContract {
                    cvkg_type: "Button",
                    platform_type: "NSButton/Button/GTKButton",
                    rendering_mode: RenderingMode::Native,
                    native_accessibility: true,
                },
                TranslationContract {
                    cvkg_type: "TextInput",
                    platform_type: "NSTextField/TextBox/GTKEntry",
                    rendering_mode: RenderingMode::Native,
                    native_accessibility: true,
                },
                TranslationContract {
                    cvkg_type: "Canvas",
                    platform_type: "NSView/HWND/GtkDrawingArea",
                    rendering_mode: RenderingMode::Custom,
                    native_accessibility: false,
                },
                TranslationContract {
                    cvkg_type: "TreeView",
                    platform_type: "NSTableView/TreeView/GTKTreeView",
                    rendering_mode: RenderingMode::Hybrid,
                    native_accessibility: true,
                },
            ],
        }
    }

    /// Look up the contract for a CVKG widget type.
    pub fn find(&self, cvkg_type: &str) -> Option<&TranslationContract> {
        self.contracts.iter().find(|c| c.cvkg_type == cvkg_type)
    }
}

impl Default for TranslationContractRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Bidirectional state synchronization between CVKG and native widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    /// CVKG state drives native widget.
    CvkgToNative,
    /// Native widget state drives CVKG.
    NativeToCvkg,
    /// Both directions.
    Bidirectional,
}

/// State synchronization contract for a widget.
#[derive(Debug, Clone)]
pub struct StateSyncContract {
    /// Widget type name.
    pub widget_type: &'static str,
    /// Synchronization direction.
    pub direction: SyncDirection,
    /// Whether to debounce rapid changes.
    pub debounce: bool,
    /// Debounce interval in milliseconds.
    pub debounce_ms: u64,
}

/// Registry of state synchronization contracts.
pub struct StateSyncRegistry {
    contracts: Vec<StateSyncContract>,
}

impl StateSyncRegistry {
    /// Creates a new state sync registry.
    pub fn new() -> Self {
        Self {
            contracts: vec![
                StateSyncContract {
                    widget_type: "Button",
                    direction: SyncDirection::Bidirectional,
                    debounce: false,
                    debounce_ms: 0,
                },
                StateSyncContract {
                    widget_type: "TextInput",
                    direction: SyncDirection::Bidirectional,
                    debounce: true,
                    debounce_ms: 50,
                },
                StateSyncContract {
                    widget_type: "Slider",
                    direction: SyncDirection::Bidirectional,
                    debounce: true,
                    debounce_ms: 16,
                },
                StateSyncContract {
                    widget_type: "Checkbox",
                    direction: SyncDirection::Bidirectional,
                    debounce: false,
                    debounce_ms: 0,
                },
            ],
        }
    }

    /// Finds a state sync contract for a widget type.
    pub fn find(&self, widget_type: &str) -> Option<&StateSyncContract> {
        self.contracts.iter().find(|c| c.widget_type == widget_type)
    }
}

impl Default for StateSyncRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Widget virtualization configuration for UIs.
#[derive(Debug, Clone, Copy)]
pub struct WidgetVirtualizationConfig {
    /// Number of widgets to render outside the viewport (buffer).
    pub buffer_size: usize,
    /// Whether to recycle widget native handles.
    pub recycle_handles: bool,
    /// Maximum number of active native handles.
    pub max_active_handles: usize,
}

impl Default for WidgetVirtualizationConfig {
    fn default() -> Self {
        Self {
            buffer_size: 5,
            recycle_handles: true,
            max_active_handles: 100,
        }
    }
}

/// Explicit mapping from AccessKit/CVKG role to platform accessibility concepts:
/// macOS (AXRole), Windows (UIA ControlType), and Linux (ATK Role).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticRoleMapping {
    /// The input AccessKit role.
    pub role: accesskit::Role,
    /// macOS AXRole string.
    pub mac_ax_role: &'static str,
    /// Windows UI Automation ControlType constant name or ID string.
    pub win_uia_control_type: &'static str,
    /// Linux ATK Role constant name or ID string.
    pub linux_atk_role: &'static str,
}

/// Registry of semantic accessibility mappings.
pub struct SemanticRoleRegistry {
    mappings: Vec<SemanticRoleMapping>,
}

impl SemanticRoleRegistry {
    /// Creates a new semantic role registry.
    pub fn new() -> Self {
        Self {
            mappings: vec![
                SemanticRoleMapping {
                    role: accesskit::Role::Button,
                    mac_ax_role: "AXButton",
                    win_uia_control_type: "UIA_ButtonControlTypeId",
                    linux_atk_role: "ATK_ROLE_PUSH_BUTTON",
                },
                SemanticRoleMapping {
                    role: accesskit::Role::TextInput,
                    mac_ax_role: "AXTextField",
                    win_uia_control_type: "UIA_EditControlTypeId",
                    linux_atk_role: "ATK_ROLE_ENTRY",
                },
                SemanticRoleMapping {
                    role: accesskit::Role::CheckBox,
                    mac_ax_role: "AXCheckBox",
                    win_uia_control_type: "UIA_CheckBoxControlTypeId",
                    linux_atk_role: "ATK_ROLE_CHECK_BOX",
                },
                SemanticRoleMapping {
                    role: accesskit::Role::Slider,
                    mac_ax_role: "AXSlider",
                    win_uia_control_type: "UIA_SliderControlTypeId",
                    linux_atk_role: "ATK_ROLE_SLIDER",
                },
                SemanticRoleMapping {
                    role: accesskit::Role::Label,
                    mac_ax_role: "AXStaticText",
                    win_uia_control_type: "UIA_TextControlTypeId",
                    linux_atk_role: "ATK_ROLE_LABEL",
                },
            ],
        }
    }

    /// Look up the platform mappings for a given role.
    pub fn find(&self, role: accesskit::Role) -> Option<&SemanticRoleMapping> {
        self.mappings.iter().find(|m| m.role == role)
    }
}

impl Default for SemanticRoleRegistry {
    fn default() -> Self {
        Self::new()
    }
}
