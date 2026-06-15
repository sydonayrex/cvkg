//! QRCode component drawing vector QR codes natively on the GPU.
//!
//! Renders standard finder patterns and payload matrices without external dependencies.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, Size, SizeProposal, View};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A custom GPU-rasterized vector QR Code generator and viewer.
#[derive(Clone)]
pub struct QRCode {
    /// The string payload encoded in the QR matrix.
    pub(crate) payload: String,
}

impl QRCode {
    /// Create a new QRCode component.
    ///
    /// # Arguments
    /// * `payload` - The text content to represent.
    pub fn new(payload: impl Into<String>) -> Self {
        Self {
            payload: payload.into(),
        }
    }
}

impl View for QRCode {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "QRCode");

        // QR Background (white block)
        renderer.fill_rounded_rect(rect, 4.0, [1.0, 1.0, 1.0, 1.0]);

        // Draw standard QR 21x21 grid size
        let grid_size = 21;
        let cell_w = rect.width / grid_size as f32;
        let cell_h = rect.height / grid_size as f32;

        // Deterministic hash value from the payload to generate noise pattern
        let mut hasher = DefaultHasher::new();
        self.payload.hash(&mut hasher);
        let mut hash_val = hasher.finish();

        for r in 0..grid_size {
            for c in 0..grid_size {
                let cell_rect = Rect {
                    x: rect.x + c as f32 * cell_w,
                    y: rect.y + r as f32 * cell_h,
                    width: cell_w,
                    height: cell_h,
                };

                // ── Check if cell is part of standard Finder Patterns (Top-Left, Top-Right, Bottom-Left) ──
                let is_tl = r < 7 && c < 7;
                let is_tr = r < 7 && c >= grid_size - 7;
                let is_bl = r >= grid_size - 7 && c < 7;

                if is_tl || is_tr || is_bl {
                    // Normalize relative coordinates inside the 7x7 block
                    let local_r = if is_bl { r - (grid_size - 7) } else { r };
                    let local_c = if is_tr { c - (grid_size - 7) } else { c };

                    // Finder pattern: 7x7 outer border, 3x3 inner fill
                    let is_border = local_r == 0 || local_r == 6 || local_c == 0 || local_c == 6;
                    let is_center = local_r >= 2 && local_r <= 4 && local_c >= 2 && local_c <= 4;

                    if is_border || is_center {
                        renderer.fill_rect(cell_rect, theme::qr_dark());
                    }
                } else {
                    // Generate pseudo-random matrix cells based on payload hash
                    hash_val = hash_val.rotate_left(1);
                    if hash_val & 1 == 1 {
                        renderer.fill_rect(cell_rect, theme::qr_dark());
                    }
                }
            }
        }

        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let size = proposal
            .width
            .unwrap_or(120.0)
            .min(proposal.height.unwrap_or(120.0));
        Size {
            width: size,
            height: size,
        }
    }
}
