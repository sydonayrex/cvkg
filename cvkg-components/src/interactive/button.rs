use crate::theme;
use crate::{ButtonSize, ButtonVariant, FONT_BASE, RADIUS_MD};
use cvkg_core::{AriaProperties, AriaRole, Event, KeyModifiers, Never, Rect, Renderer, View};
use std::sync::Arc;

/// Button with action callback, variant styling, size options, and disabled state.
#[derive(Clone)]
pub struct Button {
    pub(crate) label: String,
    pub(crate) on_click: Arc<dyn Fn() + Send + Sync>,
    pub(crate) variant: ButtonVariant,
    pub(crate) size: ButtonSize,
    pub(crate) disabled: bool,
    pub(crate) loading: bool,
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
            loading: false,
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

    /// Set the loading state. When loading, the button shows a spinner
    /// and click events are suppressed.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Compute the background color based on variant and state.
    fn bg_color(&self, is_pressed: bool, is_hovered: bool) -> [f32; 4] {
        if self.disabled || self.loading {
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
                    theme::active_color()
                } else if is_hovered {
                    theme::hover()
                } else {
                    theme::button_ghost_bg()
                }
            }
            ButtonVariant::Link => theme::button_ghost_bg(),
            ButtonVariant::Glass => {
                if is_pressed {
                    theme::surface_elevated()
                } else if is_hovered {
                    theme::surface()
                } else {
                    theme::button_ghost_bg()
                }
            }
            ButtonVariant::TintedGlass => {
                let accent = theme::accent();
                if is_pressed {
                    [
                        accent[0] * 0.3 + 0.1,
                        accent[1] * 0.3 + 0.1,
                        accent[2] * 0.3 + 0.1,
                        0.9,
                    ]
                } else if is_hovered {
                    [
                        accent[0] * 0.2 + 0.08,
                        accent[1] * 0.2 + 0.08,
                        accent[2] * 0.2 + 0.08,
                        0.85,
                    ]
                } else {
                    [
                        accent[0] * 0.15 + 0.05,
                        accent[1] * 0.15 + 0.05,
                        accent[2] * 0.15 + 0.05,
                        0.75,
                    ]
                }
            }
            ButtonVariant::Capsule => {
                let accent = theme::accent();
                if is_pressed {
                    [accent[0] * 0.8, accent[1] * 0.8, accent[2] * 0.8, 1.0]
                } else if is_hovered {
                    [accent[0] * 0.9, accent[1] * 0.9, accent[2] * 0.9, 1.0]
                } else {
                    accent
                }
            }
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
                    (theme::border_strong(), 1.0)
                } else {
                    (theme::button_ghost_bg(), 0.0)
                }
            }
            ButtonVariant::Link => (theme::button_ghost_bg(), 0.0),
            ButtonVariant::Glass => {
                if is_hovered {
                    (theme::border_strong(), 1.0)
                } else {
                    (theme::border(), 1.0)
                }
            }
            ButtonVariant::TintedGlass => {
                let accent = theme::accent();
                if is_hovered {
                    ([accent[0], accent[1], accent[2], 0.25], 1.0)
                } else {
                    ([accent[0], accent[1], accent[2], 0.12], 1.0)
                }
            }
            ButtonVariant::Capsule => {
                let accent = theme::accent();
                if is_pressed {
                    (
                        [accent[0] * 0.7, accent[1] * 0.7, accent[2] * 0.7, 0.3],
                        1.0,
                    )
                } else {
                    (
                        [accent[0] * 0.5, accent[1] * 0.5, accent[2] * 0.5, 0.2],
                        1.0,
                    )
                }
            }
        }
    }

    /// Compute the text color based on variant and state.
    fn text_color(&self, is_hovered: bool) -> [f32; 4] {
        if self.disabled {
            return theme::disabled_text();
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
            ButtonVariant::Glass => theme::text(),
            ButtonVariant::TintedGlass => theme::text(),
            ButtonVariant::Capsule => theme::text(),
        }
    }

    /// Compute the height based on size variant.
    fn height(&self) -> f32 {
        match self.size {
            ButtonSize::Small => 44.0,
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
                let mut solver = solver_arc.write().expect("lock poisoned");
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
                let mut solver = solver_arc.write().expect("lock poisoned");
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

        let intensity = if self.disabled || self.loading { 0.0 } else { 0.25 };
        let mut offset_x = 0.0;
        let mut offset_y = 0.0;
        if dist < radius && dist > 0.0 && !self.disabled && !self.loading {
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
        if !self.disabled && !self.loading {
            let glow_color = [
                theme::accent()[0],
                theme::accent()[1],
                theme::accent()[2],
                0.8 * proximity,
            ];
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
            renderer.push_shadow(1.0, theme::shadow(), [0.0, 1.0]);
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

        // Focus ring -- WCAG 2.4.7
        if is_focused && !self.disabled && !self.loading {
            crate::draw_focus_ring(renderer, final_rect);
        }

        // Label text centered or spinner when loading
        if self.loading {
            // Spinning indicator: draw arc segments with varying opacity
            let time = renderer.elapsed_time();
            let cx = final_rect.x + final_rect.width / 2.0;
            let cy = final_rect.y + final_rect.height / 2.0;
            let spinner_radius = font_size * 0.4;
            let segments = 8u32;
            let rotation = time * 4.0; // radians per second
            for i in 0..segments {
                let angle = rotation + (i as f32 / segments as f32) * std::f32::consts::TAU;
                let alpha = 0.15 + 0.85 * (i as f32 / segments as f32);
                let dx = angle.cos();
                let dy = angle.sin();
                let start_dist = spinner_radius * 0.55;
                let end_dist = spinner_radius;
                renderer.draw_line(
                    cx + dx * start_dist,
                    cy + dy * start_dist,
                    cx + dx * end_dist,
                    cy + dy * end_dist,
                    [theme::accent()[0], theme::accent()[1], theme::accent()[2], alpha],
                    2.0,
                );
            }
        } else {
            // Label text centered
            let (tw, _th) = renderer.measure_text(&self.label, font_size);
            let text_x = final_rect.x + (final_rect.width - tw) / 2.0;
            let text_y = final_rect.y + (final_rect.height - font_size) / 2.0;
            renderer.draw_text(&self.label, text_x, text_y, font_size, text_color);
        }

        // Register interaction handlers
        if !self.disabled && !self.loading {
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

        let is_disabled = self.disabled || self.loading;
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
        let (tw, _th) = if self.loading {
            (font_size, font_size)
        } else {
            renderer.measure_text(&self.label, font_size)
        };
        let h_pad = self.h_padding();
        cvkg_core::Size {
            width: (tw + h_pad * 2.0).max(self.height()),
            height: self.height(),
        }
    }

    fn aria_properties(&self) -> Option<AriaProperties> {
        Some(AriaProperties::new(AriaRole::Button, &self.label).disabled(self.disabled || self.loading))
    }

    fn on_key_event(&self, key: &str, _modifiers: KeyModifiers) -> bool {
        if self.disabled || self.loading {
            return false;
        }
        match key {
            "Enter" | " " => {
                (self.on_click)();
                true
            }
            _ => false,
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

        let focus_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            self.label.hash(&mut s);
            "toggle_focus".hash(&mut s);
            s.finish()
        };
        let (is_focused, set_focused) = cvkg_vdom::use_state(focus_hash, false);

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
                let mut solver = solver_arc.write().expect("lock poisoned");
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

        // Focus ring -- WCAG 2.4.7
        if is_focused {
            let total_w = 40.0 + 8.0 + renderer.measure_text(&self.label, 14.0).0;
            let toggle_rect = Rect {
                x: rect.x,
                y: rect.y,
                width: total_w,
                height: rect.height,
            };
            crate::draw_focus_ring(renderer, toggle_rect);
        }
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

    fn aria_properties(&self) -> Option<AriaProperties> {
        Some(
            AriaProperties::new(AriaRole::Switch, &self.label)
                .checked(self.is_on)
                .value(if self.is_on { "on" } else { "off" }),
        )
    }

    fn on_key_event(&self, key: &str, _modifiers: KeyModifiers) -> bool {
        match key {
            "Enter" | " " => {
                (self.on_change)(!self.is_on);
                true
            }
            _ => false,
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
        let val_min = *self.range.start();
        let val_max = *self.range.end();
        let val_range = (val_max - val_min).max(0.001);

        renderer.push_vnode(rect, "Slider");
        renderer.set_aria_role("slider");

        // ARIA value properties for screen readers
        renderer.set_aria_valuemin(val_min);
        renderer.set_aria_valuemax(val_max);
        renderer.set_aria_valuenow(self.value);

        let track_h = 4.0;
        let track_y = rect.y + (rect.height - track_h) / 2.0;
        let track = Rect {
            x: rect.x + 8.0,
            y: track_y,
            width: (rect.width - 16.0).max(0.0),
            height: track_h,
        };

        // Draw track
        renderer.fill_rounded_rect(track, 2.0, theme::surface_elevated());

        let fraction = ((self.value - val_min) / val_range).clamp(0.0, 1.0);
        let thumb_x = track.x + fraction * track.width;

        // Draw active track highlight
        let active_track = Rect {
            x: track.x,
            y: track_y,
            width: fraction * track.width,
            height: track_h,
        };
        renderer.fill_rounded_rect(active_track, 2.0, theme::accent());

        // Draw thumb
        let thumb_size = 16.0;
        let thumb = Rect {
            x: thumb_x - thumb_size / 2.0,
            y: rect.y + (rect.height - thumb_size) / 2.0,
            width: thumb_size,
            height: thumb_size,
        };
        renderer.fill_rounded_rect(thumb, thumb_size / 2.0, theme::text());
        renderer.stroke_rounded_rect(thumb, thumb_size / 2.0, theme::border_strong(), 1.0);

        // ── Pointer interaction ──
        let on_change = self.on_change.clone();
        let step = self.step;
        renderer.register_handler(
            "pointerdown",
            std::sync::Arc::new(move |event| {
                if let Event::PointerDown { x, .. } = event {
                    let relative_x = (x - track.x) / track.width;
                    let mut val = val_min + relative_x.clamp(0.0, 1.0) * val_range;
                    if let Some(s) = step {
                        val = (val / s).round() * s;
                    }
                    (on_change)(val);
                }
            }),
        );

        let on_move = self.on_change.clone();
        renderer.register_handler(
            "pointermove",
            std::sync::Arc::new(move |event| {
                if let Event::PointerMove { x, .. } = event {
                    let relative_x = (x - track.x) / track.width;
                    let mut val = val_min + relative_x.clamp(0.0, 1.0) * val_range;
                    if let Some(s) = step {
                        val = (val / s).round() * s;
                    }
                    (on_move)(val);
                }
            }),
        );

        // ── Keyboard interaction ──
        let on_key_change = self.on_change.clone();
        let key_step = self.step;
        let current_val = self.value;
        renderer.register_handler(
            "keydown",
            std::sync::Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    let delta = match key.as_str() {
                        "ArrowRight" | "ArrowUp" => Some(1.0),
                        "ArrowLeft" | "ArrowDown" => Some(-1.0),
                        "Home" => None, // sentinel: jump to min
                        "End" => None,  // sentinel: jump to max
                        "PageUp" => Some(10.0),
                        "PageDown" => Some(-10.0),
                        _ => return,
                    };

                    let new_val = match key.as_str() {
                        "Home" => val_min,
                        "End" => val_max,
                        _ => {
                            let multiplier = delta.unwrap();
                            let step_size = key_step.unwrap_or(1.0);
                            current_val + multiplier * step_size
                        }
                    };

                    // Clamp to range and snap to step if defined
                    let mut clamped = new_val.clamp(val_min, val_max);
                    if let Some(s) = key_step {
                        clamped = (clamped / s).round() * s;
                        clamped = clamped.clamp(val_min, val_max);
                    }

                    (on_key_change)(clamped);
                }
            }),
        );

        renderer.pop_vnode();
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        _proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: 150.0,
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

        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |event| {
                if let Event::PointerClick { x, .. } = event {
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
        renderer.stroke_rect(rect, theme::text_muted(), 1.0);

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

        // Interaction
        let current_text = std::sync::Arc::new(std::sync::Mutex::new(self.text.clone()));
        let on_change = self.on_change.clone();

        renderer.register_handler(
            "keydown",
            std::sync::Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
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
        };
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
