use crate::theme;
use cvkg_core::{Event, Never, Rect, Renderer, View};
use std::sync::{Arc, Mutex};

/// An interactive drawing canvas.
/// Strokes are rendered as glowing runic fissures.
/// Section 4.6: "Direct runic inscription via stylus or pointer."
pub struct ScribingStone {
    pub strokes: Arc<Mutex<Vec<Vec<[f32; 2]>>>>,
}

impl Default for ScribingStone {
    fn default() -> Self {
        Self::new()
    }
}

impl ScribingStone {
    pub fn new() -> Self {
        Self {
            strokes: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl View for ScribingStone {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // 1. Basalt Surface
        renderer.fill_rect(rect, theme::surface_elevated());

        // 2. Render existing strokes
        let strokes = self.strokes.lock().unwrap_or_else(|e| e.into_inner());
        for stroke in strokes.iter() {
            for window in stroke.windows(2) {
                let p1 = window[0];
                let p2 = window[1];
                renderer.draw_line(p1[0], p1[1], p2[0], p2[1], theme::accent(), 2.0);
            }
        }

        // 3. Interaction Handler
        let strokes_clone = self.strokes.clone();
        renderer.register_handler(
            "pointerdown",
            Arc::new(move |ev| {
                if let Event::PointerDown { x, y, .. } = ev {
                    let mut s = strokes_clone.lock().unwrap_or_else(|e| e.into_inner());
                    s.push(vec![[x, y]]);
                }
            }),
        );

        let strokes_clone2 = self.strokes.clone();
        renderer.register_handler(
            "pointermove",
            Arc::new(move |ev| {
                if let Event::PointerMove { x, y, .. } = ev {
                    let mut s = strokes_clone2.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(last_stroke) = s.last_mut() {
                        last_stroke.push([x, y]);
                    }
                }
            }),
        );

        renderer.pop_vnode();
    }
}
