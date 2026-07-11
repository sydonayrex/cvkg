use cvkg::components::calendar::Date;
use cvkg::components::{
    AutoComplete, BifrostTabs, Breadcrumb, BreadcrumbItem, Combobox, DatePicker, Dialog,
    MimirSpotlight, SpinnerVariant, Toggle,
};
use cvkg::core::{Event, Renderer, View};
use cvkg::prelude::AnyView;
use cvkg::prelude::*;
use cvkg_core::{load_system_state, update_system_state};

// Shared system-state key for the command palette open flag (must match command_palette.rs).
const SPOTLIGHT_OPEN_HASH: u64 = 0xD00_0001;

// Must match DIALOG_OPEN_HASH in cvkg-components/src/container/modal.rs.
const GALLERY_DIALOG_OPEN_HASH: u64 = 0xB00_0001;

// -- Component catalog ------------------------------------------------

struct GalleryEntry {
    name: &'static str,
    category: &'static str,
    render: fn(&GalleryState, &std::sync::Arc<std::sync::Mutex<GalleryState>>) -> AnyView,
}

fn catalog() -> Vec<GalleryEntry> {
    vec![
        // -- Forms --
        GalleryEntry {
            name: "Button",
            category: "Forms",
            render: |_state, _state_arc| {
                AnyView::new(
                    VStack::new(12.0)
                        .child(Button::new("Default Button", || {}))
                        .child(Button::new("Disabled Button", || {}).disabled(true)),
                )
            },
        },
        GalleryEntry {
            name: "Checkbox",
            category: "Forms",
            render: |state, state_arc| {
                let arc1 = state_arc.clone();
                let arc2 = state_arc.clone();
                AnyView::new(
                    VStack::new(8.0)
                        .child(
                            Checkbox::new(state.checkbox_1, move |val| {
                                let mut s = arc1.lock().unwrap_or_else(|e| e.into_inner());
                                s.checkbox_1 = val;
                            })
                            .label("Enable Berserk Mode")
                            .frame(Some(220.0), Some(30.0)),
                        )
                        .child(
                            Checkbox::new(state.checkbox_2, move |val| {
                                let mut s = arc2.lock().unwrap_or_else(|e| e.into_inner());
                                s.checkbox_2 = val;
                            })
                            .label("Auto-charge Rage")
                            .frame(Some(220.0), Some(30.0)),
                        ),
                )
            },
        },
        GalleryEntry {
            name: "Input",
            category: "Forms",
            render: |state, state_arc| {
                let arc = state_arc.clone();
                AnyView::new(
                    VStack::new(12.0)
                        .child(
                            Input::new(state.input_text.as_str()).on_change(move |text| {
                                let mut s = arc.lock().unwrap_or_else(|e| e.into_inner());
                                s.input_text = text;
                            }),
                        )
                        .child(
                            Text::new(format!("Typed: {}", state.input_text))
                                .font_size(14.0)
                                .color([0.7, 0.7, 0.7, 1.0]),
                        ),
                )
            },
        },
        GalleryEntry {
            name: "Toggle",
            category: "Forms",
            render: |state, state_arc| {
                let arc1 = state_arc.clone();
                let arc2 = state_arc.clone();
                AnyView::new(
                    VStack::new(8.0)
                        .child(Toggle::new("Shield Wall", state.toggle_1, move |val| {
                            let mut s = arc1.lock().unwrap_or_else(|e| e.into_inner());
                            s.toggle_1 = val;
                        }))
                        .child(Toggle::new("Odin's Sight", state.toggle_2, move |val| {
                            let mut s = arc2.lock().unwrap_or_else(|e| e.into_inner());
                            s.toggle_2 = val;
                        })),
                )
            },
        },
        GalleryEntry {
            name: "Slider",
            category: "Forms",
            render: |state, state_arc| {
                let arc = state_arc.clone();
                AnyView::new(
                    VStack::new(12.0)
                        .child(
                            Text::new(format!("Volume: {}%", (state.slider_value * 100.0) as i32))
                                .font_size(14.0)
                                .color([0.7, 0.7, 0.7, 1.0]),
                        )
                        .child(Slider::new(state.slider_value, 0.0..=1.0, move |val| {
                            let mut s = arc.lock().unwrap_or_else(|e| e.into_inner());
                            s.slider_value = val;
                        })),
                )
            },
        },
        GalleryEntry {
            name: "Combobox",
            category: "Forms",
            render: |state, state_arc| {
                let arc = state_arc.clone();
                AnyView::new(
                    VStack::new(8.0).child(
                        Combobox::new(vec![
                            "Odin".to_string(),
                            "Tyr".to_string(),
                            "Bor".to_string(),
                        ])
                        .selected(state.combobox_index)
                        .on_change(move |idx| {
                            let mut s = arc.lock().unwrap_or_else(|e| e.into_inner());
                            s.combobox_index = idx;
                        })
                        .frame(Some(220.0), Some(38.0)),
                    ),
                )
            },
        },
        GalleryEntry {
            name: "AutoComplete",
            category: "Forms",
            render: |_state, state_arc| {
                let arc = state_arc.clone();
                let suggestions = vec![
                    "Astrid".to_string(),
                    "Bjorn".to_string(),
                    "Freya".to_string(),
                    "Odin".to_string(),
                    "Thor".to_string(),
                ];
                AnyView::new(
                    VStack::new(8.0).child(
                        AutoComplete::new(
                            "Search warriors...".to_string(),
                            suggestions,
                            move |query| {
                                let mut s = arc.lock().unwrap_or_else(|e| e.into_inner());
                                s.autocomplete_query = query;
                            },
                            move |_selected| {},
                        )
                        .frame(Some(220.0), Some(38.0)),
                    ),
                )
            },
        },
        GalleryEntry {
            name: "DatePicker",
            category: "Forms",
            render: |state, state_arc| {
                let arc = state_arc.clone();
                AnyView::new(
                    VStack::new(8.0)
                        .child(
                            Text::new(format!("Selected: {}", state.selected_date.format()))
                                .font_size(14.0)
                                .color([1.0, 1.0, 1.0, 1.0]),
                        )
                        .child(
                            DatePicker::new(move |d, m, y| {
                                let mut s = arc.lock().unwrap_or_else(|e| e.into_inner());
                                s.selected_date = Date {
                                    year: y as i32,
                                    month: m,
                                    day: d,
                                };
                            })
                            .selected(
                                state.selected_date.day,
                                state.selected_date.month,
                                state.selected_date.year as u32,
                            )
                            .frame(Some(220.0), Some(38.0)),
                        )
                        .child(
                            Text::new("Date picker component")
                                .font_size(11.0)
                                .color([0.6, 0.6, 0.6, 1.0]),
                        ),
                )
            },
        },
        // -- Navigation --
        GalleryEntry {
            name: "Tabs",
            category: "Navigation",
            render: |state, state_arc| {
                let arc_clone = state_arc.clone();
                AnyView::new(BifrostTabs::new(
                    vec![
                        "Shield".to_string(),
                        "Rage".to_string(),
                        "Runes".to_string(),
                    ],
                    state.tab_index,
                    move |idx| {
                        let mut s = arc_clone.lock().unwrap_or_else(|e| e.into_inner());
                        s.tab_index = idx;
                    },
                ))
            },
        },
        GalleryEntry {
            name: "Breadcrumb",
            category: "Navigation",
            render: |_state, _state_arc| {
                AnyView::new(VStack::new(8.0).child(Breadcrumb::new(vec![
                    BreadcrumbItem::new("Home"),
                    BreadcrumbItem::new("Loadout"),
                    BreadcrumbItem::new("Berserker"),
                ])))
            },
        },
        // -- Overlays --
        GalleryEntry {
            name: "Tooltip",
            category: "Overlays",
            render: |_state, _state_arc| {
                AnyView::new(
                    Tooltip::new(
                        AnyView::new(
                            Text::new("Hover target")
                                .font_size(14.0)
                                .color([0.9, 0.9, 0.9, 1.0]),
                        ),
                        "Hidden wisdom: Runes guide the worthy",
                    )
                    .visible(true),
                )
            },
        },
        GalleryEntry {
            name: "Command Palette",
            category: "Overlays",
            render: |state, state_arc| {
                let arc = state_arc.clone();
                AnyView::new(
                    VStack::new(12.0)
                        .child(
                            Text::new(if state.command_palette_open {
                                "Command Palette is OPEN — select an item or click backdrop to close"
                            } else {
                                "Command Palette is CLOSED — click Open to launch it"
                            })
                                .font_size(14.0)
                                .color([1.0, 1.0, 1.0, 0.8]),
                        )
                        .child(
                            Button::new("Open Command Palette", move || {
                                // Mark open in gallery state
                                arc.lock().unwrap_or_else(|e| e.into_inner()).command_palette_open = true;
                                // Drive the MimirSpotlight system state directly.
                                update_system_state(|s| {
                                    let mut s = s.clone();
                                    s.set_component_state(SPOTLIGHT_OPEN_HASH, true);
                                    s
                                });
                            }),
                        )
                        .child(
                            // The palette reads its own open state from system state.
                            // .open() only seeds on first render; after that, state owns it.
                            MimirSpotlight::new()
                                .open()
                                .command("Save File", Some("Ctrl+S"), {
                                    let arc = state_arc.clone();
                                    move || {
                                        arc.lock().unwrap_or_else(|e| e.into_inner()).command_palette_open = false;
                                    }
                                })
                                .command("Open Preferences", Some("Ctrl+P"), {
                                    let arc = state_arc.clone();
                                    move || {
                                        arc.lock().unwrap_or_else(|e| e.into_inner()).command_palette_open = false;
                                    }
                                })
                                .command("Toggle Fullscreen", Some("F11"), {
                                    let arc = state_arc.clone();
                                    move || {
                                        arc.lock().unwrap_or_else(|e| e.into_inner()).command_palette_open = false;
                                    }
                                })
                                .search(state.command_query.as_str()),
                        ),
                )
            },
        },
        GalleryEntry {
            name: "Dialog",
            category: "Overlays",
            render: |_state, _state_arc| {
                // Read the open flag from system state (toggled by the button below
                // and by GeriDialog's own internal close handlers).
                let is_open = load_system_state()
                    .get_component_state::<bool>(GALLERY_DIALOG_OPEN_HASH)
                    .and_then(|v| v.read().ok().map(|g| *g))
                    .unwrap_or(false);

                let dialog = Dialog::new(AnyView::new(
                    VStack::new(8.0)
                        .child(
                            Text::new("This is a modal dialog.")
                                .font_size(14.0)
                                .color([0.9, 0.9, 0.9, 1.0]),
                        )
                        .child(
                            Text::new("Click outside or press Esc to close.")
                                .font_size(12.0)
                                .color([0.6, 0.6, 0.6, 1.0]),
                        ),
                ))
                .presented(is_open)
                .title("Confirm Action")
                .action("Close", || {
                    update_system_state(|s| {
                        let mut s = s.clone();
                        s.set_component_state(GALLERY_DIALOG_OPEN_HASH, false);
                        s
                    });
                });

                AnyView::new(
                    VStack::new(12.0)
                        .child(
                            Text::new("Dialog component demo")
                                .font_size(14.0)
                                .color([0.9, 0.9, 0.9, 1.0]),
                        )
                        .child(Button::new("Open Modal", || {
                            update_system_state(|s| {
                                let mut s = s.clone();
                                s.set_component_state(GALLERY_DIALOG_OPEN_HASH, true);
                                s
                            });
                        }))
                        .child(
                            Text::new("Click Open Modal to see overlay")
                                .font_size(11.0)
                                .color([0.6, 0.6, 0.6, 1.0]),
                        )
                        // Give the dialog a full-area rect so its backdrop covers the
                        // whole preview region (not just its tiny intrinsic size), and
                        // let it render above sibling content via its own high z-index.
                        .child(dialog.flex(1.0)),
                )
            },
        },
        // -- Data Display --
        GalleryEntry {
            name: "Progress",
            category: "Data Display",
            render: |state, _state_arc| {
                let t = state.start_time.elapsed().as_secs_f32();
                let progress = (t % 3.0) / 3.0;
                let pct = (progress * 100.0) as u32;
                AnyView::new(
                    VStack::new(8.0)
                        .child(
                            Text::new(format!("Progress: {}%", pct))
                                .font_size(14.0)
                                .color([0.7, 0.7, 0.7, 1.0]),
                        )
                        .child(Progress::new(progress)),
                )
            },
        },
        GalleryEntry {
            name: "Spinner",
            category: "Data Display",
            render: |_state, _state_arc| {
                let _t = _state.start_time.elapsed().as_secs_f32();
                AnyView::new(
                    HStack::new(8.0)
                        .child(Spinner::new().variant(SpinnerVariant::Ouroboros).size(48.0))
                        .child(
                            Text::new("Loading...")
                                .font_size(14.0)
                                .color([0.7, 0.7, 0.7, 1.0]),
                        ),
                )
            },
        },
        GalleryEntry {
            name: "Alert",
            category: "Feedback",
            render: |_state, _state_arc| {
                AnyView::new(
                    VStack::new(8.0)
                        .child(
                            Text::new("ALERT: Bifrost interference detected")
                                .font_size(14.0)
                                .color([1.0, 0.3, 0.3, 1.0]),
                        )
                        .child(
                            Text::new("Warning: Low runic charge")
                                .font_size(12.0)
                                .color([1.0, 0.7, 0.0, 1.0]),
                        )
                        .child(
                            Text::new("Info: All systems nominal")
                                .font_size(12.0)
                                .color([0.3, 1.0, 0.5, 1.0]),
                        ),
                )
            },
        },
    ]
}

// -- Gallery app state -----------------------------------------------

struct GalleryState {
    selected: usize,
    entries: Vec<GalleryEntry>,
    toggle_1: bool,
    toggle_2: bool,
    checkbox_1: bool,
    checkbox_2: bool,
    slider_value: f32,
    input_text: String,
    start_time: std::time::Instant,
    tab_index: usize,
    combobox_index: Option<usize>,
    autocomplete_query: String,
    selected_date: Date,
    command_query: String,
    command_palette_open: bool,
    /// Accumulated wheel delta; one carousel step fires per notch threshold so a
    /// single momentum/inertial scroll gesture (many small MouseWheel events)
    /// advances the carousel smoothly instead of racing through every card.
    wheel_accum: f32,
    /// Continuous rotation angle for smooth 2.5D carousel animation.
    /// Radians; updated via spring physics toward `target_angle`.
    current_angle: f32,
    /// Target rotation angle (set when `selected` changes).
    target_angle: f32,
    /// Angular velocity for spring animation (radians/sec).
    angular_velocity: f32,
}

impl GalleryState {
    fn new() -> Self {
        Self {
            selected: 0,
            entries: catalog(),
            toggle_1: false,
            toggle_2: true,
            checkbox_1: false,
            checkbox_2: true,
            slider_value: 0.5,
            input_text: "Placeholder text".to_string(),
            start_time: std::time::Instant::now(),
            tab_index: 0,
            combobox_index: None,
            autocomplete_query: String::new(),
            selected_date: Date {
                year: 2026,
                month: 6,
                day: 27,
            },
            command_query: String::new(),
            command_palette_open: true,
            wheel_accum: 0.0,
            current_angle: 0.0,
            target_angle: 0.0,
            angular_velocity: 0.0,
        }
    }
}

// -- Gallery app view ------------------------------------------------

struct GalleryApp {
    state: std::sync::Arc<std::sync::Mutex<GalleryState>>,
}

impl View for GalleryApp {
    type Body = HStack;

    fn body(self) -> Self::Body {
        unreachable!("GalleryApp renders via render(), not body()")
    }

    fn changed(&self) -> bool {
        true
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());

        // ── Spring animation: tick current_angle toward target_angle ──
        // Spring parameters: stiff (snappy) but with enough damping to avoid oscillation.
        const SPRING_STIFFNESS: f32 = 180.0;
        const SPRING_DAMPING: f32 = 22.0;
        let dt = 1.0 / 60.0; // Fixed timestep per frame
        let displacement = state.current_angle - state.target_angle;
        let spring_force = -SPRING_STIFFNESS * displacement;
        let damping_force = -SPRING_DAMPING * state.angular_velocity;
        let acceleration = spring_force + damping_force;
        state.angular_velocity += acceleration * dt;
        state.current_angle += state.angular_velocity * dt;
        // Snap to target when close enough to avoid sub-pixel jitter
        if displacement.abs() < 0.001 && state.angular_velocity.abs() < 0.01 {
            state.current_angle = state.target_angle;
            state.angular_velocity = 0.0;
        }

        // Snapshot the state we need for rendering (immutable borrow)
        let entries = &state.entries;
        let selected = state.selected;
        let num_entries = state.entries.len();
        let current_angle = state.current_angle;

        // Capture the cumulative parent translation so the hit-test closure
        // can convert screen-space click coordinates to local space.
        let translation = renderer.current_translation();

        // 1. Draw Background Area
        renderer.push_vnode(rect, "GalleryApp");
        renderer.set_z_index(1000.0);
        renderer.fill_rect(rect, [0.07, 0.008, 0.008, 1.0]);
        renderer.set_z_index(0.0);

        // 2. Draw 2.5D Carousel (Top Panel)
        // Each card is placed at a continuous angle from the carousel center.
        // The `current_angle` offset provides smooth animated transitions.
        let card_spacing = 0.42; // radians between cards

        // Sort by depth (back-to-front) for correct overlap.
        // Use continuous angle so sorting updates smoothly during animation.
        let mut draw_order: Vec<usize> = (0..num_entries).collect();
        draw_order.sort_by(|&a, &b| {
            let angle_a = current_angle + a as f32 * card_spacing;
            let angle_b = current_angle + b as f32 * card_spacing;
            let cos_a = angle_a.cos();
            let cos_b = angle_b.cos();
            // Back cards (lower cos) drawn first
            cos_a
                .partial_cmp(&cos_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Carousel area bounds
        let carousel_height = 200.0;
        let carousel_rect = Rect {
            x: rect.x,
            y: rect.y + 20.0,
            width: rect.width,
            height: carousel_height,
        };

        let center_x = carousel_rect.x + carousel_rect.width / 2.0;
        let center_y = carousel_rect.y + carousel_rect.height / 2.0;

        // ── 2.5D transform constants ──
        const YAW_FACTOR: f32 = 0.18; // rotation (yaw) per radian of arc position
        const SKEW_FACTOR: f32 = 0.12; // horizontal perspective skew per radian
        const DEPTH_FADE: f32 = 0.4; // opacity reduction at maximum depth
        const SHADOW_INTENSITY: f32 = 0.6; // shadow alpha at back cards

        for i in draw_order {
            // Continuous angle for this card (animated)
            let angle = current_angle + i as f32 * card_spacing;
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            let scale = 1.0 / (1.0 + 0.35 * (1.0 - cos_a));

            // Card rect (same math as before, but using continuous angle)
            let card_w = 190.0 * scale * cos_a.abs();
            let card_h = 110.0 * scale;
            let card_rect = Rect {
                x: center_x + 360.0 * sin_a * scale - card_w / 2.0,
                y: center_y + 12.0 * (1.0 - cos_a) * scale - card_h / 2.0,
                width: card_w,
                height: card_h,
            };

            // Z-index: cards further from center drawn behind
            let depth = (angle.rem_euclid(std::f32::consts::TAU / num_entries as f32)
                - std::f32::consts::PI)
                .abs();
            let z_index = depth * 10.0;
            renderer.set_z_index(z_index);

            let is_selected = i == selected;

            // ── 2.5D transform: push affine for perspective + rotation ──
            // Compute yaw rotation (tilt left/right based on arc position)
            let yaw = -sin_a * YAW_FACTOR;
            // Compute perspective skew (horizontal compression for side cards)
            let skew_x = sin_a * SKEW_FACTOR;
            // Build affine matrix: [a, b, c, d, e, f] = [m11, m12, m21, m22, tx, ty]
            // Rotation:  cos(yaw)  -sin(yaw)
            //            sin(yaw)   cos(yaw)
            // Skew:  applied as horizontal shear
            let cy = yaw.cos();
            let sy = yaw.sin();
            let affine = [
                cy * scale,                 // m11: scale + yaw rotation
                sy * scale,                 // m12
                -sy * scale + skew_x,       // m21: skew
                cy * scale,                 // m22
                card_rect.x + card_w / 2.0, // tx: rotate around card center
                card_rect.y + card_h / 2.0, // ty
            ];
            renderer.push_affine(affine);
            // Translate back so drawing happens at the correct position
            renderer.push_transform(
                [
                    card_rect.x - (card_rect.x + card_w / 2.0),
                    card_rect.y - (card_rect.y + card_h / 2.0),
                ],
                [1.0, 1.0],
                0.0,
            );

            // ── Depth-based opacity ──
            let depth_factor = (1.0 - cos_a).clamp(0.0, 1.0);
            let depth_alpha = 1.0 - depth_factor * DEPTH_FADE;

            let bg_color = if is_selected {
                [0.06, 0.055, 0.06, depth_alpha]
            } else {
                [0.02, 0.018, 0.02, depth_alpha * 0.9]
            };

            let border_color = if is_selected {
                [0.65, 0.58, 0.42, depth_alpha]
            } else {
                [0.14, 0.13, 0.12, depth_alpha * 0.7]
            };

            // Draw card body.
            // NOTE: do NOT wrap the carousel in push_vnode(card_rect)/pop_vnode().
            // Cards are drawn at ABSOLUTE card_rect coords; after the nodal
            // migration + Phase 3b, push_vnode bumps the renderer translation
            // stack by (rect.x, rect.y) and GPU primitives add it to every
            // vertex -> double-offset (card drawn at card_rect + card_rect).
            // The 2.5D placement is done purely via push_affine/push_transform
            // (GPU matrix), which does NOT touch the renderer translation
            // stack. The manual screen-space hit-test needs no VDOM node.
            renderer.fill_rounded_rect(card_rect, 6.0, bg_color);

            // Bevels (top highlight, left edge, bottom shadow)
            let bevel_h = if is_selected { 2.0 } else { 1.0 };
            let bevel_top = Rect {
                x: card_rect.x + 6.0,
                y: card_rect.y + 1.0,
                width: card_rect.width - 12.0,
                height: bevel_h,
            };
            let bevel_alpha = if is_selected { 0.65 } else { 0.22 };
            renderer.fill_rounded_rect(
                bevel_top,
                1.0,
                [0.80, 0.72, 0.55, bevel_alpha * depth_alpha],
            );

            let left_bevel = Rect {
                x: card_rect.x + 1.0,
                y: card_rect.y + 6.0,
                width: 1.2,
                height: card_rect.height - 12.0,
            };
            renderer.fill_rounded_rect(
                left_bevel,
                1.0,
                [0.60, 0.52, 0.38, bevel_alpha * 0.45 * depth_alpha],
            );

            // Bottom shadow: intensity varies with depth
            let shadow_alpha = 0.4 + depth_factor * SHADOW_INTENSITY;
            let shadow_bottom = Rect {
                x: card_rect.x + 6.0,
                y: card_rect.y + card_rect.height - 2.0,
                width: card_rect.width - 12.0,
                height: 2.0,
            };
            renderer.fill_rounded_rect(shadow_bottom, 1.0, [0.0, 0.0, 0.0, shadow_alpha]);

            // Border
            renderer.stroke_rounded_rect(
                card_rect,
                6.0,
                border_color,
                if is_selected { 1.5 } else { 0.8 },
            );

            // ── Text (depth-faded) ──
            let abs_diff = (angle / card_spacing).abs() % num_entries as f32;
            let text_alpha = if is_selected {
                depth_alpha
            } else if abs_diff <= 1.05 {
                0.55 * depth_alpha
            } else if abs_diff <= 2.05 {
                0.25 * depth_alpha
            } else {
                0.0
            };

            let text_color = if is_selected {
                [0.0, 1.0, 0.95, text_alpha]
            } else {
                [0.75, 0.70, 0.62, text_alpha]
            };

            let cat_font_size = 9.0 * scale;
            let (cat_w, _) = renderer.measure_text(entries[i].category, cat_font_size);
            renderer.draw_text_raw(
                entries[i].category,
                card_rect.x + (card_rect.width - cat_w) / 2.0,
                card_rect.y + 16.0 * scale,
                cat_font_size,
                text_color,
            );

            let name_font_size = 15.0 * scale * cos_a.max(0.6);
            let (name_w, _) = renderer.measure_text(entries[i].name, name_font_size);
            renderer.draw_text_raw(
                entries[i].name,
                card_rect.x + (card_rect.width - name_w) / 2.0,
                card_rect.y + 35.0 * scale,
                name_font_size,
                text_color,
            );

            // Pop the two transforms we pushed for this card
            renderer.pop_transform();
            renderer.pop_transform();
        }

        // Reset Z-index to default for the rest of the UI
        renderer.set_z_index(0.0);

        // Register a single click handler with manual hit-testing.
        // GpuRenderer uses a flat handler map (keyed by event type string, not by VNode).
        // Per-card handlers would ALL fire on every click, with the last one winning.
        // Instead, we register ONE handler that re-calculates each card's rect and tests
        // the click coordinates against it.
        //
        // Card rects are in local coordinates (relative to the parent's content origin).
        // Click events arrive in screen space, so we subtract the cumulative parent
        // translation captured at the top of render() to convert to local space.
        let click_state = self.state.clone();
        let click_cx = center_x;
        let click_cy = center_y;
        let click_num = num_entries;
        let click_tx = translation.x;
        let click_ty = translation.y;
        let card_spacing_click = card_spacing;
        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |evt| {
                if let Event::PointerClick { x, y, .. } = evt {
                    // Convert screen-space click to local coordinates
                    let local_x = x - click_tx;
                    let local_y = y - click_ty;
                    let s = click_state.lock().unwrap_or_else(|e| e.into_inner());
                    let current_angle = s.current_angle;
                    drop(s);
                    for i in 0..click_num {
                        // Use continuous angle for hit-testing (matches rendering)
                        let angle = current_angle + i as f32 * card_spacing_click;
                        let cos_a = angle.cos();
                        let sin_a = angle.sin();
                        let scale = 1.0 / (1.0 + 0.35 * (1.0 - cos_a));
                        let cw = 190.0 * scale * cos_a.abs();
                        let ch = 110.0 * scale;
                        let cx = click_cx + 360.0 * sin_a * scale - cw / 2.0;
                        let cy = click_cy + 12.0 * (1.0 - cos_a) * scale - ch / 2.0;

                        if local_x >= cx
                            && local_x <= cx + cw
                            && local_y >= cy
                            && local_y <= cy + ch
                        {
                            let mut s = click_state.lock().unwrap_or_else(|e| e.into_inner());
                            let old_selected = s.selected;
                            s.selected = i;
                            // Update target_angle for smooth animation
                            if i != old_selected {
                                // Compute shortest-path rotation
                                let old_angle =
                                    s.target_angle + old_selected as f32 * card_spacing_click;
                                let new_angle = s.target_angle + i as f32 * card_spacing_click;
                                let delta = new_angle - old_angle;
                                // Wrap to [-PI, PI] for shortest path
                                let wrapped = (delta + std::f32::consts::PI)
                                    .rem_euclid(std::f32::consts::TAU)
                                    - std::f32::consts::PI;
                                s.target_angle += wrapped;
                            }
                            break;
                        }
                    }
                }
            }),
        );

        // Register scroll-wheel handler for carousel cycling.
        // A single physical wheel notch (or momentum/inertial trackpad gesture)
        // produces several small `MouseWheel` events. Accumulate the delta and
        // advance the carousel exactly one step per notch so it progresses
        // smoothly instead of racing through every card on one gesture.
        let wheel_state = self.state.clone();
        let card_spacing_wheel = card_spacing;
        renderer.register_handler(
            "pointerwheel",
            std::sync::Arc::new(move |evt| {
                if let Event::PointerWheel { delta_y, .. } = evt {
                    let mut s = wheel_state.lock().unwrap_or_else(|e| e.into_inner());
                    let num = s.entries.len();
                    if num == 0 {
                        return;
                    }
                    // One notch ≈ 1.0 logical line; the dispatcher scales LineDelta
                    // by 10.0, so a full notch yields |delta_y| ≈ 10.0.
                    const NOTCH: f32 = 10.0;
                    s.wheel_accum += delta_y;
                    while s.wheel_accum >= NOTCH {
                        s.wheel_accum -= NOTCH;
                        let old_selected = s.selected;
                        s.selected = (s.selected + 1) % num;
                        // Update target_angle for smooth animation (one card forward)
                        s.target_angle += card_spacing_wheel;
                    }
                    while s.wheel_accum <= -NOTCH {
                        s.wheel_accum += NOTCH;
                        let old_selected = s.selected;
                        s.selected = (s.selected + num - 1) % num;
                        // Update target_angle for smooth animation (one card backward)
                        s.target_angle -= card_spacing_wheel;
                    }
                    // Avoid unbounded accumulation from tiny sub-notch deltas.
                    if s.wheel_accum.abs() > NOTCH * 4.0 {
                        s.wheel_accum = 0.0;
                    }
                }
            }),
        );

        // 3. Draw Divider Line
        let div_y = carousel_rect.y + carousel_rect.height + 15.0;
        renderer.draw_line(
            rect.x + 40.0,
            div_y,
            rect.x + rect.width - 40.0,
            div_y,
            [0.35, 0.12, 0.15, 0.6],
            1.0,
        );

        // 4. Draw Selected Component Title & View
        let title_y = div_y + 15.0;
        renderer.draw_text_raw(
            format!("GALLERY / {}", entries[selected].name.to_uppercase()).as_str(),
            rect.x + 40.0,
            title_y,
            12.0,
            [0.9, 0.2, 0.25, 0.85],
        );

        // Render target preview area
        let detail = (entries[selected].render)(&state, &self.state);
        let preview_rect = Rect {
            x: rect.x + 40.0,
            y: title_y + 25.0,
            width: rect.width - 80.0,
            height: rect.height - (title_y + 25.0) - 20.0,
        };

        let centered_detail = VStack::new(0.0)
            .distribution(cvkg::core::Distribution::Center)
            .alignment(cvkg::core::Alignment::Center)
            .child(detail)
            .flex(1.0)
            .frame(None, None);

        centered_detail.render(renderer, preview_rect);

        renderer.pop_vnode();
    }
}

impl GalleryApp {
    fn new() -> Self {
        Self {
            state: std::sync::Arc::new(std::sync::Mutex::new(GalleryState::new())),
        }
    }
}

fn main() {
    // Install panic hook that writes crash dump to disk
    std::panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };
        let bt = std::backtrace::Backtrace::force_capture();
        eprintln!("[CVKG PANIC] {msg}");
        eprintln!(
            "[CVKG PANIC] Backtrace:
{bt}"
        );
        if let Ok(mut file) = std::fs::File::create("cvkg-crash.log") {
            use std::io::Write;
            let _ = writeln!(file, "CVKG Panic Dump");
            let _ = writeln!(file, "Message: {msg}");
            let _ = writeln!(
                file,
                "Backtrace:
{bt}"
            );
        }
    }));

    cvkg::native::NativeRenderer::run(GalleryApp::new(), None);
}

#[cfg(test)]
mod smoke_tests {
    #[test]
    fn test_gallery_compiles() {
        assert!(true, "gallery smoke test");
    }
}
