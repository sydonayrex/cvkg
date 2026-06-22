use crate::theme;
use cvkg_core::{Event, Never, Rect, Renderer, View};

/// Internal scroll state stored in the system state map.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollState {
    pub scroll_offset: [f32; 2],
    pub momentum_velocity: [f32; 2],
    pub scrollbar_opacity: f32,
    pub last_scroll_frame: u32,
    pub is_dragging: bool,
    pub last_pointer_pos: [f32; 2],
    pub is_scrollbar_dragging_v: bool,
    pub is_scrollbar_dragging_h: bool,
    pub scrollbar_drag_offset: f32,
    pub spring_x: Option<cvkg_core::SpringSolver>,
    pub spring_y: Option<cvkg_core::SpringSolver>,
    /// Current zoom level from pinch gestures (1.0 = normal).
    pub zoom_level: f32,
    /// Sleipnir spring for smooth zoom animation.
    pub zoom_spring: Option<cvkg_core::SpringSolver>,
    /// Minimum allowed zoom from pinch.
    pub min_zoom: f32,
    /// Maximum allowed zoom from pinch.
    pub max_zoom: f32,
}

impl Default for ScrollState {
    fn default() -> Self {
        Self {
            scroll_offset: [0.0, 0.0],
            momentum_velocity: [0.0, 0.0],
            scrollbar_opacity: 0.0,
            last_scroll_frame: 0,
            is_dragging: false,
            last_pointer_pos: [0.0, 0.0],
            is_scrollbar_dragging_v: false,
            is_scrollbar_dragging_h: false,
            scrollbar_drag_offset: 0.0,
            spring_x: None,
            spring_y: None,
            zoom_level: 1.0,
            zoom_spring: None,
            min_zoom: 0.25,
            max_zoom: 4.0,
        }
    }
}

/// Scrollable container for content that exceeds available space.
///
/// # Contract
/// Manages viewport coordinates, touch-based dragging, pinch zoom gestures, mouse wheel scroll events, and scrollbar drawing.
#[derive(Clone)]
pub struct ScrollView<V> {
    pub(crate) content: V,
    pub(crate) scroll_id: u64,
    pub(crate) content_size: [f32; 2],
    pub(crate) scrollbar_width: f32,
    pub(crate) scroll_speed: f32,
    pub(crate) momentum_decay: f32,
    pub(crate) scrollbar_fade_delay: u32,
    pub(crate) scrollbar_fade_speed: f32,
}

impl<V: View> ScrollView<V> {
    /// Create a new ScrollView wrapping `content`.
    pub fn new(content: V) -> Self {
        Self {
            content,
            scroll_id: 0,
            content_size: [0.0, 0.0],
            scrollbar_width: 6.0,
            scroll_speed: 1.0,
            momentum_decay: 0.92,
            scrollbar_fade_delay: 60,
            scrollbar_fade_speed: 0.85,
        }
    }

    /// Set a unique ID for this scroll view's state in the system state map.
    pub fn scroll_id(mut self, id: u64) -> Self {
        self.scroll_id = id;
        self
    }

    /// Set the content size hint for scrollbar calculations.
    pub fn content_size(mut self, width: f32, height: f32) -> Self {
        self.content_size = [width, height];
        self
    }

    /// Set scroll speed multiplier for wheel events (default 1.0).
    pub fn scroll_speed(mut self, speed: f32) -> Self {
        self.scroll_speed = speed;
        self
    }

    /// Set momentum decay factor (default 0.92). Higher = more inertia.
    pub fn momentum_decay(mut self, decay: f32) -> Self {
        self.momentum_decay = decay.clamp(0.0, 0.999);
        self
    }

    /// Set scrollbar width in pixels (default 6.0).
    pub fn scrollbar_width(mut self, width: f32) -> Self {
        self.scrollbar_width = width.max(2.0);
        self
    }

    /// Set how many frames the scrollbar stays visible after scrolling (default 60).
    pub fn scrollbar_fade_delay(mut self, frames: u32) -> Self {
        self.scrollbar_fade_delay = frames;
        self
    }

    /// Set scrollbar fade-out speed per frame (default 0.85).
    pub fn scrollbar_fade_speed(mut self, speed: f32) -> Self {
        self.scrollbar_fade_speed = speed.clamp(0.0, 1.0);
        self
    }

    fn read_state(&self) -> ScrollState {
        if self.scroll_id == 0 {
            return ScrollState::default();
        }
        let state = cvkg_core::load_system_state();
        state
            .get_component_state::<ScrollState>(self.scroll_id)
            .and_then(|guard| guard.read().ok().map(|v| *v))
            .unwrap_or_default()
    }

    fn write_state(&self, new_state: ScrollState) {
        if self.scroll_id == 0 {
            return;
        }
        cvkg_core::update_system_state(move |s| {
            let mut s = s.clone();
            s.set_component_state(self.scroll_id, new_state);
            s
        });
    }

    fn tick_physics(
        state: &mut ScrollState,
        viewport_w: f32,
        viewport_h: f32,
        content_w: f32,
        content_h: f32,
        dt: f32,
        decay: f32,
    ) {
        let max_x = (content_w - viewport_w).max(0.0);
        let max_y = (content_h - viewport_h).max(0.0);

        if state.is_dragging || state.is_scrollbar_dragging_v || state.is_scrollbar_dragging_h {
            state.spring_x = None;
            state.spring_y = None;
            return;
        }

        if state.scroll_offset[0] < 0.0 || state.scroll_offset[0] > max_x {
            let target = if state.scroll_offset[0] < 0.0 {
                0.0
            } else {
                max_x
            };
            let mut solver = state.spring_x.unwrap_or_else(|| {
                cvkg_core::SpringSolver::new(
                    cvkg_core::SpringParams::fluid(),
                    target,
                    state.scroll_offset[0],
                )
            });
            solver.set_target(target);
            state.scroll_offset[0] = solver.tick(dt);
            state.momentum_velocity[0] = 0.0;
            if solver.is_settled() {
                state.scroll_offset[0] = target;
                state.spring_x = None;
            } else {
                state.spring_x = Some(solver);
            }
        } else {
            state.spring_x = None;
            if state.momentum_velocity[0].abs() > 0.01 {
                state.scroll_offset[0] += state.momentum_velocity[0] * dt * 60.0;
                state.momentum_velocity[0] *= decay;
            } else {
                state.momentum_velocity[0] = 0.0;
            }
        }

        if state.scroll_offset[1] < 0.0 || state.scroll_offset[1] > max_y {
            let target = if state.scroll_offset[1] < 0.0 {
                0.0
            } else {
                max_y
            };
            let mut solver = state.spring_y.unwrap_or_else(|| {
                cvkg_core::SpringSolver::new(
                    cvkg_core::SpringParams::fluid(),
                    target,
                    state.scroll_offset[1],
                )
            });
            solver.set_target(target);
            state.scroll_offset[1] = solver.tick(dt);
            state.momentum_velocity[1] = 0.0;
            if solver.is_settled() {
                state.scroll_offset[1] = target;
                state.spring_y = None;
            } else {
                state.spring_y = Some(solver);
            }
        } else {
            state.spring_y = None;
            if state.momentum_velocity[1].abs() > 0.01 {
                state.scroll_offset[1] += state.momentum_velocity[1] * dt * 60.0;
                state.momentum_velocity[1] *= decay;
            } else {
                state.momentum_velocity[1] = 0.0;
            }
        }

        let zoom_target = state.zoom_level.clamp(state.min_zoom, state.max_zoom);
        if (state.zoom_level - zoom_target).abs() > 0.001 {
            let mut solver = state.zoom_spring.unwrap_or_else(|| {
                cvkg_core::SpringSolver::new(
                    cvkg_core::SpringParams::fluid(),
                    zoom_target,
                    state.zoom_level,
                )
            });
            solver.set_target(zoom_target);
            state.zoom_level = solver.tick(dt);
            if solver.is_settled() {
                state.zoom_level = zoom_target;
                state.zoom_spring = None;
            } else {
                state.zoom_spring = Some(solver);
            }
        } else {
            state.zoom_spring = None;
        }
    }

    fn render_scrollbar(
        &self,
        renderer: &mut dyn Renderer,
        rect: Rect,
        content_size: f32,
        viewport_size: f32,
        scroll_pos: f32,
        opacity: f32,
        is_vertical: bool,
    ) {
        if opacity <= 0.001 || content_size <= viewport_size {
            return;
        }

        let sb_w = self.scrollbar_width;
        let track_color = theme::with_alpha(theme::bg(), 0.15 * opacity);
        let thumb_color = theme::with_alpha(theme::text_muted(), 0.6 * opacity);

        let thumb_ratio = viewport_size / content_size;
        let thumb_size = (viewport_size * thumb_ratio).max(24.0);
        let max_scroll = (content_size - viewport_size).max(0.0);
        let thumb_pos = if max_scroll > 0.0 {
            ((scroll_pos / max_scroll) * (viewport_size - thumb_size)).round()
        } else {
            0.0
        };

        if is_vertical {
            let track_rect = Rect {
                x: rect.x + rect.width - sb_w - 2.0,
                y: rect.y + 2.0,
                width: sb_w,
                height: rect.height - 4.0,
            };
            renderer.fill_rounded_rect(track_rect, sb_w / 2.0, track_color);

            let thumb_rect = Rect {
                x: rect.x + rect.width - sb_w - 2.0,
                y: rect.y + 2.0 + thumb_pos,
                width: sb_w,
                height: thumb_size,
            };
            renderer.fill_rounded_rect(thumb_rect, sb_w / 2.0, thumb_color);
        } else {
            let track_rect = Rect {
                x: rect.x + 2.0,
                y: rect.y + rect.height - sb_w - 2.0,
                width: rect.width - 4.0,
                height: sb_w,
            };
            renderer.fill_rounded_rect(track_rect, sb_w / 2.0, track_color);

            let thumb_rect = Rect {
                x: rect.x + 2.0 + thumb_pos,
                y: rect.y + rect.height - sb_w - 2.0,
                width: thumb_size,
                height: sb_w,
            };
            renderer.fill_rounded_rect(thumb_rect, sb_w / 2.0, thumb_color);
        }
    }
}

impl<V: View> View for ScrollView<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let content_w = self.content_size[0];
        let content_h = self.content_size[1];
        let dt = renderer.delta_time();

        let mut state = self.read_state();

        Self::tick_physics(
            &mut state,
            rect.width,
            rect.height,
            content_w,
            content_h,
            dt,
            self.momentum_decay,
        );

        let is_moving = state.momentum_velocity[0].abs() > 0.05
            || state.momentum_velocity[1].abs() > 0.05
            || state.spring_x.is_some()
            || state.spring_y.is_some()
            || state.zoom_spring.is_some()
            || state.is_dragging;

        if is_moving {
            state.scrollbar_opacity = 1.0;
            state.last_scroll_frame = 0;
            renderer.request_redraw();
        } else {
            state.last_scroll_frame += 1;
            if state.last_scroll_frame > self.scrollbar_fade_delay {
                state.scrollbar_opacity *= self.scrollbar_fade_speed;
                if state.scrollbar_opacity < 0.01 {
                    state.scrollbar_opacity = 0.0;
                } else {
                    renderer.request_redraw();
                }
            }
        }

        self.write_state(state);

        if self.scroll_id != 0 {
            let scroll_id = self.scroll_id;
            let speed = self.scroll_speed;
            let decay = self.momentum_decay;
            let sb_w = self.scrollbar_width;

            renderer.register_handler(
                "pointerwheel",
                std::sync::Arc::new(move |event| {
                    if let Event::PointerWheel {
                        delta_x, delta_y, ..
                    } = event
                    {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            let mut st = s
                                .get_component_state::<ScrollState>(scroll_id)
                                .and_then(|g| g.read().ok().map(|v| *v))
                                .unwrap_or_default();

                            let max_scroll_x = (content_w - rect.width).max(0.0);
                            let max_scroll_y = (content_h - rect.height).max(0.0);

                            let mut dx = delta_x * speed;
                            let mut dy = delta_y * speed;

                            if (st.scroll_offset[0] <= 0.0 && dx < 0.0)
                                || (st.scroll_offset[0] >= max_scroll_x && dx > 0.0)
                            {
                                dx *= 0.35;
                            }
                            if (st.scroll_offset[1] <= 0.0 && dy < 0.0)
                                || (st.scroll_offset[1] >= max_scroll_y && dy > 0.0)
                            {
                                dy *= 0.35;
                            }

                            st.scroll_offset[0] += dx;
                            st.scroll_offset[1] += dy;

                            st.momentum_velocity[0] += dx * 0.5;
                            st.momentum_velocity[1] += dy * 0.5;
                            st.scrollbar_opacity = 1.0;
                            st.last_scroll_frame = 0;
                            s.set_component_state(scroll_id, st);
                            s
                        });
                    }
                }),
            );

            renderer.register_handler(
                "pointerdown",
                std::sync::Arc::new(move |event| {
                    if let Event::PointerDown { x, y, .. } = event {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            let mut st = s
                                .get_component_state::<ScrollState>(scroll_id)
                                .and_then(|g| g.read().ok().map(|v| *v))
                                .unwrap_or_default();

                            let max_scroll_x = (content_w - rect.width).max(0.0);
                            let max_scroll_y = (content_h - rect.height).max(0.0);

                            let track_rect_v = Rect {
                                x: rect.x + rect.width - sb_w - 2.0,
                                y: rect.y + 2.0,
                                width: sb_w,
                                height: rect.height - 4.0,
                            };
                            let is_on_v = x >= track_rect_v.x
                                && x <= track_rect_v.x + track_rect_v.width
                                && y >= track_rect_v.y
                                && y <= track_rect_v.y + track_rect_v.height;

                            let track_rect_h = Rect {
                                x: rect.x + 2.0,
                                y: rect.y + rect.height - sb_w - 2.0,
                                width: rect.width - 4.0,
                                height: sb_w,
                            };
                            let is_on_h = x >= track_rect_h.x
                                && x <= track_rect_h.x + track_rect_h.width
                                && y >= track_rect_h.y
                                && y <= track_rect_h.y + track_rect_h.height;

                            if is_on_v {
                                let thumb_ratio = rect.height / content_h;
                                let thumb_h = (rect.height * thumb_ratio).max(24.0);
                                let thumb_pos_y = ((st.scroll_offset[1] / max_scroll_y)
                                    * (rect.height - thumb_h))
                                    .round();
                                let thumb_start_y = rect.y + 2.0 + thumb_pos_y;
                                let inside_thumb =
                                    y >= thumb_start_y && y <= thumb_start_y + thumb_h;

                                if inside_thumb {
                                    st.is_scrollbar_dragging_v = true;
                                } else {
                                    let click_y_relative = (y - track_rect_v.y - thumb_h / 2.0)
                                        / (track_rect_v.height - thumb_h);
                                    st.scroll_offset[1] =
                                        (click_y_relative * max_scroll_y).clamp(0.0, max_scroll_y);
                                    st.is_scrollbar_dragging_v = true;
                                }
                                st.last_pointer_pos = [x, y];
                            } else if is_on_h {
                                let thumb_ratio = rect.width / content_w;
                                let thumb_w = (rect.width * thumb_ratio).max(24.0);
                                let thumb_pos_x = ((st.scroll_offset[0] / max_scroll_x)
                                    * (rect.width - thumb_w))
                                    .round();
                                let thumb_start_x = rect.x + 2.0 + thumb_pos_x;
                                let inside_thumb =
                                    x >= thumb_start_x && x <= thumb_start_x + thumb_w;

                                if inside_thumb {
                                    st.is_scrollbar_dragging_h = true;
                                } else {
                                    let click_x_relative = (x - track_rect_h.x - thumb_w / 2.0)
                                        / (track_rect_h.width - thumb_w);
                                    st.scroll_offset[0] =
                                        (click_x_relative * max_scroll_x).clamp(0.0, max_scroll_x);
                                    st.is_scrollbar_dragging_h = true;
                                }
                                st.last_pointer_pos = [x, y];
                            } else {
                                st.is_dragging = true;
                                st.last_pointer_pos = [x, y];
                                st.momentum_velocity = [0.0, 0.0];
                            }

                            st.scrollbar_opacity = 1.0;
                            st.last_scroll_frame = 0;
                            s.set_component_state(scroll_id, st);
                            s
                        });
                    }
                }),
            );

            renderer.register_handler(
                "pointermove",
                std::sync::Arc::new(move |event| {
                    if let Event::PointerMove { x, y, .. } = event {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            let mut st = s
                                .get_component_state::<ScrollState>(scroll_id)
                                .and_then(|g| g.read().ok().map(|v| *v))
                                .unwrap_or_default();

                            let max_scroll_x = (content_w - rect.width).max(0.0);
                            let max_scroll_y = (content_h - rect.height).max(0.0);

                            if st.is_dragging {
                                let mut dx = st.last_pointer_pos[0] - x;
                                let mut dy = st.last_pointer_pos[1] - y;

                                if (st.scroll_offset[0] <= 0.0 && dx < 0.0)
                                    || (st.scroll_offset[0] >= max_scroll_x && dx > 0.0)
                                {
                                    dx *= 0.3;
                                }
                                if (st.scroll_offset[1] <= 0.0 && dy < 0.0)
                                    || (st.scroll_offset[1] >= max_scroll_y && dy > 0.0)
                                {
                                    dy *= 0.3;
                                }

                                st.scroll_offset[0] += dx;
                                st.scroll_offset[1] += dy;

                                st.momentum_velocity[0] = dx * 0.5;
                                st.momentum_velocity[1] = dy * 0.5;
                                st.last_pointer_pos = [x, y];
                            } else if st.is_scrollbar_dragging_v {
                                let track_h = rect.height - 4.0;
                                let thumb_ratio = rect.height / content_h;
                                let thumb_h = (rect.height * thumb_ratio).max(24.0);
                                let max_travel = track_h - thumb_h;
                                if max_travel > 0.0 {
                                    let delta_y = y - st.last_pointer_pos[1];
                                    let scroll_delta = (delta_y / max_travel) * max_scroll_y;
                                    st.scroll_offset[1] = (st.scroll_offset[1] + scroll_delta)
                                        .clamp(0.0, max_scroll_y);
                                    st.last_pointer_pos = [x, y];
                                }
                            } else if st.is_scrollbar_dragging_h {
                                let track_w = rect.width - 4.0;
                                let thumb_ratio = rect.width / content_w;
                                let thumb_w = (rect.width * thumb_ratio).max(24.0);
                                let max_travel = track_w - thumb_w;
                                if max_travel > 0.0 {
                                    let delta_x = x - st.last_pointer_pos[0];
                                    let scroll_delta = (delta_x / max_travel) * max_scroll_x;
                                    st.scroll_offset[0] = (st.scroll_offset[0] + scroll_delta)
                                        .clamp(0.0, max_scroll_x);
                                    st.last_pointer_pos = [x, y];
                                }
                            }

                            st.scrollbar_opacity = 1.0;
                            st.last_scroll_frame = 0;
                            s.set_component_state(scroll_id, st);
                            s
                        });
                    }
                }),
            );

            renderer.register_handler(
                "pointerup",
                std::sync::Arc::new(move |_| {
                    cvkg_core::update_system_state(move |s| {
                        let mut s = s.clone();
                        let mut st = s
                            .get_component_state::<ScrollState>(scroll_id)
                            .and_then(|g| g.read().ok().map(|v| *v))
                            .unwrap_or_default();
                        st.is_dragging = false;
                        st.is_scrollbar_dragging_v = false;
                        st.is_scrollbar_dragging_h = false;
                        st.momentum_velocity[0] *= decay;
                        st.momentum_velocity[1] *= decay;
                        s.set_component_state(scroll_id, st);
                        s
                    });
                }),
            );

            renderer.register_handler(
                "gesturepinch",
                std::sync::Arc::new(move |event| {
                    if let Event::GesturePinch {
                        center,
                        scale,
                        velocity,
                        phase,
                    } = event
                    {
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            let mut st = s
                                .get_component_state::<ScrollState>(scroll_id)
                                .and_then(|g| g.read().ok().map(|v| *v))
                                .unwrap_or_default();

                            let pinch_center_x = center[0];
                            let pinch_center_y = center[1];

                            let zoom_delta = scale - 1.0;
                            let velocity_boost = velocity * 0.1;
                            let new_zoom = st.zoom_level * (1.0 + zoom_delta + velocity_boost);
                            st.zoom_level = new_zoom.clamp(st.min_zoom, st.max_zoom);

                            let zoom_ratio = st.zoom_level / (st.zoom_level - zoom_delta).max(0.01);
                            st.scroll_offset[0] = pinch_center_x
                                - (pinch_center_x - st.scroll_offset[0]) * zoom_ratio;
                            st.scroll_offset[1] = pinch_center_y
                                - (pinch_center_y - st.scroll_offset[1]) * zoom_ratio;

                            st.momentum_velocity = [0.0, 0.0];
                            st.scrollbar_opacity = 1.0;
                            st.last_scroll_frame = 0;

                            if phase == cvkg_core::TouchPhase::Ended
                                || phase == cvkg_core::TouchPhase::Cancelled
                            {
                                st.zoom_level = (st.zoom_level * 4.0).round() / 4.0;
                                st.momentum_velocity = [0.0, 0.0];
                            }

                            s.set_component_state(scroll_id, st);
                            s
                        });
                    }
                }),
            );

            renderer.register_handler(
                "keydown",
                std::sync::Arc::new(move |event| {
                    if let Event::KeyDown { key, .. } = event {
                        let scroll_amount = match key.as_str() {
                            "PageDown" => 100.0,
                            "PageUp" => -100.0,
                            "End" => f32::MAX,
                            "Home" => f32::MIN,
                            "ArrowDown" => 40.0,
                            "ArrowUp" => -40.0,
                            "ArrowRight" => 40.0,
                            "ArrowLeft" => -40.0,
                            _ => return,
                        };
                        cvkg_core::update_system_state(move |s| {
                            let mut s = s.clone();
                            let mut st = s
                                .get_component_state::<ScrollState>(scroll_id)
                                .and_then(|g| g.read().ok().map(|v| *v))
                                .unwrap_or_default();
                            let max_y = (content_h - rect.height).max(0.0);
                            let max_x = (content_w - rect.width).max(0.0);
                            if scroll_amount == f32::MAX {
                                st.scroll_offset[1] = max_y;
                            } else if scroll_amount == f32::MIN {
                                st.scroll_offset[0] = 0.0;
                                st.scroll_offset[1] = 0.0;
                            } else {
                                match key.as_str() {
                                    "ArrowLeft" | "ArrowRight" => {
                                        st.scroll_offset[0] =
                                            (st.scroll_offset[0] + scroll_amount).clamp(0.0, max_x);
                                    }
                                    _ => {
                                        st.scroll_offset[1] =
                                            (st.scroll_offset[1] + scroll_amount).clamp(0.0, max_y);
                                    }
                                }
                            }
                            st.momentum_velocity = [0.0, 0.0];
                            st.scrollbar_opacity = 1.0;
                            st.last_scroll_frame = 0;
                            s.set_component_state(scroll_id, st);
                            s
                        });
                    }
                }),
            );
        }

        renderer.push_clip_rect(rect);
        let zoom = state.zoom_level;
        let cx = rect.x + rect.width / 2.0;
        let cy = rect.y + rect.height / 2.0;
        renderer.push_transform(
            [
                -state.scroll_offset[0].round() + cx - cx * zoom,
                -state.scroll_offset[1].round() + cy - cy * zoom,
            ],
            [zoom, zoom],
            0.0,
        );

        let content_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: if content_w > 0.0 {
                content_w
            } else {
                rect.width
            },
            height: if content_h > 0.0 {
                content_h
            } else {
                rect.height
            },
        };
        self.content.render(renderer, content_rect);

        renderer.pop_transform();
        renderer.pop_clip_rect();

        let needs_v_scrollbar = content_h > rect.height;
        let needs_h_scrollbar = content_w > rect.width;

        if needs_v_scrollbar {
            self.render_scrollbar(
                renderer,
                rect,
                content_h,
                rect.height,
                state.scroll_offset[1],
                state.scrollbar_opacity,
                true,
            );
        }

        if needs_h_scrollbar {
            self.render_scrollbar(
                renderer,
                rect,
                content_w,
                rect.width,
                state.scroll_offset[0],
                state.scrollbar_opacity,
                false,
            );
        }
    }
}
