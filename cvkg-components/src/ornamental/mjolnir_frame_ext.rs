//! Extended MjolnirFrame variants for berserker theming.
//! Extends the existing MjolnirFrame system with 5 new frame styles.

use cvkg_core::{Rect, Renderer};

pub enum MjolnirFrameStyle {
    /// Standard geometric frame (existing).
    Standard,
    /// Carved runestone: weathered edges with embedded runes.
    RuneStone { runes: Vec<RuneGlyph> },
    /// Hammered metal: irregular forged surface with rivets.
    HammeredMetal { oxidation: f32 },
    /// Dragon scale: interlocking scale tessellation.
    DragonScale { scale_size: f32 },
    /// Ice crystal: fractal ice growth from corners.
    IceCrystal { growth_progress: f32 },
    /// Void rift: dark energy tearing at frame boundaries.
    VoidRift { rift_intensity: f32 },
}

pub struct RuneGlyph {
    pub character: char,
    pub position: f32,
    pub glow_intensity: f32,
}

/// Render a frame with the selected style.
pub fn render_mjolnir_frame(
    renderer: &mut dyn Renderer,
    rect: Rect,
    style: &MjolnirFrameStyle,
    color: [f32; 4],
) {
    match style {
        MjolnirFrameStyle::Standard => {
            renderer.stroke_rect(rect, color, 2.0);
        }
        MjolnirFrameStyle::RuneStone { runes } => {
            let weathered = [color[0] * 0.8, color[1] * 0.7, color[2] * 0.6, color[3]];
            renderer.stroke_rect(rect, weathered, 3.0);
            for rune in runes {
                let x = rect.x + rect.width * rune.position;
                let y = rect.y - 8.0;
                renderer.draw_text(
                    &rune.character.to_string(),
                    x,
                    y,
                    10.0,
                    [0.9, 0.72, 0.3, rune.glow_intensity],
                );
            }
        }
        MjolnirFrameStyle::HammeredMetal { oxidation } => {
            let base = [0.5 - *oxidation * 0.2, 0.45 - *oxidation * 0.15, 0.4, 0.9];
            renderer.stroke_rect(rect, base, 4.0);
            // Rivets at corners
            let corners = [
                (rect.x, rect.y),
                (rect.x + rect.width, rect.y),
                (rect.x, rect.y + rect.height),
                (rect.x + rect.width, rect.y + rect.height),
            ];
            for (cx, cy) in &corners {
                renderer.fill_ellipse(
                    cvkg_core::Rect {
                        x: cx - 3.0,
                        y: cy - 3.0,
                        width: 6.0,
                        height: 6.0,
                    },
                    [0.6, 0.55, 0.5, 1.0],
                );
            }
        }
        MjolnirFrameStyle::DragonScale { scale_size } => {
            let count = (rect.width / scale_size) as usize;
            for i in 0..count.max(1) {
                let x = rect.x + i as f32 * *scale_size;
                renderer.fill_rounded_rect(
                    cvkg_core::Rect {
                        x,
                        y: rect.y - 2.0,
                        width: scale_size * 0.8,
                        height: 6.0,
                    },
                    3.0,
                    [0.2, 0.5, 0.3, 0.7],
                );
            }
        }
        MjolnirFrameStyle::IceCrystal { growth_progress } => {
            let corners = [
                (rect.x, rect.y),
                (rect.x + rect.width, rect.y),
                (rect.x, rect.y + rect.height),
                (rect.x + rect.width, rect.y + rect.height),
            ];
            for (cx, cy) in &corners {
                let size = 8.0 + growth_progress * 12.0;
                renderer.draw_line(*cx, *cy, *cx + size, *cy - size, [0.7, 0.85, 1.0, 0.6], 1.5);
                renderer.draw_line(*cx, *cy, *cx - size, *cy - size, [0.7, 0.85, 1.0, 0.6], 1.5);
            }
        }
        MjolnirFrameStyle::VoidRift { rift_intensity } => {
            let jitter = rift_intensity * 4.0;
            renderer.stroke_rect(
                cvkg_core::Rect {
                    x: rect.x - jitter,
                    y: rect.y - jitter,
                    width: rect.width + jitter * 2.0,
                    height: rect.height + jitter * 2.0,
                },
                [0.1, 0.0, 0.15, 0.9],
                2.0 + jitter,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rune_stone_has_runes() {
        let style = MjolnirFrameStyle::RuneStone {
            runes: vec![RuneGlyph {
                character: '\u{16A0}',
                position: 0.5,
                glow_intensity: 0.8,
            }],
        };
        match style {
            MjolnirFrameStyle::RuneStone { runes } => {
                assert_eq!(runes.len(), 1);
                assert_eq!(runes[0].character, '\u{16A0}');
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_hammered_metal_oxidation() {
        let style = MjolnirFrameStyle::HammeredMetal { oxidation: 0.5 };
        match style {
            MjolnirFrameStyle::HammeredMetal { oxidation } => {
                assert!((oxidation - 0.5).abs() < 0.01)
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_dragon_scale_size() {
        let style = MjolnirFrameStyle::DragonScale { scale_size: 12.0 };
        match style {
            MjolnirFrameStyle::DragonScale { scale_size } => {
                assert!((scale_size - 12.0).abs() < 0.01)
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_ice_crystal_growth() {
        let style = MjolnirFrameStyle::IceCrystal {
            growth_progress: 0.5,
        };
        match style {
            MjolnirFrameStyle::IceCrystal { growth_progress } => {
                assert!((growth_progress - 0.5).abs() < 0.01)
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_void_rift_intensity() {
        let style = MjolnirFrameStyle::VoidRift {
            rift_intensity: 1.0,
        };
        match style {
            MjolnirFrameStyle::VoidRift { rift_intensity } => {
                assert!((rift_intensity - 1.0).abs() < 0.01)
            }
            _ => panic!("wrong variant"),
        }
    }
}
