use cvkg_core::{Event, Never, Rect, Renderer, View};
use std::sync::Arc;

/// Keyboard navigation focus identifier.
pub type FocusId = String;

/// Keyboard navigation handler callback.
pub type KeyHandler = Arc<dyn Fn(&str) + Send + Sync>;

/// A focusable wrapper that adds keyboard navigation to any view.
///
/// Wraps a view with Tab/Arrow key navigation and focus management.
/// Register keyboard handlers to respond to specific keys.
#[derive(Clone)]
pub struct Focusable<V: View> {
    /// The wrapped view content.
    pub content: V,
    /// Unique identifier for focus management.
    pub focus_id: FocusId,
    /// Whether this element can be focused via Tab navigation.
    pub is_focusable: bool,
    /// Callback invoked when a keyboard event is received while focused.
    pub on_key: Option<KeyHandler>,
    /// Callback invoked when this element gains focus.
    pub on_focus: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Callback invoked when this element loses focus.
    pub on_blur: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Tab order index. Lower values are focused first.
    pub tab_index: i32,
}

impl<V: View> Focusable<V> {
    /// Create a new Focusable wrapping the given content.
    pub fn new(focus_id: impl Into<FocusId>, content: V) -> Self {
        Self {
            content,
            focus_id: focus_id.into(),
            is_focusable: true,
            on_key: None,
            on_focus: None,
            on_blur: None,
            tab_index: 0,
        }
    }

    /// Set whether this element is focusable via Tab.
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.is_focusable = focusable;
        self
    }

    /// Set the tab order index.
    pub fn tab_index(mut self, index: i32) -> Self {
        self.tab_index = index;
        self
    }

    /// Set the keyboard event handler.
    pub fn on_key(mut self, handler: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_key = Some(Arc::new(handler));
        self
    }

    /// Set the focus gained callback.
    pub fn on_focus(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_focus = Some(Arc::new(callback));
        self
    }

    /// Set the focus lost callback.
    pub fn on_blur(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_blur = Some(Arc::new(callback));
        self
    }

    /// Convenience: handle Enter key to trigger an action.
    pub fn on_enter(mut self, action: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_key = Some(Arc::new(move |key| {
            if key == "Return" || key == "Enter" {
                action();
            }
        }));
        self
    }

    /// Convenience: handle Escape key to trigger an action.
    pub fn on_escape(mut self, action: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_key = Some(Arc::new(move |key| {
            if key == "Escape" {
                action();
            }
        }));
        self
    }
}

impl<V: View> View for Focusable<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Focusable");
        self.content.render(renderer, rect);

        // Register keydown handler for keyboard events.
        if let Some(on_key) = &self.on_key {
            let on_key = on_key.clone();
            let focus_id = self.focus_id.clone();
            renderer.register_handler(
                "keydown",
                Arc::new(move |event| {
                    if let Event::KeyDown { key, .. } = event {
                        let state = cvkg_core::load_system_state();
                        let focused = state
                            .get_component_state::<String>(focus_id_hash(&focus_id))
                            .map(|v| {
                                let guard = v.read().unwrap_or_else(|e| {
                                    tracing::warn!("Lock poisoned, recovering...");
                                    e.into_inner()
                                });
                                *guard == focus_id
                            })
                            .unwrap_or(false);
                        if focused {
                            on_key(key.as_str());
                        }
                    }
                }),
            );
        }
    }
}

/// Keyboard shortcut definition.
#[derive(Clone)]
pub struct Shortcut {
    /// The key combination (e.g. "Ctrl+S", "Escape", "Tab").
    pub keys: String,
    /// Callback to invoke when the shortcut is triggered.
    pub action: Arc<dyn Fn() + Send + Sync>,
    /// Human-readable description of the shortcut.
    pub description: String,
}

/// Keyboard shortcuts manager.
///
/// Registers global keyboard shortcuts that work regardless of focus.
#[derive(Clone)]
pub struct KeyboardShortcuts {
    shortcuts: Vec<Shortcut>,
}

impl KeyboardShortcuts {
    /// Create a new empty KeyboardShortcuts manager.
    pub fn new() -> Self {
        Self {
            shortcuts: Vec::new(),
        }
    }

    /// Add a keyboard shortcut.
    pub fn shortcut(
        mut self,
        keys: impl Into<String>,
        description: impl Into<String>,
        action: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        self.shortcuts.push(Shortcut {
            keys: keys.into(),
            action: Arc::new(action),
            description: description.into(),
        });
        self
    }

    /// Add a Ctrl+key shortcut.
    pub fn ctrl_key(
        mut self,
        key: impl Into<String>,
        description: impl Into<String>,
        action: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        self.shortcuts.push(Shortcut {
            keys: format!("Ctrl+{}", key.into()),
            action: Arc::new(action),
            description: description.into(),
        });
        self
    }

    /// Get all registered shortcuts.
    pub fn shortcuts(&self) -> &[Shortcut] {
        &self.shortcuts
    }
}

impl Default for KeyboardShortcuts {
    fn default() -> Self {
        Self::new()
    }
}

/// Focus trap: a container that traps keyboard focus within it.
///
/// When active, Tab/Shift+Tab cycle through focusable children only
/// within this container, preventing focus from escaping.
#[derive(Clone)]
pub struct FocusTrap<V: View> {
    /// The content to render inside the trap.
    pub content: V,
    /// Whether the focus trap is active.
    pub is_active: bool,
    /// Callback invoked when Escape is pressed inside the trap.
    pub on_escape: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl<V: View> FocusTrap<V> {
    /// Create a new FocusTrap.
    pub fn new(content: V) -> Self {
        Self {
            content,
            is_active: true,
            on_escape: None,
        }
    }

    /// Set whether the focus trap is active.
    pub fn active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    /// Set the Escape handler.
    pub fn on_escape(mut self, action: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_escape = Some(Arc::new(action));
        self
    }
}

impl<V: View> View for FocusTrap<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "FocusTrap");
        self.content.render(renderer, rect);

        if self.is_active {
            // Register Escape handler to close/dismiss.
            if let Some(on_escape) = &self.on_escape {
                let on_escape = on_escape.clone();
                renderer.register_handler(
                    "keydown",
                    Arc::new(move |event| {
                        if let Event::KeyDown { key, .. } = event
                            && key == "Escape"
                        {
                            on_escape();
                        }
                    }),
                );
            }

            // Register Tab handler to cycle focus within the trap.
            renderer.register_handler(
                "keydown",
                Arc::new(move |event| {
                    if let Event::KeyDown { key, modifiers } = event
                        && key == "Tab"
                    {
                        cycle_focus(!modifiers.shift);
                    }
                }),
            );
        }

        renderer.pop_vnode();
    }
}

/// Cycle focus to the next (forward) or previous (backward) element.
fn cycle_focus(forward: bool) {
    let state = cvkg_core::load_system_state();
    let focus_order = state
        .get_component_state::<Vec<String>>(FOCUS_ORDER_HASH)
        .map(|v| {
            let guard = v.read().unwrap_or_else(|e| {
                tracing::warn!("Lock poisoned, recovering...");
                e.into_inner()
            });
            guard.clone()
        })
        .unwrap_or_default();

    if focus_order.is_empty() {
        return;
    }

    let current = state
        .get_component_state::<String>(CURRENT_FOCUS_HASH)
        .map(|v| {
            let guard = v.read().unwrap_or_else(|e| {
                tracing::warn!("Lock poisoned, recovering...");
                e.into_inner()
            });
            guard.clone()
        })
        .unwrap_or_default();

    let next_idx = if current.is_empty() {
        if forward { 0 } else { focus_order.len() - 1 }
    } else if let Some(idx) = focus_order.iter().position(|id| id == &current) {
        if forward {
            (idx + 1) % focus_order.len()
        } else {
            (idx + focus_order.len() - 1) % focus_order.len()
        }
    } else {
        0
    };

    if let Some(next_id) = focus_order.get(next_idx) {
        cvkg_core::update_system_state(|s| {
            let mut s = s.clone();
            s.set_component_state(CURRENT_FOCUS_HASH, next_id.clone());
            s
        });
    }
}

/// Hash for the focus order list in system state.
pub fn focus_order_hash() -> u64 {
    use std::hash::{Hash, Hasher};
    let mut s = std::collections::hash_map::DefaultHasher::new();
    "focus_order".hash(&mut s);
    s.finish()
}

/// Hash for the current focus ID in system state.
pub fn current_focus_hash() -> u64 {
    use std::hash::{Hash, Hasher};
    let mut s = std::collections::hash_map::DefaultHasher::new();
    "current_focus".hash(&mut s);
    s.finish()
}

/// Hash for a specific focus ID's state.
fn focus_id_hash(id: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut s = std::collections::hash_map::DefaultHasher::new();
    "focus_".hash(&mut s);
    id.hash(&mut s);
    s.finish()
}

/// Constants for system state hashes.
const FOCUS_ORDER_HASH: u64 = 0xC0DE_01;
const CURRENT_FOCUS_HASH: u64 = 0xC0DE_02;

/// Arrow key direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Arrow key handler for directional navigation.
///
/// Wraps a view and handles ArrowUp/ArrowDown/ArrowLeft/ArrowRight keys.
#[derive(Clone)]
pub struct ArrowNav<V: View> {
    /// The wrapped view.
    pub content: V,
    /// Handler for arrow key events.
    pub on_arrow: Arc<dyn Fn(ArrowDirection) + Send + Sync>,
    /// Whether this element must be focused to receive arrow keys.
    pub require_focus: bool,
}

impl<V: View> ArrowNav<V> {
    /// Create a new ArrowNav wrapping the given content.
    pub fn new(content: V, on_arrow: impl Fn(ArrowDirection) + Send + Sync + 'static) -> Self {
        Self {
            content,
            on_arrow: Arc::new(on_arrow),
            require_focus: true,
        }
    }

    /// Set whether focus is required to receive arrow key events.
    pub fn require_focus(mut self, require: bool) -> Self {
        self.require_focus = require;
        self
    }
}

impl<V: View> View for ArrowNav<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ArrowNav");
        self.content.render(renderer, rect);

        let on_arrow = self.on_arrow.clone();
        let require_focus = self.require_focus;

        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    let matches = match key.as_str() {
                        "ArrowUp" => Some(ArrowDirection::Up),
                        "ArrowDown" => Some(ArrowDirection::Down),
                        "ArrowLeft" => Some(ArrowDirection::Left),
                        "ArrowRight" => Some(ArrowDirection::Right),
                        _ => None,
                    };

                    if let Some(dir) = matches {
                        if require_focus {
                            let state = cvkg_core::load_system_state();
                            let focused = state
                                .get_component_state::<bool>(CURRENT_FOCUS_HASH)
                                .and_then(|v| v.read().ok().map(|g| *g))
                                .unwrap_or(false);
                            if !focused {
                                return;
                            }
                        }
                        (on_arrow)(dir);
                    }
                }
            }),
        );

        renderer.pop_vnode();
    }
}
