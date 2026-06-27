use cvkg::components::{Badge, BadgeVariant, BifrostTabs, ButtonVariant, SpinnerVariant, Toggle};
use cvkg::prelude::AnyView;
use cvkg::prelude::*;
use cvkg::core::{Event, Renderer, View};

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
                        .child(Checkbox::new(state.checkbox_1, move |val| {
                            let mut s = arc1.lock().unwrap();
                            s.checkbox_1 = val;
                        }).label("Enable Berserk Mode").frame(Some(220.0), Some(30.0)))
                        .child(Checkbox::new(state.checkbox_2, move |val| {
                            let mut s = arc2.lock().unwrap();
                            s.checkbox_2 = val;
                        }).label("Auto-charge Rage").frame(Some(220.0), Some(30.0))),
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
                            Input::new(state.input_text.as_str())
                                .on_change(move |text| {
                                    let mut s = arc.lock().unwrap();
                                    s.input_text = text;
                                })
                        )
                        .child(
                            Text::new(format!("Typed: {}", state.input_text))
                                .font_size(14.0)
                                .color([0.7, 0.7, 0.7, 1.0]),
                        )
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
                            let mut s = arc1.lock().unwrap();
                            s.toggle_1 = val;
                        }))
                        .child(Toggle::new("Odin's Sight", state.toggle_2, move |val| {
                            let mut s = arc2.lock().unwrap();
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
                            let mut s = arc.lock().unwrap();
                            s.slider_value = val;
                        })),
                )
            },
        },
        GalleryEntry {
            name: "Select",
            category: "Forms",
            render: |_state, _state_arc| {
                AnyView::new(
                    VStack::new(4.0)
                        .child(Text::new("Select: Berserker").font_size(14.0).color([1.0, 1.0, 1.0, 1.0]))
                        .child(Text::new("▼ Shieldmaiden · Runecaster · Valkyrie").font_size(11.0).color([0.8, 0.7, 0.9, 1.0])),
                )
            },
        },
        // -- Layout --
        GalleryEntry {
            name: "VStack",
            category: "Layout",
            render: |_state, _state_arc| {
                AnyView::new(
                    VStack::new(8.0)
                        .child(
                            Text::new("Item 1")
                                .font_size(14.0)
                                .color([1.0, 1.0, 1.0, 1.0]),
                        )
                        .child(
                            Text::new("Item 2")
                                .font_size(14.0)
                                .color([0.8, 0.8, 0.8, 1.0]),
                        )
                        .child(
                            Text::new("Item 3")
                                .font_size(14.0)
                                .color([0.6, 0.6, 0.6, 1.0]),
                        ),
                )
            },
        },
        GalleryEntry {
            name: "HStack",
            category: "Layout",
            render: |_state, _state_arc| {
                AnyView::new(
                    HStack::new(8.0)
                        .child(
                            Text::new("Left")
                                .font_size(14.0)
                                .color([1.0, 1.0, 1.0, 1.0]),
                        )
                        .child(
                            Text::new("Center")
                                .font_size(14.0)
                                .color([0.8, 0.8, 0.8, 1.0]),
                        )
                        .child(
                            Text::new("Right")
                                .font_size(14.0)
                                .color([0.6, 0.6, 0.6, 1.0]),
                        ),
                )
            },
        },
        GalleryEntry {
            name: "Text",
            category: "Layout",
            render: |_state, _state_arc| {
                AnyView::new(
                    VStack::new(4.0)
                        .child(
                            Text::new("Heading")
                                .font_size(24.0)
                                .color([1.0, 1.0, 1.0, 1.0]),
                        )
                        .child(
                            Text::new("Body text")
                                .font_size(14.0)
                                .color([0.7, 0.7, 0.7, 1.0]),
                        )
                        .child(
                            Text::new("Caption")
                                .font_size(10.0)
                                .color([0.5, 0.5, 0.5, 1.0]),
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
                AnyView::new(
                    BifrostTabs::new(
                        vec!["Shield".to_string(), "Rage".to_string(), "Runes".to_string()],
                        state.tab_index,
                        move |idx| {
                            let mut s = arc_clone.lock().unwrap();
                            s.tab_index = idx;
                        },
                    ),
                )
            },
        },
        // -- Overlays --
        GalleryEntry {
            name: "Tooltip",
            category: "Overlays",
            render: |_state, _state_arc| {
                AnyView::new(
                    Tooltip::new(
                        AnyView::new(Text::new("Hover target").font_size(14.0).color([0.9, 0.9, 0.9, 1.0])),
                        "Hidden wisdom: Runes guide the worthy",
                    ).visible(true),
                )
            },
        },
        // -- Data Display --
        GalleryEntry {
            name: "Progress",
            category: "Data Display",
            render: |state, _state_arc| {
                // Animate progress 0→1 cycling every 3 seconds using wall-clock time.
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
            render: |state, _state_arc| {
                // Compute rotation from wall-clock time; the Spinner render
                // also reads elapsed_time() internally for its arc offset.
                let _t = state.start_time.elapsed().as_secs_f32();
                // Use Ring variant at larger size so the spin animation is clearly visible
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
            name: "Badge",
            category: "Data Display",
            render: |_state, _state_arc| {
                AnyView::new(
                    HStack::new(8.0)
                        .child(Badge::new("Default"))
                        .child(Badge::new("Info").variant(BadgeVariant::Secondary))
                        .child(Badge::new("Outline").variant(BadgeVariant::Outline)),
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
        GalleryEntry {
            name: "Dialog",
            category: "Overlays",
            render: |_state, _state_arc| {
                AnyView::new(
                    VStack::new(12.0)
                        .child(
                            Text::new("Confirm Rite of Passage?")
                                .font_size(16.0)
                                .color([1.0, 1.0, 1.0, 1.0]),
                        )
                        .child(
                            Text::new("This action cannot be undone.")
                                .font_size(12.0)
                                .color([0.7, 0.7, 0.7, 1.0]),
                        )
                        .child(
                            HStack::new(12.0)
                                .child(Button::new("Accept", || {}).variant(ButtonVariant::TintedGlass))
                                .child(Button::new("Decline", || {}).variant(ButtonVariant::Ghost)),
                        ),
                )
            },
        },
        GalleryEntry {
            name: "Avatar",
            category: "Data Display",
            render: |_state, _state_arc| {
                AnyView::new(
                    HStack::new(16.0)
                        .child(
                            VStack::new(4.0)
                                .child(
                                    Text::new("[A]")
                                        .font_size(24.0)
                                        .color([0.0, 1.0, 1.0, 1.0]),
                                )
                                .child(
                                    Text::new("Astrid")
                                        .font_size(10.0)
                                        .color([0.7, 0.7, 0.7, 1.0]),
                                ),
                        )
                        .child(
                            VStack::new(4.0)
                                .child(
                                    Text::new("[B]")
                                        .font_size(24.0)
                                        .color([1.0, 0.5, 0.0, 1.0]),
                                )
                                .child(
                                    Text::new("Bjorn")
                                        .font_size(10.0)
                                        .color([0.7, 0.7, 0.7, 1.0]),
                                ),
                        )
                        .child(
                            VStack::new(4.0)
                                .child(
                                    Text::new("[F]")
                                        .font_size(24.0)
                                        .color([0.5, 0.3, 1.0, 1.0]),
                                )
                                .child(
                                    Text::new("Freya")
                                        .font_size(10.0)
                                        .color([0.7, 0.7, 0.7, 1.0]),
                                ),
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
    /// Wall-clock start time so render closures can compute elapsed seconds.
    start_time: std::time::Instant,
    tab_index: usize,
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
        let state = self.state.lock().unwrap();
        let selected = state.selected;
        let entries = &state.entries;
        let num_entries = entries.len();
        let half = num_entries as f32 / 2.0;

        // 1. Draw Background Area
        renderer.push_vnode(rect, "GalleryApp");

        // 2. Draw 3D Carousel (Top Panel)
        // Depth-sort indices to draw background cards first
        let mut draw_order: Vec<usize> = (0..num_entries).collect();
        draw_order.sort_by(|&a, &b| {
            let mut diff_a = (a as i32 - selected as i32) as f32;
            while diff_a > half { diff_a -= num_entries as f32; }
            while diff_a < -half { diff_a += num_entries as f32; }

            let mut diff_b = (b as i32 - selected as i32) as f32;
            while diff_b > half { diff_b -= num_entries as f32; }
            while diff_b < -half { diff_b += num_entries as f32; }

            let cos_a = (diff_a * 0.45).cos();
            let cos_b = (diff_b * 0.45).cos();
            cos_a.partial_cmp(&cos_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Carousel area bounds
        let carousel_height = 200.0;
        let carousel_rect = Rect {
            x: rect.x,
            y: rect.y + 20.0,
            width: rect.width,
            height: carousel_height,
        };

        // Pre-calculate active card's rect for occlusion clipping
        let active_scale = 1.0; // cos_a = 1.0 for selected
        let active_w = 190.0 * active_scale;
        let active_h = 110.0 * active_scale;
        let center_x = carousel_rect.x + carousel_rect.width / 2.0;
        let center_y = carousel_rect.y + carousel_rect.height / 2.0;
        let _active_rect = Rect {
            x: center_x - active_w / 2.0,
            y: center_y - active_h / 2.0,
            width: active_w,
            height: active_h,
        };

        // We use Z-index layering to ensure cards drawn later (closer to camera)
        // correctly occlude the text of cards drawn earlier.
        for i in draw_order {
            let mut diff = (i as i32 - selected as i32) as f32;
            while diff > half { diff -= num_entries as f32; }
            while diff < -half { diff += num_entries as f32; }

            let calculate_card_rect = |d: f32| -> Rect {
                let angle = d * 0.42;
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                let scale = 1.0 / (1.0 + 0.35 * (1.0 - cos_a));
                let card_w = 190.0 * scale * cos_a;
                let card_h = 110.0 * scale;
                let center_x = carousel_rect.x + carousel_rect.width / 2.0;
                let center_y = carousel_rect.y + carousel_rect.height / 2.0;
                Rect {
                    x: center_x + 360.0 * sin_a * scale - card_w / 2.0,
                    y: center_y + 12.0 * (1.0 - cos_a) * scale - card_h / 2.0,
                    width: card_w,
                    height: card_h,
                }
            };

            // Math for cylindrical projection
            let angle = diff * 0.42; // angle spacing
            let cos_a = angle.cos();
            let scale = 1.0 / (1.0 + 0.35 * (1.0 - cos_a));
            let card_rect = calculate_card_rect(diff);

            // Assign Z-index based on depth (closest card = Z 0.0, furthest = higher Z)
            let z_index = diff.abs() * 10.0;
            renderer.set_z_index(z_index);

            // Render Card — reflective ceramic black / dull dark metal
            let is_selected = i == selected;

            // Base: near-black ceramic. Active card is just barely warmer.
            let bg_color = if is_selected {
                [0.06, 0.055, 0.06, 1.0] // very dark ceramic — active
            } else {
                [0.02, 0.018, 0.02, 1.0] // near pure black — inactive
            };

            // Border: active gets a warm amber rim; inactive nearly disappears into black
            let border_color = if is_selected {
                [0.65, 0.58, 0.42, 1.0] // warm forged-steel rim on active
            } else {
                [0.14, 0.13, 0.12, 1.0] // near-black, barely visible
            };

            renderer.push_vnode(card_rect, "CarouselCard");

            // 1. Solid near-black ceramic base
            renderer.fill_rounded_rect(card_rect, 6.0, bg_color);

            // 2. Top-edge bevel: the only visible "reflection" on ceramic black
            let bevel_h = if is_selected { 2.0 } else { 1.0 };
            let bevel_top = Rect {
                x: card_rect.x + 6.0,
                y: card_rect.y + 1.0,
                width: card_rect.width - 12.0,
                height: bevel_h,
            };
            let bevel_alpha = if is_selected { 0.65 } else { 0.22 };
            renderer.fill_rounded_rect(bevel_top, 1.0, [0.80, 0.72, 0.55, bevel_alpha]);

            // 3. Left-edge secondary catch-light
            let left_bevel = Rect {
                x: card_rect.x + 1.0,
                y: card_rect.y + 6.0,
                width: 1.2,
                height: card_rect.height - 12.0,
            };
            renderer.fill_rounded_rect(left_bevel, 1.0, [0.60, 0.52, 0.38, bevel_alpha * 0.45]);

            // 4. Bottom anvil shadow
            let shadow_bottom = Rect {
                x: card_rect.x + 6.0,
                y: card_rect.y + card_rect.height - 2.0,
                width: card_rect.width - 12.0,
                height: 2.0,
            };
            renderer.fill_rounded_rect(shadow_bottom, 1.0, [0.0, 0.0, 0.0, 0.95]);

            // 5. Border rim
            renderer.stroke_rounded_rect(card_rect, 6.0, border_color, if is_selected { 1.5 } else { 0.8 });

            // Text: now safely drawn inside the loop.
            // Since CVKG batches text at the end of the frame, Z-index is ignored for text depth.
            // To prevent side cards' text from bleeding through the active card, we must
            // explicitly clip it to the visible region outside the active card.
            let abs_diff = diff.abs();
            let text_alpha = if is_selected {
                1.0
            } else if abs_diff <= 1.05 {
                0.55
            } else if abs_diff <= 2.05 {
                0.25
            } else {
                0.0 // too far back
            };

            let text_color = if is_selected {
                [0.0, 1.0, 0.95, text_alpha] // neon cyan
            } else {
                [0.75, 0.70, 0.62, text_alpha] // warm off-white, faded by depth
            };

            if !is_selected {
                let covering_diff = if diff < 0.0 { diff + 1.0 } else { diff - 1.0 };
                let covering_rect = calculate_card_rect(covering_diff);

                let clip_rect = if diff < 0.0 {
                    // Card is on the left, clip right side where it overlaps its right neighbor
                    Rect {
                        x: card_rect.x,
                        y: card_rect.y,
                        width: (covering_rect.x - card_rect.x).max(0.0),
                        height: card_rect.height,
                    }
                } else {
                    // Card is on the right, clip left side where it overlaps its left neighbor
                    let start_x = covering_rect.x + covering_rect.width;
                    Rect {
                        x: start_x,
                        y: card_rect.y,
                        width: ((card_rect.x + card_rect.width) - start_x).max(0.0),
                        height: card_rect.height,
                    }
                };
                renderer.push_clip_rect(clip_rect);
            }

            let cat_font_size = 9.0 * scale;
            let (cat_w, _) = renderer.measure_text(entries[i].category, cat_font_size);
            renderer.draw_text(
                entries[i].category,
                card_rect.x + (card_rect.width - cat_w) / 2.0,
                card_rect.y + 16.0 * scale,
                cat_font_size,
                text_color,
            );

            let name_font_size = 15.0 * scale * cos_a.max(0.6);
            let (name_w, _) = renderer.measure_text(entries[i].name, name_font_size);
            renderer.draw_text(
                entries[i].name,
                card_rect.x + (card_rect.width - name_w) / 2.0,
                card_rect.y + 35.0 * scale,
                name_font_size,
                text_color,
            );

            if !is_selected {
                renderer.pop_clip_rect();
            }

            // Register card select click handler
            let state_arc_clone = self.state.clone();
            renderer.register_handler(
                "pointerclick",
                std::sync::Arc::new(move |_| {
                    let mut s = state_arc_clone.lock().unwrap();
                    s.selected = i;
                }),
            );

            renderer.pop_vnode();
        }
        
        // Reset Z-index to default for the rest of the UI
        renderer.set_z_index(0.0);

        // Register scroll-wheel handler for carousel cycling
        let wheel_state = self.state.clone();
        renderer.register_handler(
            "pointerwheel",
            std::sync::Arc::new(move |evt| {
                if let Event::PointerWheel { delta_y, .. } = evt {
                    let mut s = wheel_state.lock().unwrap();
                    let num = s.entries.len();
                    if num > 0 {
                        if delta_y > 0.5 {
                            s.selected = (s.selected + 1) % num;
                        } else if delta_y < -0.5 {
                            s.selected = (s.selected + num - 1) % num;
                        }
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
        renderer.draw_text(
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
    cvkg::native::NativeRenderer::run(GalleryApp::new(), None);
}
