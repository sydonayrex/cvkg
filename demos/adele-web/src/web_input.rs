//! Minimal DOM input bridge for the canvas demo (P1-5).
//!
//! The web demo renders into a single `<canvas>`; without DOM listeners it has
//! no keyboard focus or pointer semantics. This module installs a small,
//! honest input layer that normalizes browser events through the shared
//! `cvkg_core` shortcut API, exposes the resulting state for verification, and
//! announces focus/key activity through the P1-3 accessibility bridge.
//!
//! It is deliberately small: two mirrored nodes, no hit-testing tree, no IME
//! rendering beyond the existing text proxy, and no frame loop.

use std::cell::RefCell;

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
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static INPUT_STATE: RefCell<InputState> = RefCell::new(InputState::default());
}

#[cfg(not(target_arch = "wasm32"))]
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

/// Install keyboard/pointer listeners on the canvas.
///
/// Returns a descriptive error instead of panicking when the canvas is missing
/// or a DOM operation fails.
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

    let focus = {
        let canvas = canvas.clone();
        Closure::<dyn FnMut()>::new(move || {
            INPUT_STATE.with(|s| s.borrow_mut().focused = true);
            let _ = crate::announce_to_a11y("Canvas focused");
            let _ = &canvas;
        })
    };
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
            let shortcut = if key.is_empty() {
                String::new()
            } else {
                let prefix = modifier_label(&mods);
                let text = if prefix == "none" {
                    key.clone()
                } else {
                    format!("{prefix}+{key}")
                };
                match cvkg_core::KeyShortcut::parse(&text) {
                    Ok(parsed) => parsed.key,
                    Err(_) => text,
                }
            };
            INPUT_STATE.with(|slot| {
                let mut state = slot.borrow_mut();
                state.last_key = key.clone();
                state.modifiers = modifier_label(&mods);
                state.last_shortcut = shortcut.clone();
            });
            log::info!(
                "[web-input] keydown key={key} mods={}",
                modifier_label(&mods)
            );
            let _ = crate::announce_to_a11y(&format!("Key {key}"));
            // Let the browser use shortcuts it owns (Tab, etc.).
            if key == "Tab" || key == "Escape" {
                return;
            }
            event.prevent_default();
        });
    canvas.add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())?;
    keydown.forget();

    let pointer =
        Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
            INPUT_STATE.with(|slot| {
                let mut state = slot.borrow_mut();
                state.pointer = (event.offset_x() as f32, event.offset_y() as f32);
            });
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

/// Snapshot of the current input state as a JSON string (for tests/debugging).
pub fn input_state_json() -> String {
    INPUT_STATE.with(|slot| {
        let s = slot.borrow();
        format!(
            "{{\"focused\":{},\"last_key\":\"{}\",\"modifiers\":\"{}\",\"last_shortcut\":\"{}\",\"pointer\":[{},{}]}}",
            s.focused,
            escape(&s.last_key),
            escape(&s.modifiers),
            escape(&s.last_shortcut),
            s.pointer.0,
            s.pointer.1,
        )
    })
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(json.contains("\"focused\""));
    }
}

/// wasm-bindgen export used by browser tests to read the input snapshot.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn input_state_json_js() -> String {
    input_state_json()
}
