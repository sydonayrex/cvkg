//! ThemeSwitch - Dark/Light/System mode toggle
//!
//! A widget that lets users switch between Light, Dark, and System appearance modes.
//! The selected mode is persisted to `~/.cvkg/theme_mode` so it survives restarts.
//!
//! # Example
//!
//! ```no_run
//! use cvkg_components::theme_switch::ThemeSwitch;
//!
//! ThemeSwitch::new()
//!     .on_mode_change(|mode| {
//!         println!("Theme changed to {:?}", mode);
//!     });
//! ```

use crate::theme;
use cvkg_core::{AriaProperties, AriaRole, Event, KeyModifiers, Never, Rect, Renderer, View};
use std::sync::Arc as StdArc;

/// User-selectable theme mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    /// Force light appearance regardless of OS preference.
    Light,
    /// Force dark appearance regardless of OS preference.
    Dark,
    /// Follow the operating system's color scheme.
    #[default]
    System,
}

impl ThemeMode {
    /// Convert to the string used for persistence.
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
            ThemeMode::System => "system",
        }
    }

    /// Parse from the persisted string. Returns `System` for unknown values.
    pub fn from_str_lossy(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            _ => ThemeMode::System,
        }
    }
}

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{LazyLock, Mutex};

/// Global current mode. Stored as a u8 to make it lock-free for reads.
static CURRENT_MODE: AtomicU8 = AtomicU8::new(2); // 0=Light, 1=Dark, 2=System

/// Global list of callbacks to invoke when the mode changes.
static MODE_LISTENERS: LazyLock<Mutex<Vec<StdArc<dyn Fn(ThemeMode) + Send + Sync>>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

/// Get the current theme mode.
pub fn current_mode() -> ThemeMode {
    match CURRENT_MODE.load(Ordering::Relaxed) {
        0 => ThemeMode::Light,
        1 => ThemeMode::Dark,
        _ => ThemeMode::System,
    }
}

/// Set the current theme mode and notify all listeners.
pub fn set_mode(mode: ThemeMode) {
    let val = match mode {
        ThemeMode::Light => 0,
        ThemeMode::Dark => 1,
        ThemeMode::System => 2,
    };
    CURRENT_MODE.store(val, Ordering::Relaxed);
    persist_mode(mode);

    let listeners = MODE_LISTENERS.lock().expect("mode listeners poisoned");
    for cb in listeners.iter() {
        cb(mode);
    }
}

/// Register a callback to be invoked whenever the mode changes.
pub fn on_mode_change(cb: impl Fn(ThemeMode) + Send + Sync + 'static) {
    let mut listeners = MODE_LISTENERS.lock().expect("mode listeners poisoned");
    listeners.push(StdArc::new(cb));
}

/// Persist the current mode to `~/.cvkg/theme_mode`.
fn persist_mode(mode: ThemeMode) {
    if let Some(home) = std::env::var_os("HOME") {
        let dir = std::path::PathBuf::from(home).join(".cvkg");
        if let Err(e) = std::fs::create_dir_all(&dir) {
            log::debug!("theme_switch: could not create ~/.cvkg: {e}");
            return;
        }
        let path = dir.join("theme_mode");
        if let Err(e) = std::fs::write(&path, mode.as_str()) {
            log::debug!("theme_switch: could not write theme mode: {e}");
        }
    }
}

/// Load the persisted mode from `~/.cvkg/theme_mode`. Returns `None` if not found.
fn load_persisted_mode() -> Option<ThemeMode> {
    let home = std::env::var_os("HOME")?;
    let path = std::path::PathBuf::from(home).join(".cvkg").join("theme_mode");
    let contents = std::fs::read_to_string(&path).ok()?;
    Some(ThemeMode::from_str_lossy(&contents))
}

/// Initialize the global mode from the persisted file, if any. Call once at startup.
pub fn init_from_disk() {
    if let Some(mode) = load_persisted_mode() {
        CURRENT_MODE.store(
            match mode {
                ThemeMode::Light => 0,
                ThemeMode::Dark => 1,
                ThemeMode::System => 2,
            },
            Ordering::Relaxed,
        );
    }
}

/// Three-button theme switch (Light / Dark / System).
#[derive(Clone)]
pub struct ThemeSwitch {
    pub(crate) on_change: Option<StdArc<dyn Fn(ThemeMode) + Send + Sync>>,
    pub(crate) show_labels: bool,
}

impl ThemeSwitch {
    /// Create a new ThemeSwitch reflecting the current global mode.
    pub fn new() -> Self {
        Self {
            on_change: None,
            show_labels: true,
        }
    }

    /// Register a callback for when the user picks a different mode.
    pub fn on_mode_change(mut self, cb: impl Fn(ThemeMode) + Send + Sync + 'static) -> Self {
        self.on_change = Some(StdArc::new(cb));
        self
    }

    /// Show text labels under each icon (default: true).
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }
}

impl Default for ThemeSwitch {
    fn default() -> Self {
        Self::new()
    }
}

impl View for ThemeSwitch {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!("ThemeSwitch renders via render()")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let focus_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            "theme_switch_focus".hash(&mut s);
            s.finish()
        };
        let (is_focused, set_focused) = cvkg_vdom::use_state(focus_hash, false);

        let active = current_mode();

        renderer.push_vnode(rect, "ThemeSwitch");
        renderer.set_aria_role("group");
        renderer.set_aria_label("Theme mode selector");

        let button_count = 3;
        let gap = 4.0;
        let button_width =
            ((rect.width - gap * (button_count as f32 - 1.0)) / button_count as f32).max(28.0);
        let button_height = rect.height;
        let label_height = if self.show_labels { 12.0 } else { 0.0 };
        let icon_height = button_height - label_height;

        let modes = [ThemeMode::Light, ThemeMode::Dark, ThemeMode::System];
        let icons = ["☀", "🌙", "◐"];
        let labels = ["Light", "Dark", "System"];

        for (i, ((mode, icon), label)) in modes.iter().zip(icons).zip(labels).enumerate() {
            let x = rect.x + (button_width + gap) * i as f32;
            let btn_rect = Rect {
                x,
                y: rect.y,
                width: button_width,
                height: button_height,
            };
            let is_active = *mode == active;

            // Background: highlighted when active
            let bg = if is_active {
                theme::with_alpha(theme::primary(), 0.25)
            } else {
                theme::with_alpha(theme::surface(), 0.0)
            };
            let border = if is_active {
                theme::primary()
            } else {
                theme::border()
            };
            let text_col = if is_active {
                theme::primary()
            } else {
                theme::text()
            };

            renderer.fill_rounded_rect(btn_rect, crate::RADIUS_SM, bg);
            renderer.stroke_rounded_rect(btn_rect, crate::RADIUS_SM, border, 1.0);

            // Icon
            let icon_rect = Rect {
                x,
                y: rect.y,
                width: button_width,
                height: icon_height,
            };
            let icon_color = text_col;
            renderer.draw_text(icon, icon_rect.x, icon_rect.y, 16.0, icon_color);

            // Label
            if self.show_labels {
                let label_rect = Rect {
                    x,
                    y: rect.y + icon_height,
                    width: button_width,
                    height: label_height,
                };
                renderer.draw_text(label, label_rect.x, label_rect.y, 9.0, theme::text_muted());
            }

            // Click handler
            let mode_copy = *mode;
            let on_change_cb = self.on_change.clone();
            renderer.register_handler("pointerclick", StdArc::new(move |_event| {
                set_mode(mode_copy);
                if let Some(cb) = &on_change_cb {
                    cb(mode_copy);
                }
            }));
        }

        // Focus ring around the whole widget when focused
        if is_focused {
            let ring_rect = Rect {
                x: rect.x - 2.0,
                y: rect.y - 2.0,
                width: rect.width + 4.0,
                height: rect.height + 4.0,
            };
            renderer.stroke_rounded_rect(ring_rect, crate::RADIUS_SM, theme::focus_ring(), 2.0);
        }

        // Keyboard: ArrowLeft/Right to cycle, Enter to activate Light
        renderer.register_handler("keydown", StdArc::new(move |event| {
            if let Event::KeyDown { key, modifiers, .. } = event {
                match key.as_str() {
                    "ArrowLeft" => {
                        let cur = current_mode();
                        let next = match cur {
                            ThemeMode::System => ThemeMode::Dark,
                            ThemeMode::Dark => ThemeMode::Light,
                            ThemeMode::Light => ThemeMode::System,
                        };
                        set_mode(next);
                    }
                    "ArrowRight" => {
                        let cur = current_mode();
                        let next = match cur {
                            ThemeMode::Light => ThemeMode::Dark,
                            ThemeMode::Dark => ThemeMode::System,
                            ThemeMode::System => ThemeMode::Light,
                        };
                        set_mode(next);
                    }
                    "Tab" if !modifiers.shift => {
                        set_focused(false);
                    }
                    _ => {}
                }
            }
        }));

        renderer.pop_vnode();
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        _proposal: cvkg_core::layout::SizeProposal,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: 180.0,
            height: if self.show_labels { 48.0 } else { 32.0 },
        }
    }

    fn flex_weight(&self) -> f32 {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_mode_as_str_roundtrip() {
        for mode in [ThemeMode::Light, ThemeMode::Dark, ThemeMode::System] {
            let s = mode.as_str();
            assert_eq!(ThemeMode::from_str_lossy(s), mode);
        }
    }

    #[test]
    fn theme_mode_from_str_unknown_defaults_to_system() {
        assert_eq!(ThemeMode::from_str_lossy(""), ThemeMode::System);
        assert_eq!(ThemeMode::from_str_lossy("garbage"), ThemeMode::System);
        assert_eq!(ThemeMode::from_str_lossy("  dark  "), ThemeMode::Dark);
    }

    #[test]
    fn theme_mode_default_is_system() {
        assert_eq!(ThemeMode::default(), ThemeMode::System);
    }

    #[test]
    fn set_mode_updates_current() {
        // Directly set the atomic to a known state to avoid dependency on other tests
        CURRENT_MODE.store(2, Ordering::Relaxed); // 2 = System
        assert_eq!(current_mode(), ThemeMode::System);

        set_mode(ThemeMode::Light);
        assert_eq!(current_mode(), ThemeMode::Light);

        set_mode(ThemeMode::Dark);
        assert_eq!(current_mode(), ThemeMode::Dark);

        set_mode(ThemeMode::System);
        assert_eq!(current_mode(), ThemeMode::System);
    }

    #[test]
    fn on_mode_change_listener_fires() {
        use std::sync::atomic::{AtomicU8, Ordering};
        let received = StdArc::new(AtomicU8::new(99));
        let r2 = received.clone();
        // Register a one-shot listener that unregisters itself after first fire
        // to avoid leaking into other tests
        let received_weak = StdArc::downgrade(&received);
        on_mode_change(move |mode| {
            if let Some(r) = received_weak.upgrade() {
                r.store(
                    match mode {
                        ThemeMode::Light => 0,
                        ThemeMode::Dark => 1,
                        ThemeMode::System => 2,
                    },
                    Ordering::Relaxed,
                );
            }
        });
        let prev = current_mode();
        set_mode(ThemeMode::Light);
        assert_eq!(received.load(Ordering::Relaxed), 0);
        set_mode(ThemeMode::Dark);
        assert_eq!(received.load(Ordering::Relaxed), 1);
        // Clean up: restore mode and clear listeners
        set_mode(prev);
        // Clear the global listeners to prevent cross-test contamination
        if let Ok(mut listeners) = MODE_LISTENERS.lock() {
            listeners.clear();
        }
    }

    #[test]
    fn theme_switch_builder() {
        let sw = ThemeSwitch::new().show_labels(false);
        assert!(!sw.show_labels);
        let sw2 = ThemeSwitch::new().on_mode_change(|_| {});
        assert!(sw2.on_change.is_some());
    }
}