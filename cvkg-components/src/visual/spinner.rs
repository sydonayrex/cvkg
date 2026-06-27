use crate::theme;
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{Never, Rect, Renderer, Size, View};

/// HatiSpinner variant determining the visual style of the loading animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpinnerVariant {
    #[default]
    Dots,
    Pulse,
    Ring,
    Ouroboros,
}

/// HatiSpinner - Animated loading indicator with multiple variants.
///
/// # Examples
/// ```
/// use cvkg_components::{HatiSpinner, SpinnerVariant};
/// let spinner = HatiSpinner::new()
///     .size(32.0)
///     .variant(SpinnerVariant::Ring);
/// ```
#[doc(alias = "Spinner")]
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
                // Rotate the dot ring using elapsed time so it visibly spins
                let rotation = renderer.elapsed_time() * self.speed * std::f32::consts::TAU;
                for i in 0..8 {
                    let base_angle = (i as f32) * std::f32::consts::PI / 4.0;
                    let angle = base_angle + rotation;
                    let dx = cx + angle.cos() * r * 0.6;
                    let dy = cy + angle.sin() * r * 0.6;
                    let dot_r = r * 0.15;
                    // Fade trailing dots for a comet tail effect
                    let trail_alpha = 0.2 + 0.8 * ((i as f32) / 8.0);
                    let mut color = self.color;
                    color[3] *= trail_alpha;
                    renderer.fill_ellipse(
                        Rect {
                            x: dx - dot_r,
                            y: dy - dot_r,
                            width: dot_r * 2.0,
                            height: dot_r * 2.0,
                        },
                        color,
                     );
                }
            }
            SpinnerVariant::Ring => {
                // Draw a complete background ring (dim) then an animated arc sweep on top
                let t = renderer.elapsed_time() * self.speed;
                let ring_rect = Rect {
                    x: cx - r,
                    y: cy - r,
                    width: r * 2.0,
                    height: r * 2.0,
                };
                // Background ring at low opacity
                let mut bg = self.color;
                bg[3] *= 0.18;
                renderer.stroke_ellipse(ring_rect, bg, 2.5);
                // Spinning arc overlay — draw 6 line segments approximating a ~270° arc
                let arc_span = std::f32::consts::PI * 1.5; // 270°
                let start_angle = t * std::f32::consts::TAU;
                let segments = 24u32;
                for j in 0..segments {
                    let frac = j as f32 / segments as f32;
                    let a = start_angle + frac * arc_span;
                    let b = start_angle + (j + 1) as f32 / segments as f32 * arc_span;
                    let alpha = 0.25 + 0.75 * frac; // brighter towards the head
                    let mut seg_color = self.color;
                    seg_color[3] = alpha;
                    renderer.draw_line(
                        cx + a.cos() * r,
                        cy + a.sin() * r,
                        cx + b.cos() * r,
                        cy + b.sin() * r,
                        seg_color,
                        2.5,
                    );
                }
            }
            SpinnerVariant::Ouroboros => {
                let t = renderer.elapsed_time() * self.speed;
                let loop_duration = 3.0;
                let loop_time = (t % loop_duration) / loop_duration;

                let head_angle = -std::f32::consts::FRAC_PI_2 + loop_time * std::f32::consts::TAU;
                let max_body_len = std::f32::consts::TAU * 0.85;
                let body_len = if loop_time < 0.8 {
                    max_body_len * (loop_time / 0.8)
                } else {
                    max_body_len
                };

                // High resolution: more steps for smooth serpentine body
                let steps = 600;
                let c_tail = [0.35, 0.22, 0.05, 1.0];
                let c_head = [1.0, 0.80, 0.20, 1.0];

                // Body radius scales with size for proper proportions
                let body_r = r * 0.75;
                // Body thickness tapers from head to tail
                let max_thickness = self.size * 0.14;
                let min_thickness = self.size * 0.02;

                // 1. Draw coiling body and scales
                for i in 0..=steps {
                    let f = i as f32 / steps as f32;
                    let angle = head_angle - (1.0 - f) * body_len;
                    let sx = cx + angle.cos() * body_r;
                    let sy = cy + angle.sin() * body_r;

                    // Smooth taper: thick near head, thin at tail
                    let thickness = min_thickness + (max_thickness - min_thickness) * (1.0 - f).powf(1.8);

                    // Main dorsal body
                    let color = [
                        c_tail[0] + (c_head[0] - c_tail[0]) * f,
                        c_tail[1] + (c_head[1] - c_tail[1]) * f,
                        c_tail[2] + (c_head[2] - c_tail[2]) * f,
                        1.0,
                    ];
                    renderer.fill_ellipse(
                        Rect {
                            x: sx - thickness / 2.0,
                            y: sy - thickness / 2.0,
                            width: thickness,
                            height: thickness,
                        },
                        color,
                    );

                    // Ventral belly (inner edge, warm gold) - only on body, not head
                    if f < 0.92 {
                        let nx = angle.cos();
                        let ny = angle.sin();
                        let bx = sx + nx * thickness * 0.18;
                        let by = sy + ny * thickness * 0.18;
                        let bw = thickness * 0.6;
                        renderer.fill_ellipse(
                            Rect {
                                x: bx - bw / 2.0,
                                y: by - bw / 2.0,
                                width: bw,
                                height: bw,
                            },
                            [0.96, 0.86, 0.40, 0.85],
                        );
                    }

                    // Dorsal scale highlights (every 8th step, bright gold ridge)
                    if i % 8 == 0 && f > 0.1 && f < 0.9 {
                        let nx = angle.cos();
                        let ny = angle.sin();
                        let scx = sx - nx * thickness * 0.25;
                        let scy = sy - ny * thickness * 0.25;
                        let scw = thickness * 0.35;
                        renderer.fill_ellipse(
                            Rect {
                                x: scx - scw / 2.0,
                                y: scy - scw / 2.0,
                                width: scw,
                                height: scw,
                            },
                            [1.0, 0.95, 0.68, 0.75],
                        );
                    }

                    // Secondary scale row (offset, every 8th step offset by 4)
                    if i % 8 == 4 && f > 0.15 && f < 0.88 {
                        let nx = angle.cos();
                        let ny = angle.sin();
                        let scx = sx - nx * thickness * 0.12;
                        let scy = sy - ny * thickness * 0.12;
                        let scw = thickness * 0.22;
                        renderer.fill_ellipse(
                            Rect {
                                x: scx - scw / 2.0,
                                y: scy - scw / 2.0,
                                width: scw,
                                height: scw,
                            },
                            [0.98, 0.90, 0.55, 0.5],
                        );
                    }
                }

                // 2. Draw detailed snake head (snout, jaw, head shield)
                let hx = cx + head_angle.cos() * body_r;
                let hy = cy + head_angle.sin() * body_r;
                let head_size = max_thickness * 1.6;

                let tangent = head_angle + std::f32::consts::FRAC_PI_2;
                let tx = tangent.cos();
                let ty = tangent.sin();

                // Neck transition
                let neck_size = head_size * 0.7;
                let nx = head_angle.cos();
                let ny = head_angle.sin();
                let neck_x = hx - nx * head_size * 0.4;
                let neck_y = hy - ny * head_size * 0.4;
                renderer.fill_ellipse(
                    Rect {
                        x: neck_x - neck_size / 2.0,
                        y: neck_y - neck_size / 2.0,
                        width: neck_size,
                        height: neck_size,
                    },
                    [0.92, 0.72, 0.20, 1.0],
                );

                // Main head shield
                renderer.fill_ellipse(
                    Rect {
                        x: hx - head_size / 2.0,
                        y: hy - head_size / 2.0,
                        width: head_size,
                        height: head_size,
                    },
                    [0.95, 0.75, 0.22, 1.0],
                );

                // Head top highlight (specular)
                renderer.fill_ellipse(
                    Rect {
                        x: hx - head_size * 0.25,
                        y: hy - head_size * 0.35,
                        width: head_size * 0.4,
                        height: head_size * 0.2,
                    },
                    [1.0, 0.92, 0.55, 0.3],
                );

                // Snout
                let snout_size = head_size * 0.65;
                let snx = hx + tx * head_size * 0.35;
                let sny = hy + ty * head_size * 0.35;
                renderer.fill_ellipse(
                    Rect {
                        x: snx - snout_size / 2.0,
                        y: sny - snout_size / 2.0,
                        width: snout_size,
                        height: snout_size,
                    },
                    [1.0, 0.88, 0.38, 1.0],
                );

                // Snout tip (nostril area)
                let tip_size = snout_size * 0.4;
                let tip_x = snx + tx * snout_size * 0.35;
                let tip_y = sny + ty * snout_size * 0.35;
                renderer.fill_ellipse(
                    Rect {
                        x: tip_x - tip_size / 2.0,
                        y: tip_y - tip_size / 2.0,
                        width: tip_size,
                        height: tip_size,
                    },
                    [0.90, 0.70, 0.20, 1.0],
                );

                // Flickering red tongue
                if (t * 3.3).floor() as i32 % 2 == 0 {
                    let tongue_start_x = snx + tx * snout_size * 0.45;
                    let tongue_start_y = sny + ty * snout_size * 0.45;
                    let tongue_end_x = tongue_start_x + tx * head_size * 0.5;
                    let tongue_end_y = tongue_start_y + ty * head_size * 0.5;
                    renderer.draw_line(
                        tongue_start_x,
                        tongue_start_y,
                        tongue_end_x,
                        tongue_end_y,
                        [0.9, 0.1, 0.12, 0.95],
                        1.5,
                    );
                    let fork_l_x = tongue_end_x + (tangent + 0.5).cos() * head_size * 0.15;
                    let fork_l_y = tongue_end_y + (tangent + 0.5).sin() * head_size * 0.15;
                    let fork_r_x = tongue_end_x + (tangent - 0.5).cos() * head_size * 0.15;
                    let fork_r_y = tongue_end_y + (tangent - 0.5).sin() * head_size * 0.15;
                    renderer.draw_line(tongue_end_x, tongue_end_y, fork_l_x, fork_l_y, [0.9, 0.1, 0.12, 0.95], 1.2);
                    renderer.draw_line(tongue_end_x, tongue_end_y, fork_r_x, fork_r_y, [0.9, 0.1, 0.12, 0.95], 1.2);
                }

                // 3. Draw glowing cyan eyes
                let eye_offset_angle_l = tangent + 0.48;
                let eye_offset_angle_r = tangent - 0.48;
                let eye_dist = head_size * 0.28;
                let eye_size = head_size * 0.24;

                let lex = hx + eye_offset_angle_l.cos() * eye_dist;
                let ley = hy + eye_offset_angle_l.sin() * eye_dist;
                let rex = hx + eye_offset_angle_r.cos() * eye_dist;
                let rey = hy + eye_offset_angle_r.sin() * eye_dist;
                let eye_color = [0.35, 0.92, 1.0, 1.0];

                // Eye glow halo
                renderer.fill_ellipse(
                    Rect {
                        x: lex - eye_size * 1.8,
                        y: ley - eye_size * 1.8,
                        width: eye_size * 3.6,
                        height: eye_size * 3.6,
                    },
                    [0.35, 0.92, 1.0, 0.15],
                );
                renderer.fill_ellipse(
                    Rect {
                        x: rex - eye_size * 1.8,
                        y: rey - eye_size * 1.8,
                        width: eye_size * 3.6,
                        height: eye_size * 3.6,
                    },
                    [0.35, 0.92, 1.0, 0.15],
                );

                renderer.fill_ellipse(
                    Rect {
                        x: lex - eye_size / 2.0,
                        y: ley - eye_size / 2.0,
                        width: eye_size,
                        height: eye_size,
                    },
                    eye_color,
                );
                renderer.fill_ellipse(
                    Rect {
                        x: rex - eye_size / 2.0,
                        y: rey - eye_size / 2.0,
                        width: eye_size,
                        height: eye_size,
                    },
                    eye_color,
                );

                // Pupils (vertical slit)
                let pupil_size = eye_size * 0.4;
                renderer.fill_ellipse(
                    Rect {
                        x: lex - pupil_size / 2.0,
                        y: ley - eye_size * 0.35,
                        width: pupil_size,
                        height: eye_size * 0.7,
                    },
                    [0.0, 0.05, 0.1, 1.0],
                );
                renderer.fill_ellipse(
                    Rect {
                        x: rex - pupil_size / 2.0,
                        y: rey - eye_size * 0.35,
                        width: pupil_size,
                        height: eye_size * 0.7,
                    },
                    [0.0, 0.05, 0.1, 1.0],
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

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.size,
            height: self.size,
        }
    }

    fn layout(&self) -> Option<&dyn LayoutView> {
        Some(self)
    }
}

impl LayoutView for HatiSpinner {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: self.size,
            height: self.size,
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {}
}

