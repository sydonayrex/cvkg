use crate::theme;
use crate::{ButtonSize, ButtonVariant, FONT_BASE, RADIUS_MD, RADIUS_SM};
use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::Arc;

// =============================================================================
// BUTTON — Full state machine with variants, sizes, disabled state, focus ring
// =============================================================================

/// Button with action callback, variant styling, size options, and disabled state.
#[derive(Clone)]
pub struct Button {
    pub(crate) label: String,
    pub(crate) on_click: Arc<dyn Fn() + Send + Sync>,
    pub(crate) variant: ButtonVariant,
    pub(crate) size: ButtonSize,
    pub(crate) disabled: bool,
}

impl Button {
    /// Create a new Button with a label and an action callback.
    pub fn new(label: impl Into<String>, on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            label: label.into(),
            on_click: Arc::new(on_click),
            variant: ButtonVariant::Default,
            size: ButtonSize::Default,
            disabled: false,
        }
    }

    /// Set the button variant.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the button size.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Set the disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Compute the background color based on variant and state.
    fn bg_color(&self, is_pressed: bool, is_hovered: bool) -> [f32; 4] {
        if self.disabled {
            return theme::disabled();
        }
        match self.variant {
            ButtonVariant::Default => {
                if is_pressed {
                    theme::active_color()
                } else if is_hovered {
                    theme::hover()
                } else {
                    theme::button_secondary_bg()
                }
            }
            ButtonVariant::Destructive => theme::error_color(),
            ButtonVariant::Secondary => {
                if is_pressed {
                    theme::active_color()
                } else if is_hovered {
                    theme::hover()
                } else {
                    theme::button_secondary_bg()
                }
            }
            ButtonVariant::Ghost => {
                if is_pressed {
                    [0.08, 0.08, 0.12, 0.5]
                } else if is_hovered {
                    theme::hover()
                } else {
                    theme::button_ghost_bg()
                }
            }
            ButtonVariant::Link => theme::button_ghost_bg(),
        }
    }

    /// Compute the border color based on variant and state.
    fn border_color(&self, is_pressed: bool, is_hovered: bool) -> ([f32; 4], f32) {
        if self.disabled {
            return (theme::disabled(), 1.0);
        }
        match self.variant {
            ButtonVariant::Default => {
                if is_pressed {
                    (theme::accent(), 3.0)
                } else if is_hovered {
                    (theme::accent_hover(), 2.0)
                } else {
                    (theme::accent(), 1.5)
                }
            }
            ButtonVariant::Destructive => {
                if is_pressed {
                    (theme::error_color(), 3.0)
                } else if is_hovered {
                    (theme::error_color(), 2.0)
                } else {
                    (theme::error_color(), 1.5)
                }
            }
            ButtonVariant::Secondary => {
                if is_pressed {
                    (theme::border_strong(), 2.0)
                } else if is_hovered {
                    (theme::border(), 1.5)
                } else {
                    (theme::border(), 1.0)
                }
            }
            ButtonVariant::Ghost => {
                if is_pressed {
                    ([0.3, 0.3, 0.4, 0.5], 1.0)
                } else {
                    (theme::button_ghost_bg(), 0.0)
                }
            }
            ButtonVariant::Link => (theme::button_ghost_bg(), 0.0),
        }
    }

    /// Compute the text color based on variant and state.
    fn text_color(&self, is_hovered: bool) -> [f32; 4] {
        if self.disabled {
            return [0.35, 0.35, 0.4, 0.5];
        }
        match self.variant {
            ButtonVariant::Default | ButtonVariant::Destructive => theme::text(),
            ButtonVariant::Secondary => theme::text(),
            ButtonVariant::Ghost => {
                if is_hovered {
                    theme::text()
                } else {
                    theme::text_muted()
                }
            }
            ButtonVariant::Link => theme::accent(),
        }
    }

    /// Compute the height based on size variant.
    fn height(&self) -> f32 {
        match self.size {
            ButtonSize::Small => 32.0,
            ButtonSize::Default => 44.0,
            ButtonSize::Large => 52.0,
            ButtonSize::Icon => 44.0,
        }
    }

    /// Compute the font size based on size variant.
    fn font_size(&self) -> f32 {
        match self.size {
            ButtonSize::Small => 12.0,
            ButtonSize::Default => FONT_BASE,
            ButtonSize::Large => FONT_BASE,
            ButtonSize::Icon => FONT_BASE,
        }
    }

    /// Compute horizontal padding based on size variant.
    fn h_padding(&self) -> f32 {
        match self.size {
            ButtonSize::Small => 12.0,
            ButtonSize::Default => 16.0,
            ButtonSize::Large => 24.0,
            ButtonSize::Icon => 12.0,
        }
    }
}

impl View for Button {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let id_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            self.label.hash(&mut s);
            s.finish()
        };

        let hover_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            self.label.hash(&mut s);
            "hover".hash(&mut s);
            s.finish()
        };

        let focus_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            self.label.hash(&mut s);
            "focus".hash(&mut s);
            s.finish()
        };

        let (is_pressed, set_pressed) = cvkg_vdom::use_state(id_hash, false);
        let (is_hovered, set_hovered) = cvkg_vdom::use_state(hover_hash, false);
        let (is_focused, set_focused) = cvkg_vdom::use_state(focus_hash, false);

        let hover_anim_hash = hover_hash.wrapping_add(12345);
        let press_anim_hash = focus_hash.wrapping_add(12345);

        let hover_target = if is_hovered { 1.0 } else { 0.0 };
        let mut hover_t = 0.0;
        {
            let s = cvkg_core::load_system_state();
            if s.get_component_state::<cvkg_anim::SleipnirSolver>(hover_anim_hash)
                .is_none()
            {
                cvkg_core::update_system_state(|st| {
                    let mut new_st = st.clone();
                    new_st.set_component_state(
                        hover_anim_hash,
                        cvkg_anim::SleipnirSolver::new(
                            cvkg_anim::SleipnirParams::snappy(),
                            hover_target,
                            hover_target,
                        ),
                    );
                    new_st
                });
            }
        }
        {
            let s = cvkg_core::load_system_state();
            if let Some(solver_arc) =
                s.get_component_state::<cvkg_anim::SleipnirSolver>(hover_anim_hash)
            {
                let mut solver = solver_arc.write().unwrap();
                solver.set_target(hover_target);
                hover_t = solver.tick(renderer.delta_time());
            }
        }

        let press_target = if is_pressed { 1.0 } else { 0.0 };
        let mut press_t = 0.0;
        {
            let s = cvkg_core::load_system_state();
            if s.get_component_state::<cvkg_anim::SleipnirSolver>(press_anim_hash)
                .is_none()
            {
                cvkg_core::update_system_state(|st| {
                    let mut new_st = st.clone();
                    new_st.set_component_state(
                        press_anim_hash,
                        cvkg_anim::SleipnirSolver::new(
                            cvkg_anim::SleipnirParams::snappy(),
                            press_target,
                            press_target,
                        ),
                    );
                    new_st
                });
            }
        }
        {
            let s = cvkg_core::load_system_state();
            if let Some(solver_arc) =
                s.get_component_state::<cvkg_anim::SleipnirSolver>(press_anim_hash)
            {
                let mut solver = solver_arc.write().unwrap();
                solver.set_target(press_target);
                press_t = solver.tick(renderer.delta_time());
            }
        }

        // Pointer-based proximity and magnetic warping calculation.
        let [px, py] = renderer.get_pointer_position();
        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        let dx = px - center_x;
        let dy = py - center_y;
        let dist = (dx * dx + dy * dy).sqrt();
        let radius = 120.0;
        let proximity = if dist < radius {
            (1.0 - dist / radius).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let intensity = if self.disabled { 0.0 } else { 0.25 };
        let mut offset_x = 0.0;
        let mut offset_y = 0.0;
        if dist < radius && dist > 0.0 && !self.disabled {
            let force = (1.0 - dist / radius) * intensity;
            offset_x = dx * force;
            offset_y = dy * force;
        }

        let warped_rect = Rect {
            x: rect.x + offset_x,
            y: rect.y + offset_y,
            ..rect
        };

        // Apply scale down to 0.97x on press
        let scale = 1.0 - (0.03 * press_t);
        let scaled_w = warped_rect.width * scale;
        let scaled_h = warped_rect.height * scale;
        let scaled_x = warped_rect.x + (warped_rect.width - scaled_w) / 2.0;
        let scaled_y = warped_rect.y + (warped_rect.height - scaled_h) / 2.0;
        let final_rect = Rect {
            x: scaled_x,
            y: scaled_y,
            width: scaled_w,
            height: scaled_h,
        };

        renderer.push_vnode(final_rect, "Button");
        renderer.set_key(&self.label);
        renderer.set_aria_role("button");
        renderer.set_aria_label(&self.label);

        // Apply mani_glow() soft lunar-like highlight
        if !self.disabled {
            let glow_color = [0.0, 0.9, 1.0, 0.8 * proximity];
            let glow_radius = 20.0 * proximity;
            if glow_radius > 0.0 {
                renderer.mani_glow(final_rect, glow_color, glow_radius);
            }
        }

        let lerp_color = |a: [f32; 4], b: [f32; 4], t: f32| -> [f32; 4] {
            [
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
                a[2] + (b[2] - a[2]) * t,
                a[3] + (b[3] - a[3]) * t,
            ]
        };

        let bg_normal = self.bg_color(false, false);
        let bg_hover = self.bg_color(false, true);
        let bg_press = self.bg_color(true, true);
        let bg = lerp_color(lerp_color(bg_normal, bg_hover, hover_t), bg_press, press_t);

        let (bc_normal, bw_normal) = self.border_color(false, false);
        let (bc_hover, bw_hover) = self.border_color(false, true);
        let (bc_press, bw_press) = self.border_color(true, true);
        let border_color = lerp_color(lerp_color(bc_normal, bc_hover, hover_t), bc_press, press_t);
        let border_width =
            bw_normal + (bw_hover - bw_normal) * hover_t + (bw_press - bw_hover) * press_t;

        let tc_normal = self.text_color(false);
        let tc_hover = self.text_color(true);
        let text_color = lerp_color(tc_normal, tc_hover, hover_t);
        let font_size = self.font_size();

        // Elevation & Depth
        if !matches!(self.variant, ButtonVariant::Ghost | ButtonVariant::Link) {
            renderer.push_shadow(1.0, [0.0, 0.0, 0.0, 0.5], [0.0, 1.0]);
        }
        let corner_radius = match self.variant {
            ButtonVariant::Link => 0.0,
            _ => RADIUS_MD,
        };
        if corner_radius > 0.0 {
            renderer.fill_rounded_rect(final_rect, corner_radius, bg);
        } else {
            renderer.fill_rect(final_rect, bg);
        }
        if !matches!(self.variant, ButtonVariant::Ghost | ButtonVariant::Link) {
            renderer.pop_shadow();
        }

        // Stroke border
        if border_width > 0.0 && corner_radius > 0.0 {
            renderer.stroke_rounded_rect(final_rect, corner_radius, border_color, border_width);
        } else if border_width > 0.0 {
            renderer.stroke_rect(final_rect, border_color, border_width);
        }

        // Focus ring — WCAG 2.4.7
        if is_focused && !self.disabled {
            crate::draw_focus_ring(renderer, final_rect);
        }

        // Label text centered
        let (tw, _th) = renderer.measure_text(&self.label, font_size);
        let text_x = final_rect.x + (final_rect.width - tw) / 2.0;
        let text_y = final_rect.y + (final_rect.height - font_size) / 2.0;
        renderer.draw_text(&self.label, text_x, text_y, font_size, text_color);

        // Register interaction handlers
        if !self.disabled {
            let on_click = self.on_click.clone();
            renderer.register_handler(
                "pointerclick",
                std::sync::Arc::new(move |_| {
                    (on_click)();
                    // Haptic and audio feedback on button click.
                    cvkg_core::haptic_impact(cvkg_core::HapticIntensity::Light);
                    cvkg_core::play_sound("success_chime", 0.8);
                }),
            );
        }

        let is_disabled = self.disabled;
        let set_pressed_down = set_pressed.clone();
        renderer.register_handler(
            "pointerdown",
            std::sync::Arc::new(move |_| {
                if !is_disabled {
                    (set_pressed_down)(true);
                }
            }),
        );

        let set_pressed_up = set_pressed.clone();
        renderer.register_handler(
            "pointerup",
            std::sync::Arc::new(move |_| {
                (set_pressed_up)(false);
            }),
        );

        let set_hovered_enter = set_hovered.clone();
        renderer.register_handler(
            "pointerenter",
            std::sync::Arc::new(move |_| {
                (set_hovered_enter)(true);
            }),
        );

        let set_hovered_leave = set_hovered.clone();
        renderer.register_handler(
            "pointerleave",
            std::sync::Arc::new(move |_| {
                (set_hovered_leave)(false);
                (set_pressed)(false);
            }),
        );

        // Focus handlers
        let set_focused_in = set_focused.clone();
        renderer.register_handler(
            "focus",
            std::sync::Arc::new(move |_| {
                (set_focused_in)(true);
            }),
        );

        let set_focused_out = set_focused.clone();
        renderer.register_handler(
            "blur",
            std::sync::Arc::new(move |_| {
                (set_focused_out)(false);
            }),
        );

        renderer.pop_vnode();
    }

    fn layout(&self) -> Option<&dyn cvkg_core::layout::LayoutView> {
        Some(self)
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        _proposal: cvkg_core::layout::SizeProposal,
    ) -> cvkg_core::Size {
        let font_size = self.font_size();
        let (tw, _th) = renderer.measure_text(&self.label, font_size);
        let h_pad = self.h_padding();
        cvkg_core::Size {
            width: (tw + h_pad * 2.0).max(self.height()),
            height: self.height(),
        }
    }
}

impl cvkg_core::layout::LayoutView for Button {
    fn size_that_fits(
        &self,
        _proposal: cvkg_core::layout::SizeProposal,
        _subviews: &[&dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: (self.label.len() as f32 * self.font_size() * 0.6 + self.h_padding() * 2.0)
                .max(self.height()),
            height: self.height(),
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) {
    }
}

#[derive(Clone)]
pub struct Toggle {
    pub(crate) label: String,
    pub(crate) is_on: bool,
    pub(crate) on_change: std::sync::Arc<dyn Fn(bool) + Send + Sync>,
}

impl Toggle {
    /// Create a new Toggle switch.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::Toggle;
    /// let toggle = Toggle::new("Enable feature", false, |val| println!("Toggled: {}", val));
    /// ```
    pub fn new(
        label: impl Into<String>,
        is_on: bool,
        on_change: impl Fn(bool) + Send + Sync + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            is_on,
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for Toggle {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let id_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            self.label.hash(&mut s);
            "toggle".hash(&mut s);
            s.finish()
        };

        let target = if self.is_on { 1.0 } else { 0.0 };
        let mut toggle_t = 0.0;
        {
            let s = cvkg_core::load_system_state();
            if s.get_component_state::<cvkg_anim::SleipnirSolver>(id_hash)
                .is_none()
            {
                cvkg_core::update_system_state(|st| {
                    let mut new_st = st.clone();
                    new_st.set_component_state(
                        id_hash,
                        cvkg_anim::SleipnirSolver::new(
                            cvkg_anim::SleipnirParams::snappy(),
                            target,
                            target,
                        ),
                    );
                    new_st
                });
            }
        }
        {
            let s = cvkg_core::load_system_state();
            if let Some(solver_arc) = s.get_component_state::<cvkg_anim::SleipnirSolver>(id_hash) {
                let mut solver = solver_arc.write().unwrap();
                solver.set_target(target);
                toggle_t = solver.tick(renderer.delta_time());
            }
        }

        renderer.push_vnode(rect, "Toggle");
        renderer.set_aria_role("switch");
        renderer.set_aria_label(&self.label);

        let track_w = 40.0;
        let track_h = 20.0;
        let track_x = rect.x;
        let track_y = rect.y + (rect.height - track_h) / 2.0;
        let track = Rect {
            x: track_x,
            y: track_y,
            width: track_w,
            height: track_h,
        };

        let lerp_color = |a: [f32; 4], b: [f32; 4], t: f32| -> [f32; 4] {
            [
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
                a[2] + (b[2] - a[2]) * t,
                a[3] + (b[3] - a[3]) * t,
            ]
        };

        let bg_off = theme::surface_elevated();
        let bg_on = theme::accent();
        let bg = lerp_color(bg_off, bg_on, toggle_t);

        renderer.fill_rounded_rect(track, track_h / 2.0, bg);

        // Thumb position interpolation
        let thumb_x_off = track_x + 2.0;
        let thumb_x_on = track_x + track_w - track_h + 2.0;
        let thumb_x = thumb_x_off + (thumb_x_on - thumb_x_off) * toggle_t;

        renderer.fill_rounded_rect(
            Rect {
                x: thumb_x,
                y: track_y + 2.0,
                width: track_h - 4.0,
                height: track_h - 4.0,
            },
            (track_h - 4.0) / 2.0,
            theme::text(),
        );
        // Label
        renderer.draw_text(
            &self.label,
            rect.x + track_w + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            theme::text(),
        );

        // Interaction
        let is_on = self.is_on;
        let on_change = self.on_change.clone();
        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |_| {
                (on_change)(!is_on);
            }),
        );

        // Haptic and audio feedback on pointer down for tactile response.
        renderer.register_handler(
            "pointerdown",
            std::sync::Arc::new(move |_| {
                cvkg_core::haptic_selection();
                cvkg_core::play_sound("nav_tick", 0.7);
            }),
        );

        renderer.pop_vnode();
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        _proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let (tw, th) = renderer.measure_text(&self.label, 14.0);
        cvkg_core::Size {
            width: 40.0 + 8.0 + tw,
            height: th.max(20.0) + 4.0,
        }
    }
}

#[derive(Clone)]
pub struct Slider {
    pub(crate) value: f32,
    pub(crate) range: std::ops::RangeInclusive<f32>,
    pub(crate) step: Option<f32>,
    pub(crate) on_change: std::sync::Arc<dyn Fn(f32) + Send + Sync>,
}

impl Slider {
    /// Create a new Slider.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::Slider;
    /// let slider = Slider::new(0.5, 0.0..=1.0, |val| println!("Value: {}", val));
    /// ```
    pub fn new(
        value: f32,
        range: std::ops::RangeInclusive<f32>,
        on_change: impl Fn(f32) + Send + Sync + 'static,
    ) -> Self {
        Self {
            value,
            range,
            step: None,
            on_change: std::sync::Arc::new(on_change),
        }
    }

    /// Set a step increment for the slider.
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }
}

impl View for Slider {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let id_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            "slider".hash(&mut s);
            self.range.start().to_bits().hash(&mut s);
            self.range.end().to_bits().hash(&mut s);
            s.finish()
        };

        let (is_dragging, set_dragging) = cvkg_vdom::use_state(id_hash, false);

        renderer.push_vnode(rect, "Slider");
        renderer.set_aria_role("slider");
        let track_h = 4.0;
        let track_y = rect.y + (rect.height - track_h) / 2.0;
        // Track background
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x,
                y: track_y,
                width: rect.width,
                height: track_h,
            },
            track_h / 2.0,
            theme::surface_elevated(),
        );
        // Track fill
        let start = *self.range.start();
        let end = *self.range.end();
        let pct = if (end - start).abs() > f32::EPSILON {
            ((self.value - start) / (end - start)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x,
                y: track_y,
                width: rect.width * pct,
                height: track_h,
            },
            track_h / 2.0,
            theme::accent(),
        );
        // Thumb
        let thumb_r = 8.0;
        let thumb_x = rect.x + rect.width * pct - thumb_r;
        renderer.fill_rounded_rect(
            Rect {
                x: thumb_x,
                y: track_y - thumb_r + track_h / 2.0,
                width: thumb_r * 2.0,
                height: thumb_r * 2.0,
            },
            thumb_r,
            theme::text(),
        );

        // Interaction
        let on_change = self.on_change.clone();
        let slider_rect = rect;
        let step_val = self.step;

        let drag_start = set_dragging.clone();
        renderer.register_handler(
            "pointerdown",
            std::sync::Arc::new(move |_| {
                (drag_start)(true);
            }),
        );

        let drag_stop = set_dragging.clone();
        renderer.register_handler(
            "pointerup",
            std::sync::Arc::new(move |_| {
                (drag_stop)(false);
            }),
        );

        renderer.register_handler(
            "pointermove",
            std::sync::Arc::new(move |event| {
                if !is_dragging {
                    return;
                }
                if let cvkg_core::Event::PointerMove { x, .. } = event {
                    let pct = ((x - slider_rect.x) / slider_rect.width).clamp(0.0, 1.0);
                    let mut val = start + pct * (end - start);
                    if let Some(s) = step_val
                        && s > 0.0
                    {
                        let snapped = (val / s).round() * s;
                        // Trigger haptic feedback when crossing a snap point.
                        if (snapped - val).abs() > f32::EPSILON {
                            cvkg_core::haptic_selection();
                        }
                        val = snapped;
                    }
                    val = val.clamp(start, end);
                    (on_change)(val);
                }
            }),
        );
        renderer.pop_vnode();
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(150.0),
            height: 24.0,
        }
    }
}

/// Stepper for discrete increment/decrement
#[derive(Clone)]
pub struct Stepper {
    pub(crate) label: String,
    pub(crate) value: i32,
    pub(crate) on_change: std::sync::Arc<dyn Fn(i32) + Send + Sync>,
}

impl Stepper {
    pub fn new(
        label: impl Into<String>,
        value: i32,
        on_change: impl Fn(i32) + Send + Sync + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            value,
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for Stepper {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.set_aria_role("spinbutton");
        renderer.set_aria_label(&self.label);

        // Stepper container
        renderer.fill_rounded_rect(rect, 4.0, theme::surface_elevated());
        renderer.stroke_rect(rect, theme::text_dim(), 1.0);

        let label_w = rect.width * 0.4;
        renderer.draw_text(
            &self.label,
            rect.x + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            theme::text(),
        );

        // Buttons
        let btn_w = 30.0;
        let minus_rect = Rect {
            x: rect.x + label_w,
            y: rect.y + 4.0,
            width: btn_w,
            height: rect.height - 8.0,
        };
        let plus_rect = Rect {
            x: rect.x + rect.width - btn_w - 4.0,
            y: rect.y + 4.0,
            width: btn_w,
            height: rect.height - 8.0,
        };

        renderer.fill_rounded_rect(minus_rect, 2.0, theme::surface_elevated());
        renderer.draw_text(
            "-",
            minus_rect.x + 10.0,
            minus_rect.y + (minus_rect.height - 14.0) / 2.0,
            14.0,
            theme::text(),
        );

        renderer.fill_rounded_rect(plus_rect, 2.0, theme::surface_elevated());
        renderer.draw_text(
            "+",
            plus_rect.x + 10.0,
            plus_rect.y + (plus_rect.height - 14.0) / 2.0,
            14.0,
            theme::text(),
        );

        let val_text = self.value.to_string();
        let val_x = minus_rect.x + btn_w + 10.0;
        renderer.draw_text(
            &val_text,
            val_x,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            theme::accent(),
        );

        // Interaction
        let on_change = self.on_change.clone();
        let value = self.value;

        // We use a shared element to detect which button was clicked
        // For now, we simulate by checking coordinates in a single handler or using multiple sub-nodes.
        // In CVKG VDOM, we can just register handlers on the parent and check coords.
        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |event| {
                if let cvkg_core::Event::PointerClick { x, .. } = event {
                    if x >= minus_rect.x && x <= minus_rect.x + minus_rect.width {
                        (on_change)(value - 1);
                    } else if x >= plus_rect.x && x <= plus_rect.x + plus_rect.width {
                        (on_change)(value + 1);
                    }
                }
            }),
        );
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        _proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let (lw, lh) = renderer.measure_text(&self.label, 14.0);
        let (vw, _) = renderer.measure_text(&self.value.to_string(), 14.0);
        cvkg_core::Size {
            width: lw + 8.0 + 30.0 + vw + 20.0 + 30.0 + 8.0,
            height: lh.max(30.0) + 8.0,
        }
    }
}
#[derive(Clone)]
pub struct SecureField {
    pub(crate) placeholder: String,
    pub(crate) text: String,
    pub(crate) on_change: std::sync::Arc<dyn Fn(String) + Send + Sync>,
}

impl SecureField {
    pub fn new(
        placeholder: impl Into<String>,
        text: impl Into<String>,
        on_change: impl Fn(String) + Send + Sync + 'static,
    ) -> Self {
        Self {
            placeholder: placeholder.into(),
            text: text.into(),
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for SecureField {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.set_aria_role("password");

        // Input background
        renderer.fill_rounded_rect(rect, 6.0, theme::surface_elevated());
        renderer.stroke_rect(rect, theme::text_muted(), 1.0); // Slightly purple for security

        let display = if self.text.is_empty() {
            self.placeholder.clone()
        } else {
            "*".repeat(self.text.len())
        };

        let text_color = if self.text.is_empty() {
            theme::text_muted()
        } else {
            theme::text()
        };
        renderer.draw_text(
            &display,
            rect.x + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            text_color,
        );

        // Interaction (Secure key entry)
        let current_text = std::sync::Arc::new(std::sync::Mutex::new(self.text.clone()));
        let on_change = self.on_change.clone();

        renderer.register_handler(
            "keydown",
            std::sync::Arc::new(move |event| {
                if let cvkg_core::Event::KeyDown { key } = event {
                    let mut changed = false;
                    let mut new_text = String::new();
                    if let Ok(mut text_guard) = current_text.lock() {
                        if key.len() == 1 {
                            text_guard.push_str(&key);
                            new_text = text_guard.clone();
                            changed = true;
                        } else if key == "Backspace" {
                            text_guard.pop();
                            new_text = text_guard.clone();
                            changed = true;
                        }
                    }
                    if changed {
                        (on_change)(new_text);
                    }
                }
            }),
        );
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let text = if self.text.is_empty() {
            &self.placeholder
        } else {
            "*"
        }; // Proxy for measurement
        let (tw, th) = renderer.measure_text(text, 14.0);
        let width = if self.text.is_empty() {
            tw + 24.0
        } else {
            (self.text.len() as f32 * 10.0) + 24.0
        };
        cvkg_core::Size {
            width: proposal.width.unwrap_or(width).max(100.0),
            height: th + 16.0,
        }
    }
}

// =============================================================================
// INPUT — Text entry with validation states, focus ring, and proper focus border
// =============================================================================

/// Input validation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputState {
    Default,
    Focused,
    Error,
    Success,
    Disabled,
}

/// Single-line text input with cursor, selection, clipboard, and undo.
///
/// Supports:
/// - Text entry with visible cursor (blinking)
/// - Text selection via mouse drag or Shift+Arrow
/// - Clipboard: Ctrl+C (copy), Ctrl+V (paste), Ctrl+X (cut)
/// - Undo/Redo: Ctrl+Z / Ctrl+Shift+Z
/// - IME composition for CJK input
/// - Arrow keys, Home/End, word navigation (Ctrl+Arrow)
/// - Enter fires `on_commit`
#[derive(Clone)]
pub struct Input {
    pub(crate) placeholder: String,
    pub(crate) text: String,
    pub(crate) on_change: Arc<dyn Fn(String) + Send + Sync>,
    pub(crate) on_commit: Arc<dyn Fn(String) + Send + Sync>,
    pub(crate) is_focused: bool,
    pub(crate) input_state: InputState,
    pub(crate) error_message: Option<String>,
    /// Unique hash for this input instance (for system state)
    pub(crate) state_id: u64,
}

impl Input {
    /// Create a new Input field.
    pub fn new(placeholder: impl Into<String>) -> Self {
        use std::hash::{Hash, Hasher};
        let mut s = std::collections::hash_map::DefaultHasher::new();
        "input".hash(&mut s);
        std::time::SystemTime::now().hash(&mut s);
        Self {
            placeholder: placeholder.into(),
            text: String::new(),
            on_change: Arc::new(|_| {}),
            on_commit: Arc::new(|_| {}),
            is_focused: false,
            input_state: InputState::Default,
            error_message: None,
            state_id: s.finish(),
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.text = value.into();
        self
    }

    pub fn on_change(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_change = Arc::new(callback);
        self
    }

    pub fn on_commit(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_commit = Arc::new(callback);
        self
    }

    pub fn focused(mut self, is_focused: bool) -> Self {
        self.is_focused = is_focused;
        if is_focused {
            self.input_state = InputState::Focused;
        }
        self
    }

    pub fn error(mut self, message: impl Into<String>) -> Self {
        self.input_state = InputState::Error;
        self.error_message = Some(message.into());
        self
    }

    pub fn success(mut self) -> Self {
        self.input_state = InputState::Success;
        self
    }

    fn bg_color(&self) -> [f32; 4] {
        match self.input_state {
            InputState::Disabled => theme::surface(),
            InputState::Error => [0.15, 0.05, 0.05, 1.0],
            InputState::Success => [0.05, 0.15, 0.05, 1.0],
            _ => theme::surface_elevated(),
        }
    }

    fn border_color(&self) -> [f32; 4] {
        match self.input_state {
            InputState::Focused => theme::accent(),
            InputState::Error => theme::error_color(),
            InputState::Success => theme::success(),
            InputState::Disabled => theme::text_dim(),
            _ => theme::border(),
        }
    }

    /// Get or initialize the TextInputState from system state.
    fn get_text_state(&self) -> cvkg_core::TextInputState {
        let sys = cvkg_core::load_system_state();
        if let Some(arc) = sys.get_component_state::<cvkg_core::TextInputState>(self.state_id) {
            arc.read().unwrap().clone()
        } else {
            let mut state = cvkg_core::TextInputState::new(self.text.clone());
            state.focused = self.is_focused;
            state
        }
    }

    /// Save TextInputState to system state.
    fn save_text_state(&self, state: &cvkg_core::TextInputState) {
        cvkg_core::update_system_state(|s| {
            let mut ns = s.clone();
            ns.set_component_state(self.state_id, state.clone());
            ns
        });
    }

    /// Get the display text (or placeholder if empty).
    fn display_text(&self) -> &str {
        if self.text.is_empty() {
            &self.placeholder
        } else {
            &self.text
        }
    }

    /// Compute the pixel offset for a given byte position in the text.
    fn text_offset_for_pos(&self, renderer: &mut dyn Renderer, byte_pos: usize) -> f32 {
        let prefix = &self.text[..byte_pos.min(self.text.len())];
        if prefix.is_empty() {
            0.0
        } else {
            renderer.measure_text(prefix, FONT_BASE).0
        }
    }

    /// Compute the byte position closest to a given pixel offset.
    #[allow(dead_code)]
    fn pos_for_text_offset(&self, renderer: &mut dyn Renderer, x_offset: f32) -> usize {
        let mut best_pos = 0;
        let mut best_dist = f32::MAX;
        for (i, _) in self.text.char_indices() {
            let w = renderer.measure_text(&self.text[..i], FONT_BASE).0;
            let dist = (w - x_offset).abs();
            if dist < best_dist {
                best_dist = dist;
                best_pos = i;
            }
        }
        let end_w = renderer.measure_text(&self.text, FONT_BASE).0;
        if (end_w - x_offset).abs() < best_dist {
            self.text.len()
        } else {
            best_pos
        }
    }
}

impl View for Input {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Input");
        renderer.set_aria_role("textbox");
        renderer.set_aria_label(&self.placeholder);

        let bg = self.bg_color();
        let border = self.border_color();
        let is_disabled = self.input_state == InputState::Disabled;

        // Input background
        renderer.fill_rounded_rect(rect, RADIUS_MD, bg);
        renderer.stroke_rect(rect, border, if self.is_focused { 2.0 } else { 1.0 });

        // Focus ring — WCAG 2.4.7
        if self.is_focused && !is_disabled {
            crate::draw_focus_ring(renderer, rect);
        }

        // Get text input state
        let text_state = self.get_text_state();
        let display_text = self.display_text();
        let text_color = if self.text.is_empty() {
            theme::text_muted()
        } else if is_disabled {
            theme::text_dim()
        } else {
            theme::text()
        };

        // Text area
        let text_rect = Rect {
            x: rect.x + 8.0,
            y: rect.y,
            width: rect.width - 16.0,
            height: rect.height,
        };

        // Render selection highlight
        if text_state.focused
            && let Some((sel_start, sel_end)) = text_state.selection_range()
        {
            let x_start = self.text_offset_for_pos(renderer, sel_start);
            let x_end = self.text_offset_for_pos(renderer, sel_end);
            let sel_rect = Rect {
                x: text_rect.x + x_start,
                y: rect.y + (rect.height - FONT_BASE) - 2.0,
                width: x_end - x_start,
                height: FONT_BASE + 4.0,
            };
            renderer.fill_rounded_rect(
                sel_rect,
                2.0,
                [
                    theme::accent()[0],
                    theme::accent()[1],
                    theme::accent()[2],
                    0.3,
                ],
            );
        }

        // Render text
        renderer.draw_text(
            display_text,
            text_rect.x,
            rect.y + (rect.height - FONT_BASE) / 2.0,
            FONT_BASE,
            text_color,
        );

        // Render cursor (blinking)
        if text_state.focused && !is_disabled {
            let cursor_x_offset = self.text_offset_for_pos(renderer, text_state.cursor_pos);
            let cursor_x = text_rect.x + cursor_x_offset;
            let cursor_y = rect.y + (rect.height - 16.0) / 2.0;
            let time = renderer.elapsed_time();
            let alpha = if (time * 2.0).sin() > 0.0 { 1.0 } else { 0.3 };
            renderer.draw_line(
                cursor_x,
                cursor_y,
                cursor_x,
                cursor_y + 16.0,
                [
                    theme::accent()[0],
                    theme::accent()[1],
                    theme::accent()[2],
                    alpha,
                ],
                2.0,
            );
        }

        // Error message
        if let Some(ref msg) = self.error_message {
            renderer.draw_text(
                msg,
                rect.x + 8.0,
                rect.y + rect.height + 4.0,
                12.0,
                theme::error_color(),
            );
        }

        // Save state
        self.save_text_state(&text_state);

        // === Interaction handlers ===
        if !is_disabled {
            // Keydown handler
            {
                let on_change = self.on_change.clone();
                let on_commit = self.on_commit.clone();
                let state_id = self.state_id;
                renderer.register_handler(
                    "keydown",
                    Arc::new(move |event| {
                        if let cvkg_core::Event::KeyDown { key } = event {
                            let mut changed = false;
                            let mut commit = false;

                            let sys = cvkg_core::load_system_state();
                            let old_text_state = if let Some(arc) =
                                sys.get_component_state::<cvkg_core::TextInputState>(state_id)
                            {
                                arc.read().unwrap().clone()
                            } else {
                                cvkg_core::TextInputState::new("")
                            };
                            let mut text_state = old_text_state.clone();

                            match key.as_str() {
                                s if s.len() == 1 && !s.chars().next().unwrap().is_control() => {
                                    text_state.insert(s);
                                    changed = true;
                                }
                                "Back" | "Backspace" => {
                                    text_state.delete(true, 1);
                                    changed = true;
                                }
                                "Delete" => {
                                    text_state.delete(false, 1);
                                    changed = true;
                                }
                                "ArrowLeft" => {
                                    text_state
                                        .move_cursor(cvkg_core::TextDirection::Backward, false);
                                }
                                "ArrowRight" => {
                                    text_state
                                        .move_cursor(cvkg_core::TextDirection::Forward, false);
                                }
                                "ArrowUp" | "Home" => {
                                    text_state
                                        .move_cursor(cvkg_core::TextDirection::LineStart, false);
                                }
                                "ArrowDown" | "End" => {
                                    text_state
                                        .move_cursor(cvkg_core::TextDirection::LineEnd, false);
                                }
                                "Enter" | "Return" => {
                                    commit = true;
                                }
                                _ => {}
                            }

                            if changed {
                                let new_text = text_state.text.clone();
                                let new_text_state = text_state.clone();
                                let on_change_clone1 = on_change.clone();
                                let on_change_clone2 = on_change.clone();

                                let label = match key.as_str() {
                                    "Back" | "Backspace" | "Delete" => "Delete",
                                    _ => "Type",
                                };

                                let u_state = old_text_state.clone();
                                let r_state = new_text_state.clone();

                                cvkg_core::update_system_state(move |s| {
                                    let mut ns = s.clone();
                                    ns.set_component_state(state_id, new_text_state.clone());

                                    let oc_undo = on_change_clone1.clone();
                                    let oc_redo = on_change_clone2.clone();
                                    let u_state = u_state.clone();
                                    let r_state = r_state.clone();

                                    ns.undo_manager.push_coalesceable(
                                        label,
                                        move || {
                                            cvkg_core::update_system_state({
                                                let u_state = u_state.clone();
                                                move |st| {
                                                    let mut nst = st.clone();
                                                    nst.set_component_state(
                                                        state_id,
                                                        u_state.clone(),
                                                    );
                                                    nst
                                                }
                                            });
                                            (oc_undo)(u_state.text.clone());
                                        },
                                        move || {
                                            cvkg_core::update_system_state({
                                                let r_state = r_state.clone();
                                                move |st| {
                                                    let mut nst = st.clone();
                                                    nst.set_component_state(
                                                        state_id,
                                                        r_state.clone(),
                                                    );
                                                    nst
                                                }
                                            });
                                            (oc_redo)(r_state.text.clone());
                                        },
                                    );
                                    ns
                                });
                                (on_change)(new_text);
                            }

                            if commit {
                                let sys = cvkg_core::load_system_state();
                                if let Some(arc) =
                                    sys.get_component_state::<cvkg_core::TextInputState>(state_id)
                                {
                                    let text = arc.read().unwrap().text.clone();
                                    (on_commit)(text);
                                }
                            }
                        }
                    }),
                );
            }

            // IME handler
            {
                let on_change = self.on_change.clone();
                let state_id = self.state_id;
                renderer.register_handler(
                    "ime",
                    Arc::new(move |event| {
                        if let cvkg_core::Event::Ime(composition) = event {
                            let sys = cvkg_core::load_system_state();
                            let old_text_state = if let Some(arc) =
                                sys.get_component_state::<cvkg_core::TextInputState>(state_id)
                            {
                                arc.read().unwrap().clone()
                            } else {
                                cvkg_core::TextInputState::new("")
                            };
                            let mut text_state = old_text_state.clone();
                            text_state.insert(&composition);
                            let new_text = text_state.text.clone();
                            let new_text_state = text_state.clone();
                            let on_change_clone1 = on_change.clone();
                            let on_change_clone2 = on_change.clone();

                            let u_state = old_text_state.clone();
                            let r_state = new_text_state.clone();

                            cvkg_core::update_system_state(move |s| {
                                let mut ns = s.clone();
                                ns.set_component_state(state_id, new_text_state.clone());

                                let oc_undo = on_change_clone1.clone();
                                let oc_redo = on_change_clone2.clone();
                                let u_state = u_state.clone();
                                let r_state = r_state.clone();

                                ns.undo_manager.push_coalesceable(
                                    "Type",
                                    move || {
                                        cvkg_core::update_system_state({
                                            let u_state = u_state.clone();
                                            move |st| {
                                                let mut nst = st.clone();
                                                nst.set_component_state(state_id, u_state.clone());
                                                nst
                                            }
                                        });
                                        (oc_undo)(u_state.text.clone());
                                    },
                                    move || {
                                        cvkg_core::update_system_state({
                                            let r_state = r_state.clone();
                                            move |st| {
                                                let mut nst = st.clone();
                                                nst.set_component_state(state_id, r_state.clone());
                                                nst
                                            }
                                        });
                                        (oc_redo)(r_state.text.clone());
                                    },
                                );
                                ns
                            });
                            (on_change)(new_text);
                        }
                    }),
                );
            }

            // Pointer interaction
            {
                let state_id = self.state_id;
                renderer.register_handler(
                    "pointerclick",
                    Arc::new(move |event| {
                        if let cvkg_core::Event::PointerClick { x, .. } = event {
                            let sys = cvkg_core::load_system_state();
                            let mut text_state = if let Some(arc) =
                                sys.get_component_state::<cvkg_core::TextInputState>(state_id)
                            {
                                arc.read().unwrap().clone()
                            } else {
                                cvkg_core::TextInputState::new("")
                            };
                            let text_x = x - rect.x - 8.0;
                            let byte_pos =
                                Input::pos_for_text_offset_static(&text_state.text, text_x);
                            text_state.cursor_pos = byte_pos;
                            text_state.selection_anchor = None;
                            text_state.focused = true;
                            cvkg_core::update_system_state(|s| {
                                let mut ns = s.clone();
                                ns.set_component_state(state_id, text_state);
                                ns
                            });
                        }
                    }),
                );
            }
        }

        renderer.pop_vnode();
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(200.0),
            height: 44.0,
        }
    }
}

impl Input {
    /// Static helper for computing byte position from pixel offset.
    fn pos_for_text_offset_static(text: &str, x_offset: f32) -> usize {
        let avg_char_width = 8.0;
        let estimated = (x_offset / avg_char_width).round() as usize;
        estimated.min(text.len())
    }
}

// TEXTAREA — Multi-line text editing with focus ring and proper state management
// =============================================================================

/// Multi-line text area with proper state management via system state.
#[derive(Clone)]
pub struct Textarea {
    pub(crate) placeholder: String,
    pub(crate) text: String,
    pub(crate) rows: usize,
    pub(crate) on_change: Arc<dyn Fn(String) + Send + Sync>,
}

impl Textarea {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            text: String::new(),
            rows: 3,
            on_change: Arc::new(|_| {}),
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.text = value.into();
        self
    }

    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows;
        self
    }

    pub fn on_change(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_change = Arc::new(callback);
        self
    }
}

impl View for Textarea {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Textarea");
        renderer.set_aria_role("textbox");
        renderer.set_aria_label(&self.placeholder);

        // Editor background
        renderer.fill_rounded_rect(rect, RADIUS_SM, theme::surface());
        renderer.stroke_rect(rect, theme::border_strong(), 1.0);

        // Draw text
        let line_height = 20.0;
        if self.text.is_empty() {
            renderer.draw_text(
                &self.placeholder,
                rect.x + 8.0,
                rect.y + 8.0,
                FONT_BASE,
                theme::border_strong(),
            );
        } else {
            let lines: Vec<&str> = self.text.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                let y = rect.y + 8.0 + (i as f32 * line_height);
                if y - rect.y < rect.height - 8.0 {
                    renderer.draw_text(line, rect.x + 8.0, y, FONT_BASE, theme::text());
                }
            }
        }

        // Draw Cursor on last line
        let text_lines: Vec<&str> = self.text.lines().collect();
        let last_line = text_lines.last().copied().unwrap_or("");
        let (tw, _) = renderer.measure_text(last_line, FONT_BASE);
        let cursor_x = rect.x + 8.0 + tw;
        let cursor_y = rect.y + 8.0 + (text_lines.len().max(1) - 1) as f32 * line_height;
        let time = renderer.elapsed_time();
        let alpha = if (time * 2.0).sin() > 0.0 { 1.0 } else { 0.3 };
        renderer.draw_line(
            cursor_x,
            cursor_y,
            cursor_x,
            cursor_y + 16.0,
            [0.0, 1.0, 1.0, alpha],
            2.0,
        );

        // Character count
        let count_text = format!("{} chars", self.text.len());
        let (cw, _) = renderer.measure_text(&count_text, 12.0);
        renderer.draw_text(
            &count_text,
            rect.x + rect.width - cw - 8.0,
            rect.y + rect.height - 16.0,
            12.0,
            [0.4, 0.4, 0.5, 0.7],
        );

        // Focus ring
        crate::draw_focus_ring(renderer, rect);

        // Interaction
        let on_change = self.on_change.clone();
        let text_mutex = Arc::new(std::sync::Mutex::new(self.text.clone()));

        let on_change_kd = on_change.clone();
        let text_mutex_kd = text_mutex.clone();
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let cvkg_core::Event::KeyDown { key } = event {
                    let mut changed = false;
                    let mut new_text = String::new();
                    if let Ok(mut text_guard) = text_mutex_kd.lock() {
                        if key.len() == 1 {
                            text_guard.push_str(&key);
                            changed = true;
                        } else if key == "Back" || key == "Backspace" {
                            text_guard.pop();
                            changed = true;
                        } else if key == "Return" || key == "Enter" {
                            text_guard.push('\n');
                            changed = true;
                        }
                        if changed {
                            new_text = text_guard.clone();
                        }
                    }
                    if changed {
                        (on_change_kd)(new_text);
                    }
                }
            }),
        );

        let on_change_ime = on_change.clone();
        let text_mutex_ime = text_mutex.clone();
        renderer.register_handler(
            "ime",
            Arc::new(move |event| {
                if let cvkg_core::Event::Ime(composition) = event {
                    let mut new_text = String::new();
                    if let Ok(mut text_guard) = text_mutex_ime.lock() {
                        text_guard.push_str(composition.as_str());
                        new_text = text_guard.clone();
                    }
                    (on_change_ime)(new_text);
                }
            }),
        );

        renderer.pop_vnode();
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(300.0),
            height: proposal.height.unwrap_or(self.rows as f32 * 20.0 + 16.0),
        }
    }
}

// =============================================================================
// SELECT — Dropdown select with keyboard navigation and focus ring
// =============================================================================

/// Select/Dropdown component with keyboard navigation, dropdown popover, and focus ring.
///
/// # Example
/// ```ignore
/// use cvkg_components::Select;
/// let select = Select::new("Choose...")
///     .option("Option A", 1)
///     .option("Option B", 2)
///     .option("Option C", 3);
/// ```
#[derive(Clone)]
pub struct Select<V> {
    placeholder: String,
    options: Vec<(String, V)>,
    selected_index: Option<usize>,
    is_open: bool,
    hover_index: Option<usize>,
    id_hash: u64,
}

impl<V: Clone> Select<V> {
    pub fn new(placeholder: impl Into<String>) -> Self {
        use std::hash::{Hash, Hasher};
        let placeholder_string = placeholder.into();
        let mut s = std::collections::hash_map::DefaultHasher::new();
        "select".hash(&mut s);
        placeholder_string.hash(&mut s);
        let id_hash = s.finish();
        Self {
            placeholder: placeholder_string,
            options: Vec::new(),
            selected_index: None,
            is_open: false,
            hover_index: None,
            id_hash,
        }
    }

    pub fn option(mut self, label: impl Into<String>, value: V) -> Self {
        self.options.push((label.into(), value));
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected_index = Some(index);
        self
    }
}

impl<V: Clone + View> View for Select<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Select");
        renderer.set_aria_role("combobox");
        renderer.set_aria_label(&self.placeholder);
        renderer.set_aria_role("combobox");

        // Read open state from system state
        let is_open = cvkg_core::load_system_state()
            .get_component_state::<bool>(self.id_hash)
            .map(|v| *v.read().unwrap())
            .unwrap_or(self.is_open);

        // Main select box
        let border_color = if is_open {
            [0.0, 0.8, 1.0, 0.8]
        } else {
            theme::text_dim()
        };
        renderer.fill_rounded_rect(rect, RADIUS_MD, theme::surface());
        renderer.stroke_rect(rect, border_color, if is_open { 2.0 } else { 1.0 });

        // Focus ring when open
        if is_open {
            crate::draw_focus_ring(renderer, rect);
        }

        let display_text = self
            .selected_index
            .and_then(|i| self.options.get(i))
            .map(|(l, _)| l.as_str())
            .unwrap_or(&self.placeholder);
        renderer.draw_text(
            display_text,
            rect.x + 12.0,
            rect.y + (rect.height - FONT_BASE) / 2.0,
            FONT_BASE,
            if self.selected_index.is_some() {
                theme::text()
            } else {
                theme::text_muted()
            },
        );

        // Chevron
        renderer.draw_text(
            if is_open { "▲" } else { "▼" },
            rect.x + rect.width - 20.0,
            rect.y + (rect.height - FONT_BASE) / 2.0,
            12.0,
            theme::text_muted(),
        );

        // Dropdown popover
        if is_open {
            let item_height = 32.0;
            let popover_h = (self.options.len() as f32 * item_height).min(200.0);
            let popover_rect = Rect {
                x: rect.x,
                y: rect.y + rect.height + 4.0,
                width: rect.width,
                height: popover_h,
            };

            renderer.set_z_index(100.0);
            renderer.bifrost(popover_rect, 20.0, 1.2, 0.9);
            renderer.fill_rounded_rect(popover_rect, RADIUS_MD, [0.05, 0.05, 0.1, 0.95]);
            renderer.stroke_rect(popover_rect, [0.0, 1.0, 1.0, 0.5], 1.0);

            // Read hover index from system state
            let hover_idx = cvkg_core::load_system_state()
                .get_component_state::<usize>(self.id_hash.wrapping_add(1))
                .map(|v| *v.read().unwrap())
                .or(self.hover_index);

            for (i, (label, _)) in self.options.iter().enumerate() {
                let item_rect = Rect {
                    x: popover_rect.x,
                    y: popover_rect.y + i as f32 * item_height,
                    width: popover_rect.width,
                    height: item_height,
                };

                let is_hovered = hover_idx == Some(i);

                // Selected highlight
                if self.selected_index == Some(i) {
                    renderer.fill_rounded_rect(item_rect, RADIUS_SM, [0.0, 0.5, 0.8, 0.3]);
                } else if is_hovered {
                    renderer.fill_rounded_rect(item_rect, RADIUS_SM, [0.15, 0.15, 0.2, 0.5]);
                }

                renderer.draw_text(
                    label,
                    item_rect.x + 12.0,
                    item_rect.y + (item_height - FONT_BASE) / 2.0,
                    FONT_BASE,
                    if self.selected_index == Some(i) {
                        theme::accent()
                    } else {
                        theme::text()
                    },
                );
            }
            renderer.set_z_index(0.0);
        }

        // Toggle on click
        let id_hash = self.id_hash;
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event| {
                if let cvkg_core::Event::PointerClick { x, y, .. } = event {
                    // If click is inside the main toggle rect, toggle open
                    if x >= rect.x
                        && x <= rect.x + rect.width
                        && y >= rect.y
                        && y <= rect.y + rect.height
                    {
                        cvkg_core::update_system_state(|s| {
                            let mut s = s.clone();
                            let current = s
                                .get_component_state::<bool>(id_hash)
                                .map(|v| *v.read().unwrap())
                                .unwrap_or(false);
                            s.set_component_state(id_hash, !current);
                            s
                        });
                    }
                }
            }),
        );

        // Keyboard navigation
        let options_count = self.options.len();
        let id_hash = self.id_hash;
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let cvkg_core::Event::KeyDown { key, .. } = event {
                    match key.as_str() {
                        "ArrowDown" => {
                            cvkg_core::update_system_state(|s| {
                                let mut s = s.clone();
                                let current = s
                                    .get_component_state::<usize>(id_hash.wrapping_add(1))
                                    .map(|v| *v.read().unwrap())
                                    .unwrap_or(0);
                                let next = (current + 1).min(options_count.saturating_sub(1));
                                s.set_component_state(id_hash.wrapping_add(1), next);
                                s
                            });
                        }
                        "ArrowUp" => {
                            cvkg_core::update_system_state(|s| {
                                let mut s = s.clone();
                                let current = s
                                    .get_component_state::<usize>(id_hash.wrapping_add(1))
                                    .map(|v| *v.read().unwrap())
                                    .unwrap_or(0);
                                let next = current.saturating_sub(1);
                                s.set_component_state(id_hash.wrapping_add(1), next);
                                s
                            });
                        }
                        "Enter" => {
                            cvkg_core::update_system_state(|s| {
                                let mut s = s.clone();
                                // Close the dropdown
                                s.set_component_state(id_hash, false);
                                s
                            });
                        }
                        "Escape" => {
                            cvkg_core::update_system_state(|s| {
                                let mut s = s.clone();
                                s.set_component_state(id_hash, false);
                                s
                            });
                        }
                        _ => {}
                    }
                }
            }),
        );

        if is_open {
            let item_height = 32.0;
            let popover_h = (self.options.len() as f32 * item_height).min(200.0);
            let popover_rect = Rect {
                x: rect.x,
                y: rect.y + rect.height + 4.0,
                width: rect.width,
                height: popover_h,
            };

            // Pointer hover tracking
            let id_hash_hover = self.id_hash.wrapping_add(1);
            let pr = popover_rect;
            renderer.register_handler(
                "pointermove",
                Arc::new(move |event| {
                    if let cvkg_core::Event::PointerMove { x, y, .. } = event
                        && x >= pr.x
                        && x <= pr.x + pr.width
                        && y >= pr.y
                        && y <= pr.y + pr.height
                    {
                        let hover_idx = ((y - pr.y) / item_height) as usize;
                        cvkg_core::update_system_state(|s| {
                            let mut s = s.clone();
                            s.set_component_state(id_hash_hover, hover_idx);
                            s
                        });
                    }
                }),
            );
        }

        renderer.pop_vnode();
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(150.0),
            height: 36.0,
        }
    }
}

/// Dropdown component for selecting from a list of options with a popover
pub struct Dropdown {
    pub(crate) selection: usize,
    pub(crate) options: Vec<String>,
    pub(crate) on_change: std::sync::Arc<dyn Fn(usize) + Send + Sync>,
}

impl Dropdown {
    pub fn new(
        selection: usize,
        options: Vec<String>,
        on_change: impl Fn(usize) + Send + Sync + 'static,
    ) -> Self {
        Self {
            selection,
            options,
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for Dropdown {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Dropdown");

        let id_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            "dropdown".hash(&mut s);
            self.options.len().hash(&mut s);
            s.finish()
        };

        // Lock-free read of expanded state
        let is_expanded = {
            let s = cvkg_core::load_system_state();
            s.get_component_state::<bool>(id_hash)
                .map(|v| *v.read().unwrap())
                .unwrap_or(false)
        };

        // Main button
        renderer.fill_rounded_rect(rect, 4.0, theme::surface());
        renderer.stroke_rect(rect, theme::accent_hover(), 1.0);

        let selected = self
            .options
            .get(self.selection)
            .cloned()
            .unwrap_or_default();
        renderer.draw_text(
            &selected,
            rect.x + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            theme::text(),
        );
        renderer.draw_text(
            if is_expanded { "▲" } else { "▼" },
            rect.x + rect.width - 20.0,
            rect.y + (rect.height - 14.0) / 2.0,
            12.0,
            theme::text_muted(),
        );

        if is_expanded {
            let popover_h = (self.options.len() as f32 * 30.0).min(200.0);
            let popover_rect = Rect {
                x: rect.x,
                y: rect.y + rect.height + 4.0,
                width: rect.width,
                height: popover_h,
            };

            // Z-Index boost for popover
            renderer.set_z_index(100.0);
            renderer.bifrost(popover_rect, 20.0, 1.2, 0.9);
            renderer.fill_rounded_rect(popover_rect, 4.0, [0.05, 0.05, 0.1, 0.95]);
            renderer.stroke_rect(popover_rect, [0.0, 1.0, 1.0, 0.5], 1.0);

            for (i, opt) in self.options.iter().enumerate() {
                let item_rect = Rect {
                    x: popover_rect.x,
                    y: popover_rect.y + i as f32 * 30.0,
                    width: popover_rect.width,
                    height: 30.0,
                };

                if i == self.selection {
                    renderer.fill_rect(item_rect, [0.0, 0.5, 0.8, 0.3]);
                }

                renderer.draw_text(
                    opt,
                    item_rect.x + 8.0,
                    item_rect.y + (item_rect.height - 14.0) / 2.0,
                    14.0,
                    theme::text(),
                );
            }
            renderer.set_z_index(0.0);
        }

        let options_count = self.options.len();
        let on_change = self.on_change.clone();

        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |event| {
                if let cvkg_core::Event::PointerClick { x, y, .. } = event {
                    if is_expanded {
                        let popover_h = (options_count as f32 * 30.0).min(200.0);
                        let popover_rect = Rect {
                            x: rect.x,
                            y: rect.y + rect.height + 4.0,
                            width: rect.width,
                            height: popover_h,
                        };

                        if x >= popover_rect.x
                            && x <= popover_rect.x + popover_rect.width
                            && y >= popover_rect.y
                            && y <= popover_rect.y + popover_rect.height
                        {
                            let idx = ((y - popover_rect.y) / 30.0) as usize;
                            if idx < options_count {
                                on_change(idx);
                            }
                        }
                    }

                    // Toggle expanded state atomically
                    cvkg_core::update_system_state(|s| {
                        let mut s = s.clone();
                        s.set_component_state(id_hash, !is_expanded);
                        s
                    });
                }
            }),
        );

        renderer.pop_vnode();
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let mut max_w = 0.0f32;
        for opt in &self.options {
            let (w, _) = renderer.measure_text(opt, 14.0);
            max_w = max_w.max(w);
        }
        cvkg_core::Size {
            width: proposal.width.unwrap_or(max_w + 40.0).max(120.0),
            height: 32.0,
        }
    }
}

/// Picker for selection from a list of options#[derive(Clone)]
pub struct Picker {
    pub(crate) selection: usize,
    pub(crate) options: Vec<String>,
    pub(crate) on_change: std::sync::Arc<dyn Fn(usize) + Send + Sync>,
}

impl Picker {
    pub fn new(
        selection: usize,
        options: Vec<String>,
        on_change: impl Fn(usize) + Send + Sync + 'static,
    ) -> Self {
        Self {
            selection,
            options,
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for Picker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.set_aria_role("combobox");

        // Picker background
        renderer.fill_rounded_rect(rect, 6.0, theme::surface_elevated());
        renderer.stroke_rect(rect, theme::text_dim(), 1.0);

        let selected_text = self
            .options
            .get(self.selection)
            .cloned()
            .unwrap_or_default();
        renderer.draw_text(
            &selected_text,
            rect.x + 10.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            theme::text(),
        );

        // Chevron
        renderer.draw_text(
            "▼",
            rect.x + rect.width - 20.0,
            rect.y + (rect.height - 14.0) / 2.0,
            12.0,
            theme::text_muted(),
        );

        // Interaction (Cycle options on click)
        let on_change = self.on_change.clone();
        let selection = self.selection;
        let count = self.options.len();

        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |_| {
                if count > 0 {
                    (on_change)((selection + 1) % count);
                }
            }),
        );
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let mut max_w = 0.0f32;
        let mut max_h = 0.0f32;
        for opt in &self.options {
            let (w, h) = renderer.measure_text(opt, 14.0);
            max_w = max_w.max(w);
            max_h = max_h.max(h);
        }
        cvkg_core::Size {
            width: proposal.width.unwrap_or(max_w + 40.0).max(120.0),
            height: max_h + 16.0,
        }
    }
}
/// ColorPicker for RGBA color selection
pub struct ColorPicker {
    pub(crate) color: crate::Color,
    pub(crate) on_change: std::sync::Arc<dyn Fn(crate::Color) + Send + Sync>,
}

impl ColorPicker {
    pub fn new(
        color: crate::Color,
        on_change: impl Fn(crate::Color) + Send + Sync + 'static,
    ) -> Self {
        Self {
            color,
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for ColorPicker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.set_aria_role("colorwell");

        // ColorPicker container
        renderer.fill_rounded_rect(rect, 6.0, theme::surface_elevated());
        renderer.stroke_rect(rect, theme::text_dim(), 1.0);

        // Current color preview
        let preview_w = 40.0;
        let preview_rect = Rect {
            x: rect.x + 8.0,
            y: rect.y + 5.0,
            width: preview_w,
            height: rect.height - 10.0,
        };
        renderer.fill_rounded_rect(preview_rect, 2.0, self.color.as_array());
        renderer.stroke_rect(preview_rect, [1.0, 1.0, 1.0, 0.3], 1.0);

        // Color grid (4 colors for demo)
        let colors = [
            crate::Color::BLACK,
            crate::Color::WHITE,
            crate::Color::RED,
            crate::Color::CYAN,
        ];

        let grid_relative_x = 8.0 + preview_w + 12.0;
        let available_w = (rect.width - grid_relative_x - 10.0).max(0.0);
        let cell_w = available_w / 4.0;
        let cell_h = rect.height - 10.0;

        for (i, &col) in colors.iter().enumerate() {
            let cell_rect = Rect {
                x: rect.x + grid_relative_x + (i as f32 * (cell_w + 5.0)),
                y: rect.y + 5.0,
                width: cell_w,
                height: cell_h,
            };

            renderer.fill_rounded_rect(cell_rect, 2.0, col.as_array());

            // Interaction
            let on_change = self.on_change.clone();
            renderer.register_handler(
                "pointerclick",
                std::sync::Arc::new(move |event| {
                    if let cvkg_core::Event::PointerClick { x, .. } = event
                        && x >= cell_rect.x
                        && x <= cell_rect.x + cell_rect.width
                    {
                        (on_change)(col);
                    }
                }),
            );
        }
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(200.0),
            height: 32.0,
        }
    }
}

/// Checkbox component for boolean input.

// =============================================================================
// CHECKBOX — With focus ring
// =============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CheckboxState {
    Unchecked,
    Checked,
    Indeterminate,
}

#[derive(Clone)]
pub struct Checkbox {
    pub(crate) state: CheckboxState,
    pub(crate) label: Option<String>,
    pub(crate) on_change: std::sync::Arc<dyn Fn(bool) + Send + Sync>,
}

impl Checkbox {
    /// Create a new Checkbox.
    pub fn new(is_checked: bool, on_change: impl Fn(bool) + Send + Sync + 'static) -> Self {
        Self {
            state: if is_checked {
                CheckboxState::Checked
            } else {
                CheckboxState::Unchecked
            },
            label: None,
            on_change: std::sync::Arc::new(on_change),
        }
    }

    /// Set the checkbox to an indeterminate state.
    pub fn indeterminate(mut self, is_indeterminate: bool) -> Self {
        if is_indeterminate {
            self.state = CheckboxState::Indeterminate;
        }
        self
    }

    /// Set the label for the checkbox.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl View for Checkbox {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Checkbox");
        renderer.set_aria_role("checkbox");
        renderer.set_aria_label(self.label.as_deref().unwrap_or("Checkbox"));

        let box_size = 18.0;
        let box_rect = Rect {
            x: rect.x,
            y: rect.y + (rect.height - box_size) / 2.0,
            width: box_size,
            height: box_size,
        };

        let is_active = matches!(
            self.state,
            CheckboxState::Checked | CheckboxState::Indeterminate
        );

        let bg = if is_active {
            theme::accent()
        } else {
            theme::surface_elevated()
        };

        renderer.fill_rounded_rect(box_rect, 3.0, bg);
        renderer.stroke_rect(box_rect, theme::text_dim(), 1.0);

        match self.state {
            CheckboxState::Checked => {
                // Draw checkmark using lines (SVG-like path approximation)
                let c = theme::text();
                // Checkmark path: starts mid-left, goes down-right, then up-right.
                renderer.draw_line(
                    box_rect.x + 4.0,
                    box_rect.y + 9.0,
                    box_rect.x + 8.0,
                    box_rect.y + 13.0,
                    c,
                    2.0,
                );
                renderer.draw_line(
                    box_rect.x + 8.0,
                    box_rect.y + 13.0,
                    box_rect.x + 14.0,
                    box_rect.y + 5.0,
                    c,
                    2.0,
                );
            }
            CheckboxState::Indeterminate => {
                // Draw a horizontal dash
                renderer.draw_line(
                    box_rect.x + 4.0,
                    box_rect.y + 9.0,
                    box_rect.x + 14.0,
                    box_rect.y + 9.0,
                    theme::text(),
                    2.0,
                );
            }
            CheckboxState::Unchecked => {}
        }

        if let Some(label) = &self.label {
            renderer.draw_text(
                label,
                box_rect.x + box_size + 8.0,
                rect.y + (rect.height - 14.0) / 2.0,
                14.0,
                theme::text(),
            );
        }

        let is_checked = matches!(self.state, CheckboxState::Checked);
        let on_change = self.on_change.clone();
        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |_| {
                (on_change)(!is_checked);
            }),
        );
        renderer.pop_vnode();
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        _proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let label_width = self
            .label
            .as_ref()
            .map_or(0.0, |l| renderer.measure_text(l, 14.0).0);
        cvkg_core::Size {
            width: 18.0
                + if self.label.is_some() {
                    8.0 + label_width
                } else {
                    0.0
                },
            height: 22.0,
        }
    }
}

/// Radio Group for exclusive selection.#[derive(Clone)]
pub struct RadioGroup<V> {
    options: Vec<(String, V)>,
    selected_index: usize,
    on_change: std::sync::Arc<dyn Fn(usize) + Send + Sync>,
}

impl<V: View + Clone> RadioGroup<V> {
    pub fn new(selected_index: usize, on_change: impl Fn(usize) + Send + Sync + 'static) -> Self {
        Self {
            options: Vec::new(),
            selected_index,
            on_change: std::sync::Arc::new(on_change),
        }
    }
    pub fn option(mut self, label: impl Into<String>, view: V) -> Self {
        self.options.push((label.into(), view));
        self
    }
}

impl<V: View + Clone> View for RadioGroup<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "RadioGroup");
        for (idx, (label, _)) in self.options.iter().enumerate() {
            let item_rect = Rect {
                x: rect.x,
                y: rect.y + idx as f32 * 24.0,
                width: rect.width,
                height: 24.0,
            };
            renderer.push_vnode(item_rect, "RadioItem");

            let dot_radius = if idx == self.selected_index { 5.0 } else { 4.0 };
            renderer.fill_rounded_rect(
                Rect {
                    x: rect.x + 9.0 - dot_radius,
                    y: rect.y + idx as f32 * 24.0 + 12.0 - dot_radius,
                    width: dot_radius * 2.0,
                    height: dot_radius * 2.0,
                },
                dot_radius,
                if idx == self.selected_index {
                    theme::accent()
                } else {
                    theme::surface_elevated()
                },
            );
            if idx != self.selected_index {
                renderer.stroke_rect(
                    Rect {
                        x: rect.x + 9.0 - dot_radius,
                        y: rect.y + idx as f32 * 24.0 + 12.0 - dot_radius,
                        width: dot_radius * 2.0,
                        height: dot_radius * 2.0,
                    },
                    theme::border_strong(),
                    1.0,
                );
            }
            renderer.draw_text(
                label,
                rect.x + 22.0,
                rect.y + idx as f32 * 24.0 + 11.0,
                14.0,
                theme::text(),
            );

            let on_change = self.on_change.clone();
            renderer.register_handler(
                "pointerclick",
                Arc::new(move |_| {
                    on_change(idx);
                }),
            );
            renderer.pop_vnode();
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let max_width = self
            .options
            .iter()
            .map(|(l, _)| renderer.measure_text(l, 14.0).0)
            .fold(0.0, f32::max);
        cvkg_core::Size {
            width: (proposal.width.unwrap_or(max_width + 30.0)).max(max_width + 30.0),
            height: self.options.len() as f32 * 24.0,
        }
    }
}

/// Tabs component for tabbed navigation.#[derive(Clone)]
pub struct Tabs<V> {
    tabs: Vec<(String, V)>,
    selected_index: usize,
}

impl<V: View> Tabs<V> {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            selected_index: 0,
        }
    }
    pub fn tab(mut self, label: impl Into<String>, content: V) -> Self {
        self.tabs.push((label.into(), content));
        self
    }
    pub fn selected(mut self, index: usize) -> Self {
        self.selected_index = index.min(self.tabs.len().saturating_sub(1));
        self
    }
}

impl<V: View> Default for Tabs<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: View> View for Tabs<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Tabs");
        let tab_height = 36.0;
        for (idx, (label, _)) in self.tabs.iter().enumerate() {
            let tab_rect = Rect {
                x: rect.x + idx as f32 * (rect.width / self.tabs.len() as f32),
                y: rect.y,
                width: rect.width / self.tabs.len() as f32,
                height: tab_height,
            };
            let is_selected = idx == self.selected_index;
            renderer.fill_rounded_rect(tab_rect, 6.0, theme::surface_elevated());
            if is_selected {
                renderer.stroke_rect(tab_rect, theme::accent(), 2.0);
            }
            renderer.draw_text(
                label,
                tab_rect.x + 12.0,
                tab_rect.y + (tab_rect.height - 14.0) / 2.0,
                14.0,
                if is_selected {
                    theme::text()
                } else {
                    theme::text_muted()
                },
            );
        }
        if let Some((_, content)) = self.tabs.get(self.selected_index) {
            let content_rect = Rect {
                x: rect.x,
                y: rect.y + tab_height + 8.0,
                width: rect.width,
                height: rect.height - tab_height - 8.0,
            };
            content.render(renderer, content_rect);
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let max_h = self
            .tabs
            .iter()
            .map(|(_, c)| c.intrinsic_size(renderer, proposal).height)
            .fold(0.0, f32::max);
        cvkg_core::Size {
            width: proposal.width.unwrap_or(300.0),
            height: 36.0 + 8.0 + max_h,
        }
    }
}

// =============================================================================
// SELECT — With keyboard navigation and focus ring
// =============================================================================

#[derive(Clone)]
pub struct HringrPagination {
    pub current_page: usize,
    pub total_pages: usize,
    pub on_change: std::sync::Arc<dyn Fn(usize) + Send + Sync>,
}

impl HringrPagination {
    /// Creates a new HringrPagination.
    pub fn new(total_pages: usize, on_change: impl Fn(usize) + Send + Sync + 'static) -> Self {
        Self {
            current_page: 1,
            total_pages,
            on_change: std::sync::Arc::new(on_change),
        }
    }

    /// Sets the current page.
    pub fn current_page(mut self, page: usize) -> Self {
        self.current_page = page.clamp(1, self.total_pages);
        self
    }
}

impl View for HringrPagination {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "HringrPagination");

        let btn_w = 32.0;
        let spacing = 4.0;
        let mut current_x = rect.x;

        // 1. Previous Button
        let prev_rect = Rect {
            x: current_x,
            y: rect.y,
            width: btn_w,
            height: rect.height,
        };
        renderer.fill_rounded_rect(prev_rect, 4.0, theme::surface());
        renderer.draw_text(
            "<",
            prev_rect.x + 10.0,
            prev_rect.y + 10.0,
            14.0,
            theme::text(),
        );
        current_x += btn_w + spacing;

        // 2. Page Numbers (Simplified)
        for i in 1..=self.total_pages.min(5) {
            let page_rect = Rect {
                x: current_x,
                y: rect.y,
                width: btn_w,
                height: rect.height,
            };
            let is_selected = i == self.current_page;
            let bg = if is_selected {
                [0.0, 0.8, 1.0, 0.4]
            } else {
                theme::surface()
            };

            renderer.fill_rounded_rect(page_rect, 4.0, bg);
            if is_selected {
                renderer.stroke_rect(page_rect, [0.0, 1.0, 1.0, 0.8], 1.0);
            }

            renderer.draw_text(
                &i.to_string(),
                page_rect.x + 10.0,
                page_rect.y + 10.0,
                13.0,
                [1.0, 1.0, 1.0, 0.9],
            );
            current_x += btn_w + spacing;
        }

        // 3. Next Button
        let next_rect = Rect {
            x: current_x,
            y: rect.y,
            width: btn_w,
            height: rect.height,
        };
        renderer.fill_rounded_rect(next_rect, 4.0, theme::surface());
        renderer.draw_text(
            ">",
            next_rect.x + 10.0,
            next_rect.y + 10.0,
            14.0,
            theme::text(),
        );

        renderer.pop_vnode();
    }
}

/// ValhallaRating - A tactical rating component for assessing quality.
/// Named after Valhalla, where the chosen are assessed for their worth.#[derive(Clone, Copy)]
#[derive(Clone)]
pub struct ValhallaRating {
    pub value: f32,
    pub max: usize,
}

impl ValhallaRating {
    /// Creates a new ValhallaRating.
    pub fn new(value: f32) -> Self {
        Self { value, max: 5 }
    }

    /// Sets the maximum rating value.
    pub fn max(mut self, max: usize) -> Self {
        self.max = max;
        self
    }
}

impl View for ValhallaRating {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ValhallaRating");
        renderer.set_aria_role("slider");
        renderer.set_aria_label("Rating");

        let t = renderer.elapsed_time();
        let star_w = rect.width / self.max as f32;
        let star_h = rect.height;

        for i in 0..self.max {
            let star_rect = Rect {
                x: rect.x + i as f32 * star_w,
                y: rect.y,
                width: star_w * 0.8,
                height: star_h,
            };

            let is_filled = (i as f32) < self.value;

            // 1. Bifrost Resonance (Glowing Star Spirits)
            let resonance = if is_filled {
                (t * 2.0 + i as f32).sin() * 0.2 + 0.8
            } else {
                1.0
            };
            let color = if is_filled {
                [1.0, 0.84, 0.0, 0.9 * resonance] // Viking Gold with resonance
            } else {
                [0.2, 0.2, 0.25, 0.3] // Dimmed stone
            };

            renderer.fill_ellipse(star_rect, color);
            if is_filled {
                // Einherjar Spirit Glow
                renderer.stroke_ellipse(star_rect, [1.0, 1.0, 0.5, 0.4 * resonance], 1.5);
            }
        }

        renderer.pop_vnode();
    }
}

/// BifrostColorPicker - A color selection component.
/// Named after the Bifrost, the rainbow bridge connecting the realms.#[derive(Clone, Copy)]
#[derive(Clone)]
pub struct BifrostColorPicker {
    pub color: [f32; 4],
}

impl BifrostColorPicker {
    /// Creates a new BifrostColorPicker.
    pub fn new(color: [f32; 4]) -> Self {
        Self { color }
    }
}

impl View for BifrostColorPicker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "BifrostColorPicker");

        // 1. Rainbow Track (Bifrost Bridge)
        let track_h = rect.height * 0.2;
        let track_rect = Rect {
            x: rect.x,
            y: rect.y + (rect.height - track_h) / 2.0,
            width: rect.width,
            height: track_h,
        };

        // Mocking a rainbow gradient with segments
        let segments = 6;
        let seg_w = rect.width / segments as f32;
        let colors = [
            theme::error_color(),
            theme::warning(),
            theme::warning(),
            theme::success(),
            theme::info(),
            theme::secondary(),
        ];

        for i in 0..segments {
            renderer.fill_rect(
                Rect {
                    x: rect.x + i as f32 * seg_w,
                    y: track_rect.y,
                    width: seg_w,
                    height: track_h,
                },
                colors[i],
            );
        }

        // 2. Mimir's Refraction (Refractive Color Indicator)
        // Heimdall's Watch: A magnifying glass effect over the selection
        let indicator_size = rect.height * 0.9;
        let indicator_rect = Rect {
            x: rect.x + (rect.width - indicator_size) / 2.0,
            y: rect.y + (rect.height - indicator_size) / 2.0,
            width: indicator_size,
            height: indicator_size,
        };

        // Advanced refractive lensing
        renderer.bifrost(indicator_rect, indicator_size / 2.0, 2.0, 0.98);
        renderer.fill_ellipse(indicator_rect, self.color);

        // Surtur's Reactive Materials (Glow Ring)
        let t = renderer.elapsed_time();
        let pulse = (t * 3.0).sin() * 0.1 + 0.9;
        renderer.stroke_ellipse(indicator_rect, [1.0, 1.0, 1.0, 0.7 * pulse], 2.0);

        renderer.pop_vnode();
    }
}

// --- GeriTransfer ---
use cvkg_core::Size;
use cvkg_core::layout::SizeProposal;
#[derive(Clone)]
pub struct GeriTransfer<T> {
    left_items: Vec<T>,
    right_items: Vec<T>,
}

impl<T: Clone> GeriTransfer<T> {
    pub fn new(left: &[T], right: &[T]) -> Self {
        Self {
            left_items: left.to_vec(),
            right_items: right.to_vec(),
        }
    }
}

impl<T: Clone + View> View for GeriTransfer<T> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "GeriTransfer");
        renderer.fill_rounded_rect(rect, 4.0, [0.1, 0.1, 0.15, 1.0]);

        let half_w = rect.width / 2.0;
        let left_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: half_w,
            height: rect.height,
        };
        let right_rect = Rect {
            x: rect.x + half_w,
            y: rect.y,
            width: half_w,
            height: rect.height,
        };

        // Draw separator
        renderer.draw_line(
            rect.x + half_w,
            rect.y,
            rect.x + half_w,
            rect.y + rect.height,
            [0.3, 0.3, 0.3, 1.0],
            1.0,
        );

        let item_h = 30.0;
        let mut y_offset = 10.0;
        for item in &self.left_items {
            let item_rect = Rect {
                x: left_rect.x + 10.0,
                y: left_rect.y + y_offset,
                width: left_rect.width - 20.0,
                height: item_h,
            };
            item.render(renderer, item_rect);
            y_offset += item_h + 5.0;
        }

        let mut y_offset = 10.0;
        for item in &self.right_items {
            let item_rect = Rect {
                x: right_rect.x + 10.0,
                y: right_rect.y + y_offset,
                width: right_rect.width - 20.0,
                height: item_h,
            };
            item.render(renderer, item_rect);
            y_offset += item_h + 5.0;
        }

        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: 400.0,
            height: 300.0,
        }
    }
}
