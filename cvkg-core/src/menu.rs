/// Keyboard shortcut modifiers.
#[derive(Debug, Clone, Default)]
pub struct Modifiers {
    /// Command on macOS, Control on Windows/Linux.
    pub cmd: bool,
    /// Shift key.
    pub shift: bool,
    /// Alt/Option key.
    pub alt: bool,
    /// Control key (distinct from cmd on all platforms).
    pub ctrl: bool,
}

/// A keyboard shortcut binding to a menu action.
#[derive(Debug, Clone)]
pub struct KeyboardShortcut {
    /// The key character or name, e.g. `"s"`, `"z"`, `"Return"`.
    pub key: String,
    /// The required modifier combination.
    pub modifiers: Modifiers,
}

impl KeyboardShortcut {
    /// Convenience constructor: cmd (or ctrl on non-macOS) + `key`.
    pub fn cmd(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            modifiers: Modifiers {
                cmd: true,
                ..Default::default()
            },
        }
    }

    /// Convenience constructor: cmd+Shift + `key`.
    pub fn cmd_shift(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            modifiers: Modifiers {
                cmd: true,
                shift: true,
                ..Default::default()
            },
        }
    }
}

/// A single entry in a [`MenuBar`].
///
/// Actions hold a callback that is invoked when the user activates the item
/// (either via the menu UI or via the associated keyboard shortcut).
/// Separators provide visual grouping. Submenus allow hierarchical menus.
pub enum MenuItem {
    /// An activatable menu entry with an optional shortcut and enabled/disabled state.
    Action {
        label: String,
        shortcut: Option<KeyboardShortcut>,
        action: std::sync::Arc<dyn Fn() + Send + Sync>,
        enabled: bool,
    },
    /// A nested submenu.
    Submenu { label: String, items: Vec<MenuItem> },
    /// A visual separator line between groups of items.
    Separator,
}

impl std::fmt::Debug for MenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Action { label, enabled, .. } => f
                .debug_struct("Action")
                .field("label", label)
                .field("enabled", enabled)
                .finish(),
            Self::Submenu { label, items } => f
                .debug_struct("Submenu")
                .field("label", label)
                .field("items", items)
                .finish(),
            Self::Separator => write!(f, "Separator"),
        }
    }
}

/// A top-level menu bar containing [`MenuItem`]s.
///
/// The menu bar is a data model only; rendering it into an OS-native menu is
/// handled by the platform renderer (`cvkg-render-native`).
pub struct MenuBar {
    /// Ordered list of top-level menu items.
    pub items: Vec<MenuItem>,
}

impl MenuBar {
    /// Create an empty menu bar.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Append a menu item to the bar.
    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    /// Build the standard CVKG menu structure with all conventional shortcuts.
    ///
    /// The `cmd` modifier maps to ⌘ on macOS and Ctrl on Windows/Linux -- this
    /// translation is enforced by the renderer, not here.
    ///
    /// Menus included:
    /// - **File**: New, Open, Save, Close
    /// - **Edit**: Undo, Redo, Cut, Copy, Paste, Select All, Find
    /// - **View**: Zoom In, Zoom Out, Fullscreen
    /// - **Window**: Minimize, Zoom, Bring All to Front
    /// - **Help**: Search Help
    #[allow(clippy::too_many_arguments)]
    pub fn standard(
        new_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        open_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        save_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        close_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        quit_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        undo_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        redo_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        cut_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        copy_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        paste_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        select_all_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        find_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
    ) -> Self {
        let mut bar = Self::new();

        // ── File ──────────────────────────────────────────────────────────────
        bar.add_item(MenuItem::Submenu {
            label: "File".to_string(),
            items: vec![
                MenuItem::Action {
                    label: "New".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("n")),
                    action: new_fn,
                    enabled: true,
                },
                MenuItem::Action {
                    label: "Open…".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("o")),
                    action: open_fn,
                    enabled: true,
                },
                MenuItem::Separator,
                MenuItem::Action {
                    label: "Save".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("s")),
                    action: save_fn,
                    enabled: true,
                },
                MenuItem::Separator,
                MenuItem::Action {
                    label: "Close".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("w")),
                    action: close_fn,
                    enabled: true,
                },
                MenuItem::Separator,
                MenuItem::Action {
                    label: "Quit".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("q")),
                    action: quit_fn,
                    enabled: true,
                },
            ],
        });

        // ── Edit ──────────────────────────────────────────────────────────────
        bar.add_item(MenuItem::Submenu {
            label: "Edit".to_string(),
            items: vec![
                MenuItem::Action {
                    label: "Undo".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("z")),
                    action: undo_fn,
                    enabled: true,
                },
                MenuItem::Action {
                    label: "Redo".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd_shift("z")),
                    action: redo_fn,
                    enabled: true,
                },
                MenuItem::Separator,
                MenuItem::Action {
                    label: "Cut".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("x")),
                    action: cut_fn,
                    enabled: true,
                },
                MenuItem::Action {
                    label: "Copy".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("c")),
                    action: copy_fn,
                    enabled: true,
                },
                MenuItem::Action {
                    label: "Paste".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("v")),
                    action: paste_fn,
                    enabled: true,
                },
                MenuItem::Separator,
                MenuItem::Action {
                    label: "Select All".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("a")),
                    action: select_all_fn,
                    enabled: true,
                },
                MenuItem::Separator,
                MenuItem::Action {
                    label: "Find…".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("f")),
                    action: find_fn,
                    enabled: true,
                },
            ],
        });

        // ── View ──────────────────────────────────────────────────────────────
        // View items carry no application-level callbacks at the model layer;
        // zoom and fullscreen are handled by the renderer directly.
        let noop: std::sync::Arc<dyn Fn() + Send + Sync> = std::sync::Arc::new(|| {});
        bar.add_item(MenuItem::Submenu {
            label: "View".to_string(),
            items: vec![
                MenuItem::Action {
                    label: "Zoom In".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("=")),
                    action: noop.clone(),
                    enabled: true,
                },
                MenuItem::Action {
                    label: "Zoom Out".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("-")),
                    action: noop.clone(),
                    enabled: true,
                },
                MenuItem::Separator,
                MenuItem::Action {
                    label: "Toggle Fullscreen".to_string(),
                    shortcut: Some(KeyboardShortcut {
                        key: "f".to_string(),
                        modifiers: Modifiers {
                            ctrl: true,
                            ..Default::default()
                        },
                    }),
                    action: noop.clone(),
                    enabled: true,
                },
            ],
        });

        // ── Window ────────────────────────────────────────────────────────────
        bar.add_item(MenuItem::Submenu {
            label: "Window".to_string(),
            items: vec![
                MenuItem::Action {
                    label: "Minimize".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("m")),
                    action: noop.clone(),
                    enabled: true,
                },
                MenuItem::Action {
                    label: "Zoom".to_string(),
                    shortcut: None,
                    action: noop.clone(),
                    enabled: true,
                },
                MenuItem::Separator,
                MenuItem::Action {
                    label: "Bring All to Front".to_string(),
                    shortcut: None,
                    action: noop.clone(),
                    enabled: true,
                },
            ],
        });

        // ── Help ──────────────────────────────────────────────────────────────
        bar.add_item(MenuItem::Submenu {
            label: "Help".to_string(),
            items: vec![MenuItem::Action {
                label: "Search Help".to_string(),
                shortcut: None,
                action: noop,
                enabled: true,
            }],
        });

        bar
    }
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new()
    }
}
