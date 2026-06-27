use cvkg::components::{Badge, BadgeVariant, Toggle};
use cvkg::prelude::AnyView;
use cvkg::prelude::*;
use cvkg::core::{Renderer, View};

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
                        }).label("Enable Berserk Mode"))
                        .child(Checkbox::new(state.checkbox_2, move |val| {
                            let mut s = arc2.lock().unwrap();
                            s.checkbox_2 = val;
                        }).label("Auto-charge Rage")),
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
                    Text::new("Select (dropdown)")
                        .font_size(14.0)
                        .color([0.7, 0.7, 0.7, 1.0]),
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
            render: |_state, _state_arc| {
                AnyView::new(
                    Text::new("Tabs component")
                        .font_size(14.0)
                        .color([0.7, 0.7, 0.7, 1.0]),
                )
            },
        },
        // -- Overlays --
        GalleryEntry {
            name: "Tooltip",
            category: "Overlays",
            render: |_state, _state_arc| {
                AnyView::new(
                    Text::new("Tooltip component")
                        .font_size(14.0)
                        .color([0.7, 0.7, 0.7, 1.0]),
                )
            },
        },
        // -- Data Display --
        GalleryEntry {
            name: "Progress",
            category: "Data Display",
            render: |_state, _state_arc| {
                AnyView::new(
                    VStack::new(8.0)
                        .child(
                            Text::new("Progress: 70%")
                                .font_size(14.0)
                                .color([0.7, 0.7, 0.7, 1.0]),
                        )
                        .child(Progress::new(0.7)),
                )
            },
        },
        GalleryEntry {
            name: "Spinner",
            category: "Data Display",
            render: |_state, _state_arc| {
                AnyView::new(
                    HStack::new(8.0).child(Spinner::new()).child(
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

        for i in draw_order {
            let mut diff = (i as i32 - selected as i32) as f32;
            while diff > half { diff -= num_entries as f32; }
            while diff < -half { diff += num_entries as f32; }

            // Math for cylindrical projection
            let angle = diff * 0.42; // angle spacing
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            
            // Perspective depth factor
            let scale = 1.0 / (1.0 + 0.35 * (1.0 - cos_a));

            // Card size and 2.5D position
            let card_w = 190.0 * scale * cos_a; // foreshortened horizontally by rotation angle
            let card_h = 110.0 * scale;
            let center_x = carousel_rect.x + carousel_rect.width / 2.0;
            let center_y = carousel_rect.y + carousel_rect.height / 2.0;
            
            let card_x = center_x + 360.0 * sin_a * scale - card_w / 2.0;
            let card_y = center_y + 12.0 * (1.0 - cos_a) * scale - card_h / 2.0;

            let card_rect = Rect {
                x: card_x,
                y: card_y,
                width: card_w,
                height: card_h,
            };

            // Render Card
            let is_selected = i == selected;
            let border_color = if is_selected {
                [1.0, 0.1, 0.15, 1.0] // Neon Crimson Red for active
            } else {
                [0.45, 0.15, 0.18, 0.55] // Muted red-brown for sides
            };

            let bg_color = if is_selected {
                [0.08, 0.08, 0.12, 0.95] // Solid flat dark active surface
            } else {
                [0.04, 0.04, 0.06, 0.85] // Muted transparent side surface
            };

            renderer.push_vnode(card_rect, "CarouselCard");
            renderer.fill_rounded_rect(card_rect, 6.0, bg_color);
            renderer.stroke_rounded_rect(card_rect, 6.0, border_color, if is_selected { 2.0 } else { 1.0 });

            // Text scaling
            let text_color = if is_selected {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.65, 0.5, 0.52, 0.6]
            };

            // Category text (tiny)
            renderer.draw_text(
                entries[i].category,
                card_rect.x + 12.0 * scale,
                card_rect.y + 16.0 * scale,
                9.0 * scale,
                text_color,
            );

            // Component name text
            renderer.draw_text(
                entries[i].name,
                card_rect.x + 12.0 * scale,
                card_rect.y + 35.0 * scale,
                15.0 * scale * cos_a.max(0.6),
                text_color,
            );

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
            format!("BERZERKER PREVIEW // {}", entries[selected].name.to_uppercase()).as_str(),
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
