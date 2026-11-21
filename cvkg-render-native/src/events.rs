use std::path::PathBuf;

/// Normalise a winit physical-key debug string into the shared logical name.
///
/// Extracted from [`convert_keyboard_event`] so the platform boundary can be
/// unit-tested without constructing a `winit::event::KeyEvent` (which has
/// private platform-specific fields).
pub(crate) fn normalize_key_code(raw: &str) -> String {
    cvkg_core::normalize_key_name(raw).unwrap_or_else(|| raw.to_string())
}
use winit::event::{ElementState, Ime, KeyEvent};
use winit::keyboard::{ModifiersState, PhysicalKey};

/// Converts a winit keyboard event into a CVKG event.
pub fn convert_keyboard_event(
    event: KeyEvent,
    modifiers: &ModifiersState,
) -> Option<cvkg_core::Event> {
    if let PhysicalKey::Code(code) = event.physical_key {
        // P1-5: canonicalise winit physical codes ("KeyA", "Digit1") into the
        // same logical names web/key handlers use ("a", "1"). Components that
        // insert typed characters expect single-character keys on both targets.
        let key_str = normalize_key_code(&format!("{:?}", code));
        let cvkg_mods = cvkg_core::KeyModifiers {
            shift: modifiers.shift_key(),
            ctrl: modifiers.control_key(),
            alt: modifiers.alt_key(),
            meta: modifiers.super_key(),
        };
        if event.state == ElementState::Pressed {
            Some(cvkg_core::Event::KeyDown {
                key: key_str,
                modifiers: cvkg_mods,
            })
        } else {
            Some(cvkg_core::Event::KeyUp {
                key: key_str,
                modifiers: cvkg_mods,
            })
        }
    } else {
        None
    }
}

/// Converts a winit IME event into a CVKG event.
pub fn convert_ime_event(event: Ime) -> Option<cvkg_core::Event> {
    if let Ime::Commit(string) = event {
        Some(cvkg_core::Event::Ime(string))
    } else {
        None
    }
}

/// Converts a winit mouse event into a CVKG event.
pub fn convert_mouse_event(
    state: ElementState,
    position: [f32; 2],
    button: u32,
) -> cvkg_core::Event {
    match state {
        ElementState::Pressed => cvkg_core::Event::PointerDown {
            x: position[0],
            y: position[1],
            button,
            proximity_field: 0.0,
            tilt: None,
            azimuth: None,
            pressure: Some(1.0),
            barrel_rotation: None,
            pointer_precision: 0.0,
        },
        ElementState::Released => cvkg_core::Event::PointerUp {
            x: position[0],
            y: position[1],
            button,
            tilt: None,
            azimuth: None,
            pressure: Some(0.0),
            barrel_rotation: None,
            pointer_precision: 0.0,
        },
    }
}

/// Searches known asset directories for 'icon.png'.
/// Returns a winit Icon if found and decodable, None otherwise.
pub fn load_icon() -> Option<winit::window::Icon> {
    let base = std::env::current_dir().unwrap_or_else(|e| {
        tracing::warn!(
            "[Native] Failed to get current directory for icon search: {}",
            e
        );
        PathBuf::new()
    });

    let mut candidates = vec![
        base.join("icon.png"),
        base.join("crates/ulfhednar/icons/icon.png"),
        base.join("ulfhednar/icons/icon.png"),
        base.join("crates/ulfhednar/assets/icon.png"),
        base.join("ulfhednar/assets/icon.png"),
        base.join("assets/icon.png"),
    ];

    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        candidates.push(exe_dir.join("icons/icon.png"));
        candidates.push(exe_dir.join("assets/icon.png"));
        candidates.push(exe_dir.join("icon.png"));
        if let Some(parent) = exe_dir.parent() {
            candidates.push(parent.join("icons/icon.png"));
            candidates.push(parent.join("assets/icon.png"));
            candidates.push(parent.join("icon.png"));
        }
    }

    for path in candidates {
        if !path.exists() {
            tracing::debug!("[Native] Icon candidate not found: {:?}", path);
            continue;
        }

        match image::open(&path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (width, height) = rgba.dimensions();
                match winit::window::Icon::from_rgba(rgba.into_raw(), width, height) {
                    Ok(icon) => {
                        tracing::info!("[Native] Successfully loaded app icon from: {:?}", path);
                        return Some(icon);
                    }
                    Err(e) => {
                        tracing::warn!("[Native] Icon format error at {:?}: {}", path, e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("[Native] Failed to open icon image at {:?}: {}", path, e);
            }
        }
    }

    tracing::warn!(
        "[Native] Failed to find icon.png in any search path (CWD: {:?})",
        base
    );
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // P1-5: native physical codes must produce the same logical key names as
    // web KeyboardEvent.key so shared shortcut handlers work on both targets.
    #[test]
    fn physical_codes_are_normalized_to_logical_keys() {
        assert_eq!(normalize_key_code("KeyA"), "a");
        assert_eq!(normalize_key_code("Digit1"), "1");
        assert_eq!(normalize_key_code("ShiftLeft"), "ShiftLeft"); // bare modifiers stay raw
        assert_eq!(normalize_key_code("Escape"), "Escape");
        assert_eq!(normalize_key_code("Space"), "Space");
        assert_eq!(normalize_key_code("CustomKey"), "CustomKey");
    }
}
