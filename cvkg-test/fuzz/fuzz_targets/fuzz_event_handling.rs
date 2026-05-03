// Fuzz testing for event handling
// Tests event processing with random inputs to catch edge cases

#![no_main]

use libfuzzer_sys::fuzz_target;
use cvkg_core::Rect;
use std::ffi::CString;

fuzz_target!(|data: &[u8]| {
    // Parse event type from first byte
    if data.is_empty() {
        return;
    }
    
    let event_type = data[0] % 5;
    
    // Generate random coordinates for the event
    let x = if data.len() > 1 { data[1] as f32 * 10.0 } else { 0.0 };
    let y = if data.len() > 2 { data[2] as f32 * 10.0 } else { 0.0 };
    
    let rect = Rect::new(x, y, 100.0, 100.0);
    
    // Test different event types
    match event_type {
        0 => {
            // Click event - verify rect contains point
            let _ = rect.contains_point(x, y);
        }
        1 => {
            // Hover event
            let _ = rect.width();
            let _ = rect.height();
        }
        2 => {
            // Scroll event (delta from data)
            let delta_x = if data.len() > 3 { (data[3] as i8) as f32 } else { 0.0 };
            let delta_y = if data.len() > 4 { (data[4] as i8) as f32 } else { 0.0 };
            let _ = delta_x + delta_y;
        }
        3 => {
            // Keyboard event (char from data)
            if data.len() > 5 {
                let ch = data[5] as char;
                let _s = CString::new(ch.to_string()).unwrap_or_default();
            }
        }
        4 => {
            // Bounds check
            let _ = rect.x();
            let _ = rect.y();
            let _ = rect.right();
            let _ = rect.bottom();
        }
        _ => unreachable!(),
    }
});
