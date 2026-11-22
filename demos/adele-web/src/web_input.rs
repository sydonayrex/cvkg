//! DOM input, hit-testing and declarative shortcuts for the canvas demo (P1-5).
//!
//! The web demo renders into a single `<canvas>`. This module gives it a small
//! but real interaction model: focusable regions, pointer hit-testing, a text
//! caret with editing commands, and shortcuts authored once with the shared
//! `cvkg_core::KeyShortcut` API. State is exposed for browser tests and key
//! activity is announced through the P1-3 live region.
//!
//! It is intentionally small: a fixed region table (no geometry-aware tree),
//! a single editable region, and no rendered caret yet.

use std::cell::RefCell;

/// A focusable canvas region with a fixed rectangle (canvas-local pixels).
#[derive(Debug, Clone, Copy)]
pub struct Region {
    /// Stable id used in announcements and tests.
    pub id: u64,
    /// Human-readable accessible name.
    pub name: &'static str,
    /// Region bounds in the 800×600 demo frame.
    pub rect: (f32, f32, f32, f32),
}

/// Regions matching the panels painted by `web_render`.
pub const REGIONS: [Region; 3] = [
    Region {
        id: 1,
        name: "Sidebar",
        rect: (24.0, 24.0, 220.0, 552.0),
    },
    Region {
        id: 2,
        name: "Search field",
        rect: (268.0, 24.0, 508.0, 150.0),
    },
    Region {
        id: 3,
        name: "Detail panel",
        rect: (268.0, 190.0, 508.0, 150.0),
    },
];

/// Shortcut table: authored once, matched with platform-normalized modifiers.
const SHORTCUTS: [(&str, &str); 3] = [
    ("Ctrl+K", "focus_search"),
    ("Escape", "clear_text"),
    ("Tab", "next_region"),
];

/// Observable input state for tests and debugging.
#[derive(Debug, Default, Clone)]
pub struct InputState {
    /// Whether the canvas currently has DOM focus.
    pub focused: bool,
    /// Last normalized key name (`a`, `ArrowDown`, `Escape`, ...).
    pub last_key: String,
    /// Human-readable modifier summary, e.g. `ctrl+shift`.
    pub modifiers: String,
    /// Last normalized shortcut string, if parsing succeeded.
    pub last_shortcut: String,
    /// Last pointer position in canvas-local pixels.
    pub pointer: (f32, f32),
    /// Index into [`REGIONS`] for the focused region.
    pub focused_region: usize,
    /// Text buffer of the editable region.
    pub text: String,
    /// Caret position as a byte index into `text`.
    pub caret: usize,
    /// Last action taken (`none`, `focus_search`, `insert`, ...).
    pub last_action: String,
}

thread_local! {
    static INPUT_STATE: RefCell<InputState> = RefCell::new(InputState::default());
}

fn modifier_label(mods: &cvkg_core::KeyModifiers) -> String {
    let mut parts = Vec::new();
    if mods.ctrl {
        parts.push("ctrl");
    }
    if mods.alt {
        parts.push("alt");
    }
    if mods.shift {
        parts.push("shift");
    }
    if mods.meta {
        parts.push("meta");
    }
    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join("+")
    }
}

/// Hit-test a canvas-local point against [`REGIONS`].
pub fn hit_test(x: f32, y: f32) -> Option<usize> {
    REGIONS.iter().position(|r| {
        let (rx, ry, rw, rh) = r.rect;
        x >= rx && x <= rx + rw && y >= ry && y <= ry + rh
    })
}

#[cfg(test)]
pub(crate) fn set_state_for_tests(state: InputState) {
    INPUT_STATE.with(|slot| *slot.borrow_mut() = state);
}

/// Snapshot of the current input state.
pub fn current_state() -> InputState {
    INPUT_STATE.with(|slot| slot.borrow().clone())
}

/// Return the declarative action bound to a key/modifier pair, if any.
pub fn match_shortcut(key: &str, mods: &cvkg_core::KeyModifiers) -> Option<&'static str> {
    SHORTCUTS.iter().find_map(|(binding, action)| {
        cvkg_core::KeyShortcut::parse(binding)
            .ok()
            .filter(|shortcut| shortcut.matches_on(key, *mods, cvkg_core::ShortcutPlatform::Other))
            .map(|_| *action)
    })
}

fn apply_action(action: &str, state: &mut InputState) {
    state.last_action = action.to_string();
    match action {
        "focus_search" => {
            state.focused_region = 1;
            state.caret = state.text.len();
        }
        "clear_text" => {
            state.text.clear();
            state.caret = 0;
        }
        "next_region" => {
            state.focused_region = (state.focused_region + 1) % REGIONS.len();
        }
        _ => {}
    }
}

fn edit_text(key: &str, state: &mut InputState) {
    match key {
        "Backspace" | "Back" => {
            if state.caret > 0 {
                let previous = state.text[..state.caret]
                    .char_indices()
                    .next_back()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                state.text.replace_range(previous..state.caret, "");
                state.caret = previous;
            }
            state.last_action = "delete_backward".to_string();
        }
        "Delete" => {
            if state.caret < state.text.len() {
                let next = state.text[state.caret..]
                    .char_indices()
                    .nth(1)
                    .map(|(i, _)| state.caret + i)
                    .unwrap_or(state.text.len());
                state.text.replace_range(state.caret..next, "");
            }
            state.last_action = "delete_forward".to_string();
        }
        "ArrowLeft" | "Left" => {
            state.caret = state.text[..state.caret]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            state.last_action = "caret_left".to_string();
        }
        "ArrowRight" | "Right" => {
            state.caret = state.text[state.caret..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| state.caret + i)
                .unwrap_or(state.text.len());
            state.last_action = "caret_right".to_string();
        }
        "Home" => {
            state.caret = 0;
            state.last_action = "caret_home".to_string();
        }
        "End" => {
            state.caret = state.text.len();
            state.last_action = "caret_end".to_string();
        }
        value if value.chars().count() == 1 => {
            state.text.insert_str(state.caret, value);
            state.caret += value.len();
            state.last_action = "insert".to_string();
        }
        _ => {
            state.last_action = "ignored".to_string();
        }
    }
}

/// Install keyboard/pointer listeners on the canvas.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn install_input_handlers(canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    let document = web_sys::window()
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("no window"))?
        .document()
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("no document"))?;
    let canvas = document
        .get_element_by_id(canvas_id)
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("canvas missing"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| wasm_bindgen::JsValue::from_str("not a canvas"))?;

    canvas.set_attribute("tabindex", "0")?;
    canvas.set_attribute("aria-label", "CVKG design system explorer canvas")?;

    let focus = Closure::<dyn FnMut()>::new(move || {
        INPUT_STATE.with(|s| s.borrow_mut().focused = true);
        let _ = crate::announce_to_a11y("Canvas focused");
    });
    canvas.add_event_listener_with_callback("focus", focus.as_ref().unchecked_ref())?;
    focus.forget();

    let blur = Closure::<dyn FnMut()>::new(move || {
        INPUT_STATE.with(|s| s.borrow_mut().focused = false);
    });
    canvas.add_event_listener_with_callback("blur", blur.as_ref().unchecked_ref())?;
    blur.forget();

    let keydown =
        Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
            let raw = event.key();
            let key = cvkg_core::normalize_key_name(&raw).unwrap_or(raw);
            let mods = cvkg_core::KeyModifiers {
                shift: event.shift_key(),
                ctrl: event.ctrl_key(),
                alt: event.alt_key(),
                meta: event.meta_key(),
            };
            let action = match_shortcut(&key, &mods).map(str::to_string);

            let announcement = INPUT_STATE.with(|slot| {
                let mut state = slot.borrow_mut();
                state.last_key = key.clone();
                state.modifiers = modifier_label(&mods);
                state.last_shortcut = if action.is_some() {
                    format!("{}+{key}", modifier_label(&mods))
                } else {
                    String::new()
                };
                if let Some(action) = &action {
                    apply_action(action, &mut state);
                } else if state.focused_region == 1 {
                    edit_text(&key, &mut state);
                } else {
                    state.last_action = "ignored".to_string();
                }
                format!("{} {}", state.last_action, state.text)
            });
            log::info!("[web-input] keydown key={key} action={announcement}");
            let _ = crate::announce_to_a11y(&announcement);
            crate::schedule_render();
            if key == "Tab" || key == "Escape" {
                return;
            }
            event.prevent_default();
        });
    canvas.add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())?;
    keydown.forget();

    let pointer =
        Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
            let (x, y) = (event.offset_x() as f32, event.offset_y() as f32);
            let region = hit_test(x, y);
            let announcement = INPUT_STATE.with(|slot| {
                let mut state = slot.borrow_mut();
                state.pointer = (x, y);
                if let Some(index) = region {
                    state.focused_region = index;
                    REGIONS[index].name.to_string()
                } else {
                    "Canvas".to_string()
                }
            });
            let _ = crate::announce_to_a11y(&format!("Focused {announcement}"));
            crate::schedule_render();
        });
    canvas.add_event_listener_with_callback("pointerdown", pointer.as_ref().unchecked_ref())?;
    pointer.forget();

    Ok(())
}

/// Non-wasm stub with the same public surface.
#[cfg(not(target_arch = "wasm32"))]
pub fn install_input_handlers(_canvas_id: &str) -> Result<(), String> {
    Err("DOM input is only available on wasm32".to_string())
}

/// Apply text committed through the offscreen DOM text proxy. Keeps the
/// canvas text buffer (and therefore the rendered caret/text) in sync with
/// IME and assistive-technology input.
pub fn apply_proxy_text(value: &str) {
    INPUT_STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.text = value.to_string();
        state.caret = value.chars().count();
        state.last_action = "proxy_input".to_string();
    });
}

/// Snapshot of the current input state as a JSON string (for tests/debugging).
pub fn input_state_json() -> String {
    current_state().to_json()
}

impl InputState {
    /// Serialize to a small JSON object for browser tests.
    pub fn to_json(&self) -> String {
        format!(
            "{{\"focused\":{},\"last_key\":\"{}\",\"modifiers\":\"{}\",\"last_shortcut\":\"{}\",\"pointer\":[{},{}],\"focused_region\":{},\"text\":\"{}\",\"caret\":{},\"last_action\":\"{}\"}}",
            self.focused,
            escape(&self.last_key),
            escape(&self.modifiers),
            escape(&self.last_shortcut),
            self.pointer.0,
            self.pointer.1,
            self.focused_region,
            escape(&self.text),
            self.caret,
            escape(&self.last_action),
        )
    }
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// wasm-bindgen export used by browser tests to read the input snapshot.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn input_state_json_js() -> String {
    input_state_json()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_text_updates_canvas_state() {
        super::apply_proxy_text("hello");
        let state = super::INPUT_STATE.with(|slot| slot.borrow().clone());
        assert_eq!(state.text, "hello");
        assert_eq!(state.caret, 5);
        assert_eq!(state.last_action, "proxy_input");
    }

    #[test]
    fn modifier_labels_are_stable() {
        assert_eq!(modifier_label(&cvkg_core::KeyModifiers::default()), "none");
        let mods = cvkg_core::KeyModifiers {
            ctrl: true,
            shift: true,
            ..Default::default()
        };
        assert_eq!(modifier_label(&mods), "ctrl+shift");
    }

    #[test]
    fn json_snapshot_is_parseable_shape() {
        let json = input_state_json();
        assert!(json.starts_with('{') && json.ends_with('}'));
        assert!(json.contains("\"caret\""));
    }

    #[test]
    fn hit_test_finds_the_expected_regions() {
        assert_eq!(hit_test(100.0, 100.0), Some(0));
        assert_eq!(hit_test(400.0, 100.0), Some(1));
        assert_eq!(hit_test(400.0, 250.0), Some(2));
        assert_eq!(hit_test(790.0, 590.0), None);
    }

    #[test]
    fn declarative_shortcuts_match_native_and_web_modifiers() {
        let ctrl = cvkg_core::KeyModifiers {
            ctrl: true,
            ..Default::default()
        };
        let meta = cvkg_core::KeyModifiers {
            meta: true,
            ..Default::default()
        };
        assert_eq!(match_shortcut("k", &ctrl), Some("focus_search"));
        // Authoring uses Ctrl as primary: on non-Apple platforms Meta must not
        // silently stand in for it.
        assert_eq!(match_shortcut("k", &meta), None);
        let parsed = cvkg_core::KeyShortcut::parse("Ctrl+K").expect("valid shortcut");
        assert!(parsed.matches_on("k", meta, cvkg_core::ShortcutPlatform::Apple));
        assert_eq!(
            match_shortcut("Escape", &cvkg_core::KeyModifiers::default()),
            Some("clear_text")
        );
        assert_eq!(
            match_shortcut("a", &cvkg_core::KeyModifiers::default()),
            None
        );
    }

    #[test]
    fn caret_editing_commands_are_bounded() {
        let mut state = InputState {
            text: "abc".to_string(),
            caret: 3,
            ..Default::default()
        };
        edit_text("Backspace", &mut state);
        assert_eq!((state.text.as_str(), state.caret), ("ab", 2));
        edit_text("ArrowLeft", &mut state);
        assert_eq!(state.caret, 1);
        edit_text("X", &mut state);
        assert_eq!((state.text.as_str(), state.caret), ("aXb", 2));
        state.caret = 0;
        edit_text("Backspace", &mut state);
        assert_eq!((state.text.as_str(), state.caret), ("aXb", 0));
    }
}
