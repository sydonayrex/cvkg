use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// HatiSpinner variant determining the visual style of the loading animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinnerVariant {
    Dots,
    Pulse,
    Bars,
    Ring,
}

impl Default for SpinnerVariant {
    fn default() -> Self {
        SpinnerVariant::Dots
    }
}

/// HatiSpinner - Animated loading indicator with multiple variants.
///
/// # Examples
/// ```
/// use cvkg_components::HatiSpinner;
/// let spinner = HatiSpinner::new()
///     .size(32.0)
///     .variant(SpinnerVariant::Ring);
/// ```
#[derive(Clone)]
pub struct HatiSpinner {
    pub variant: SpinnerVariant,
    pub size: f32,
    pub color: [f32; 4],
    pub speed: f32,
}

impl HatiSpinner {
    pub fn new() -> Self {
        Self {
            variant: SpinnerVariant::Dots,
            size: 24.0,
            color: theme::accent(),
            speed: 1.0,
        }
    }

    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }

    pub fn variant(mut self, v: SpinnerVariant) -> Self {
        self.variant = v;
        self
    }

    pub fn color(mut self, c: [f32; 4]) -> Self {
        self.color = c;
        self
    }
}

impl View for HatiSpinner {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!("Primitive view has no body")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let cx = rect.x + rect.width / 2.0;
        let cy = rect.y + rect.height / 2.0;
        let r = self.size / 2.0;

        match self.variant {
            SpinnerVariant::Dots => {
                for i in 0..8 {
                    let angle = (i as f32) * std::f32::consts::PI / 4.0;
                    let dx = cx + angle.cos() * r * 0.6;
                    let dy = cy + angle.sin() * r * 0.6;
                    let dot_r = r * 0.15;
                    renderer.fill_ellipse(
                        Rect {
                            x: dx - dot_r,
                            y: dy - dot_r,
                            width: dot_r * 2.0,
                            height: dot_r * 2.0,
                        },
                        self.color,
                    );
                }
            }
            SpinnerVariant::Ring => {
                renderer.stroke_ellipse(
                    Rect {
                        x: cx - r,
                        y: cy - r,
                        width: r * 2.0,
                        height: r * 2.0,
                    },
                    self.color,
                    2.0,
                );
            }
            _ => {
                renderer.fill_rounded_rect(
                    Rect {
                        x: cx - r,
                        y: cy - r,
                        width: r * 2.0,
                        height: r * 2.0,
                    },
                    r * 0.2,
                    self.color,
                );
            }
        }
    }
}
