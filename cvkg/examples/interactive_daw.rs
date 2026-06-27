// This example requires a renderer feature to be enabled (gpu, native, or web)
#![cfg(any(feature = "gpu", feature = "native", feature = "web"))]

use cvkg::prelude::*;
use cvkg_core::{Event, Renderer};
use std::sync::{Arc, Mutex};
use std::time::Instant;

struct DawState {
    playhead_pos: f32,
    start_time: Instant,
    playing: bool,
    master_volume: f32,
    tracks: Vec<TrackState>,
}

struct TrackState {
    name: String,
    muted: bool,
    soloed: bool,
    armed: bool,
    volume: f32,
    pan: f32,
    waveform_seed: f32,
    color: [f32; 4],
}

struct DawView {
    state: Arc<Mutex<DawState>>,
}

impl View for DawView {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let mut state = self.state.lock().unwrap();

        // Time logic
        let mut elapsed = 0.0;
        if state.playing {
            elapsed = state.start_time.elapsed().as_secs_f32();
            // Let it wrap around
            state.playhead_pos = 200.0 + (elapsed * 80.0) % (rect.width.max(300.0) - 200.0);
            renderer.request_redraw();
        }

        // Layout Constants dynamically scaled
        let top_bar_height = 50.0;
        let left_panel_width = 220.0;
        // Make the mixer section 35% of the screen height, capped between 150 and 300
        let bottom_mixer_height = (rect.height * 0.35).clamp(150.0, 300.0);
        let arranger_height = rect.height - top_bar_height - bottom_mixer_height;
        let track_height = (arranger_height / state.tracks.len().max(1) as f32)
            .min(100.0)
            .max(60.0);

        let bg_color = [0.08, 0.09, 0.10, 1.0];
        let panel_bg = [0.12, 0.13, 0.14, 1.0];
        let inspector_bg = [0.14, 0.15, 0.17, 1.0];
        let border_color = [0.03, 0.03, 0.04, 1.0];
        let text_light = [0.95, 0.95, 0.95, 1.0];
        let text_dim = [0.5, 0.5, 0.55, 1.0];
        let text_dark = [0.1, 0.1, 0.1, 1.0];

        // 1. Base App Background
        renderer.fill_rect(rect, bg_color);

        // 2. Main Timeline Area (Arranger View)
        let arranger_rect = Rect {
            x: left_panel_width,
            y: top_bar_height,
            width: rect.width - left_panel_width,
            height: arranger_height,
        };

        // Timeline Ruler
        renderer.fill_rect(
            Rect {
                x: arranger_rect.x,
                y: arranger_rect.y,
                width: arranger_rect.width,
                height: 30.0,
            },
            [0.15, 0.16, 0.18, 1.0],
        );
        renderer.stroke_rect(
            Rect {
                x: arranger_rect.x,
                y: arranger_rect.y,
                width: arranger_rect.width,
                height: 30.0,
            },
            border_color,
            1.0,
        );

        // Tick marks on ruler
        for i in 0..30 {
            let tick_x = arranger_rect.x + (i as f32 * 80.0);
            if tick_x < rect.width {
                renderer.fill_rect(
                    Rect {
                        x: tick_x,
                        y: arranger_rect.y + 15.0,
                        width: 1.0,
                        height: 15.0,
                    },
                    text_dim,
                );
                renderer.draw_text(
                    &format!("{}", i + 1),
                    tick_x + 5.0,
                    arranger_rect.y + 18.0,
                    11.0,
                    text_dim,
                );
            }
        }

        // Track Lanes & Waveforms
        for (i, track) in state.tracks.iter().enumerate() {
            let lane_y = arranger_rect.y + 30.0 + (i as f32 * track_height);
            let clip_x = arranger_rect.x + 10.0;
            let clip_y = lane_y + 10.0;
            let clip_width = arranger_rect.width - 20.0;
            let clip_height = track_height - 20.0;

            // Alternating lane background
            let lane_bg = if i % 2 == 0 {
                [0.11, 0.12, 0.13, 1.0]
            } else {
                [0.10, 0.11, 0.12, 1.0]
            };
            renderer.fill_rect(
                Rect {
                    x: arranger_rect.x,
                    y: lane_y,
                    width: arranger_rect.width,
                    height: track_height,
                },
                lane_bg,
            );
            // Lane bottom border
            renderer.fill_rect(
                Rect {
                    x: arranger_rect.x,
                    y: lane_y + track_height - 1.0,
                    width: arranger_rect.width,
                    height: 1.0,
                },
                [0.05, 0.05, 0.05, 1.0],
            );

            // Clip Background with high fidelity border
            renderer.fill_rect(
                Rect {
                    x: clip_x,
                    y: clip_y,
                    width: clip_width,
                    height: clip_height,
                },
                [
                    track.color[0] * 0.25,
                    track.color[1] * 0.25,
                    track.color[2] * 0.25,
                    1.0,
                ],
            );
            renderer.stroke_rect(
                Rect {
                    x: clip_x,
                    y: clip_y,
                    width: clip_width,
                    height: clip_height,
                },
                [
                    track.color[0] * 0.5,
                    track.color[1] * 0.5,
                    track.color[2] * 0.5,
                    1.0,
                ],
                1.0,
            );

            // Clip zero-crossing line
            renderer.fill_rect(
                Rect {
                    x: clip_x,
                    y: clip_y + (clip_height / 2.0),
                    width: clip_width,
                    height: 1.0,
                },
                [
                    track.color[0] * 0.4,
                    track.color[1] * 0.4,
                    track.color[2] * 0.4,
                    0.8,
                ],
            );

            // Optimized High Fidelity Procedural Waveform
            let num_bars = (clip_width / 4.0) as usize;
            for b in 0..num_bars {
                let bx = clip_x + (b as f32 * 4.0);
                let freq1 = 0.15 * track.waveform_seed;
                let freq2 = 0.03 * track.waveform_seed;
                let mut amp = f32::sin(bx * freq1) * 0.5 + f32::cos(bx * freq2) * 0.5;
                amp = (amp.abs() * 0.9 + 0.05) * (1.0 - (b as f32 / num_bars as f32).powi(2) * 0.5); // Fade out slightly at end

                let bar_h = (clip_height - 4.0) * amp;
                let bar_y = clip_y + (clip_height - bar_h) * 0.5;

                renderer.fill_rect(
                    Rect {
                        x: bx,
                        y: bar_y,
                        width: 2.0,
                        height: bar_h,
                    },
                    track.color,
                );
            }
        }

        // 3. Left Panel (Track Headers / Inspector)
        renderer.fill_rect(
            Rect {
                x: 0.0,
                y: top_bar_height,
                width: left_panel_width,
                height: arranger_height,
            },
            inspector_bg,
        );
        // Deep inset border on the right side of inspector
        renderer.fill_rect(
            Rect {
                x: left_panel_width - 2.0,
                y: top_bar_height,
                width: 2.0,
                height: arranger_height,
            },
            border_color,
        );

        // Header Top empty area (aligns with ruler)
        renderer.fill_rect(
            Rect {
                x: 0.0,
                y: arranger_rect.y,
                width: left_panel_width - 2.0,
                height: 30.0,
            },
            [0.11, 0.12, 0.14, 1.0],
        );
        renderer.stroke_rect(
            Rect {
                x: 0.0,
                y: arranger_rect.y,
                width: left_panel_width - 2.0,
                height: 30.0,
            },
            border_color,
            1.0,
        );
        renderer.draw_text("Tracks", 15.0, arranger_rect.y + 18.0, 12.0, text_dim);

        // Drop lock before iterative drawing, we will just copy state to avoid deadlocks on event handlers
        // Wait, we need to draw M S R and volume slider which require interactive VNodes.
        for i in 0..state.tracks.len() {
            let header_y = arranger_rect.y + 30.0 + (i as f32 * track_height);
            let track = &state.tracks[i];

            // Header Bg
            renderer.fill_rect(
                Rect {
                    x: 0.0,
                    y: header_y,
                    width: left_panel_width - 2.0,
                    height: track_height,
                },
                [0.15, 0.16, 0.18, 1.0],
            );
            renderer.stroke_rect(
                Rect {
                    x: 0.0,
                    y: header_y,
                    width: left_panel_width - 2.0,
                    height: track_height,
                },
                [0.1, 0.1, 0.11, 1.0],
                1.0,
            );

            // Left color strip indicating track color
            renderer.fill_rect(
                Rect {
                    x: 0.0,
                    y: header_y,
                    width: 6.0,
                    height: track_height,
                },
                track.color,
            );

            // Track Name
            renderer.draw_text(&track.name, 15.0, header_y + 15.0, 13.0, text_light);

            // High Fidelity M S R Buttons - INTERACTIVE
            let draw_btn = |renderer: &mut dyn Renderer,
                            bx: f32,
                            by: f32,
                            text: &str,
                            active: bool,
                            active_color: [f32; 4]| {
                let btn_rect = Rect {
                    x: bx,
                    y: by,
                    width: 22.0,
                    height: 20.0,
                };
                renderer.push_vnode(btn_rect, "MSRButton");

                let state_clone = self.state.clone();
                let text_owned = text.to_string();
                renderer.register_handler(
                    "pointerdown",
                    std::sync::Arc::new(move |_| {
                        let mut s = state_clone.lock().unwrap();
                        match text_owned.as_str() {
                            "M" => s.tracks[i].muted = !s.tracks[i].muted,
                            "S" => s.tracks[i].soloed = !s.tracks[i].soloed,
                            "R" => s.tracks[i].armed = !s.tracks[i].armed,
                            _ => {}
                        }
                    }),
                );

                let bg = if active {
                    active_color
                } else {
                    [0.08, 0.08, 0.09, 1.0]
                };
                let fg = if active {
                    text_dark
                } else {
                    [0.7, 0.7, 0.7, 1.0]
                };
                renderer.fill_rect(btn_rect, bg);
                renderer.stroke_rect(btn_rect, [0.05, 0.05, 0.05, 1.0], 1.0);

                // Top inner highlight for 3D bevel
                renderer.fill_rect(
                    Rect {
                        x: bx + 1.0,
                        y: by + 1.0,
                        width: 20.0,
                        height: 1.0,
                    },
                    [1.0, 1.0, 1.0, 0.1],
                );
                renderer.draw_text_centered(text, btn_rect.x, btn_rect.y, 11.0, fg);
                renderer.pop_vnode();
            };

            draw_btn(
                renderer,
                15.0,
                header_y + 35.0,
                "M",
                track.muted,
                [0.95, 0.75, 0.2, 1.0],
            );
            draw_btn(
                renderer,
                42.0,
                header_y + 35.0,
                "S",
                track.soloed,
                [0.2, 0.85, 0.3, 1.0],
            );
            draw_btn(
                renderer,
                69.0,
                header_y + 35.0,
                "R",
                track.armed,
                [0.95, 0.25, 0.25, 1.0],
            );

            // High Fidelity Track Volume Slider inside Header - INTERACTIVE
            let slider_x = 105.0;
            let slider_y = header_y + 40.0;
            let slider_w = 90.0;

            renderer.draw_text("Vol", slider_x, slider_y - 8.0, 9.0, text_dim);

            // Slider interactive area
            let slider_rect = Rect {
                x: slider_x,
                y: slider_y - 4.0,
                width: slider_w,
                height: 14.0,
            };
            renderer.push_vnode(slider_rect, "HeaderVolSlider");
            let state_clone = self.state.clone();
            let on_move = std::sync::Arc::new(move |event| {
                if let Event::PointerDown { x, .. } | Event::PointerMove { x, .. } = event {
                    let relative = ((x - slider_x) / slider_w).clamp(0.0, 1.0);
                    state_clone.lock().unwrap().tracks[i].volume = relative;
                }
            });
            renderer.register_handler("pointerdown", on_move.clone());
            renderer.register_handler("pointermove", on_move);

            // Slider inset track
            renderer.fill_rect(
                Rect {
                    x: slider_x,
                    y: slider_y,
                    width: slider_w,
                    height: 6.0,
                },
                [0.05, 0.05, 0.06, 1.0],
            );
            renderer.stroke_rect(
                Rect {
                    x: slider_x,
                    y: slider_y,
                    width: slider_w,
                    height: 6.0,
                },
                [0.2, 0.2, 0.22, 1.0],
                1.0,
            );

            // Slider fill
            let fill_w = slider_w * track.volume;
            renderer.fill_rect(
                Rect {
                    x: slider_x + 1.0,
                    y: slider_y + 1.0,
                    width: fill_w - 2.0,
                    height: 4.0,
                },
                [0.3, 0.6, 0.9, 1.0],
            );

            // Slider thumb handle
            let thumb_x = slider_x + fill_w - 6.0;
            renderer.fill_rect(
                Rect {
                    x: thumb_x,
                    y: slider_y - 4.0,
                    width: 12.0,
                    height: 14.0,
                },
                [0.7, 0.7, 0.7, 1.0],
            );
            renderer.stroke_rect(
                Rect {
                    x: thumb_x,
                    y: slider_y - 4.0,
                    width: 12.0,
                    height: 14.0,
                },
                [0.1, 0.1, 0.1, 1.0],
                1.0,
            );
            renderer.fill_rect(
                Rect {
                    x: thumb_x + 5.0,
                    y: slider_y - 2.0,
                    width: 2.0,
                    height: 10.0,
                },
                [0.3, 0.3, 0.3, 1.0],
            );
            renderer.pop_vnode();
        }

        // 4. Playhead (Over arranger only)
        let playhead_x = state.playhead_pos.max(left_panel_width);
        if playhead_x < rect.width {
            // Playhead Line
            renderer.fill_rect(
                Rect {
                    x: playhead_x,
                    y: arranger_rect.y,
                    width: 1.5,
                    height: arranger_rect.height,
                },
                [1.0, 0.2, 0.2, 1.0],
            );
            // Playhead Triangle Cap
            renderer.fill_rect(
                Rect {
                    x: playhead_x - 5.0,
                    y: arranger_rect.y,
                    width: 11.5,
                    height: 15.0,
                },
                [0.9, 0.1, 0.1, 1.0],
            );
            renderer.stroke_rect(
                Rect {
                    x: playhead_x - 5.0,
                    y: arranger_rect.y,
                    width: 11.5,
                    height: 15.0,
                },
                [0.5, 0.0, 0.0, 1.0],
                1.0,
            );
        }

        // 5. Bottom Mixer Panel
        let mixer_y = rect.height - bottom_mixer_height;
        renderer.fill_rect(
            Rect {
                x: 0.0,
                y: mixer_y,
                width: rect.width,
                height: bottom_mixer_height,
            },
            panel_bg,
        );
        // Mixer top border
        renderer.fill_rect(
            Rect {
                x: 0.0,
                y: mixer_y,
                width: rect.width,
                height: 2.0,
            },
            [0.25, 0.26, 0.28, 1.0],
        );
        renderer.fill_rect(
            Rect {
                x: 0.0,
                y: mixer_y - 1.0,
                width: rect.width,
                height: 1.0,
            },
            border_color,
        );

        // Mixer Channel Strips
        let strip_w = 70.0;
        let start_x = left_panel_width + 10.0;
        let master_x = rect.width - 100.0;

        for i in 0..state.tracks.len() {
            let track = &state.tracks[i];
            let strip_x = start_x + (i as f32 * (strip_w + 4.0));
            if strip_x + strip_w > master_x - 20.0 {
                break;
            } // Don't overflow into master

            // Strip Bg
            renderer.fill_rect(
                Rect {
                    x: strip_x,
                    y: mixer_y + 10.0,
                    width: strip_w,
                    height: bottom_mixer_height - 20.0,
                },
                [0.10, 0.11, 0.12, 1.0],
            );
            renderer.stroke_rect(
                Rect {
                    x: strip_x,
                    y: mixer_y + 10.0,
                    width: strip_w,
                    height: bottom_mixer_height - 20.0,
                },
                [0.05, 0.05, 0.06, 1.0],
                1.0,
            );

            // Track Color Label at Bottom
            let label_rect = Rect {
                x: strip_x + 1.0,
                y: mixer_y + bottom_mixer_height - 30.0,
                width: strip_w - 2.0,
                height: 20.0,
            };
            renderer.fill_rect(
                label_rect,
                [
                    track.color[0] * 0.4,
                    track.color[1] * 0.4,
                    track.color[2] * 0.4,
                    1.0,
                ],
            );
            renderer.draw_text_centered(
                &format!("CH {}", i + 1),
                label_rect.x,
                label_rect.y,
                11.0,
                text_light,
            );

            // Pan Knob - INTERACTIVE
            let pan_y = mixer_y + 20.0;
            let pan_label_rect = Rect {
                x: strip_x + 15.0,
                y: pan_y - 10.0,
                width: 40.0,
                height: 20.0,
            };
            renderer.draw_text_centered("PAN", pan_label_rect.x, pan_label_rect.y, 9.0, text_dim);

            let knob_rect = Rect {
                x: strip_x + 25.0,
                y: pan_y + 8.0,
                width: 20.0,
                height: 20.0,
            };
            renderer.push_vnode(knob_rect, "PanKnob");
            let state_clone = self.state.clone();
            let on_move = std::sync::Arc::new(move |event| {
                if let Event::PointerDown { x, .. } | Event::PointerMove { x, .. } = event {
                    let relative = ((x - (strip_x + 15.0)) / 40.0).clamp(0.0, 1.0);
                    state_clone.lock().unwrap().tracks[i].pan = relative;
                }
            });
            renderer.register_handler("pointerdown", on_move.clone());
            renderer.register_handler("pointermove", on_move);

            // Knob body
            renderer.fill_rect(knob_rect, [0.2, 0.2, 0.22, 1.0]);
            renderer.stroke_rect(knob_rect, [0.05, 0.05, 0.05, 1.0], 1.0);

            // Indicator line mapped from pan
            let angle = (track.pan - 0.5) * std::f32::consts::PI * 1.5; // -135 to 135 degrees
            let cx = strip_x + 35.0;
            let cy = pan_y + 18.0;
            let ind_x = cx + f32::sin(angle) * 8.0;
            let ind_y = cy - f32::cos(angle) * 8.0;

            // Poor man's draw line by filling a small rect
            renderer.fill_rect(
                Rect {
                    x: ind_x - 1.0,
                    y: ind_y - 1.0,
                    width: 2.0,
                    height: 2.0,
                },
                [0.8, 0.8, 0.8, 1.0],
            );
            renderer.pop_vnode();

            // High Fidelity Segmented LED Meter
            let mut meter_level = 0.0;
            if state.playing && !track.muted {
                let freq = 0.1 * track.waveform_seed;
                meter_level = (f32::sin(state.playhead_pos * freq).abs() * track.volume).min(1.0);
            }

            let meter_h = bottom_mixer_height - 110.0;
            let meter_y = pan_y + 40.0;

            // LED background
            renderer.fill_rect(
                Rect {
                    x: strip_x + 40.0,
                    y: meter_y,
                    width: 12.0,
                    height: meter_h,
                },
                [0.03, 0.03, 0.04, 1.0],
            );
            renderer.stroke_rect(
                Rect {
                    x: strip_x + 40.0,
                    y: meter_y,
                    width: 12.0,
                    height: meter_h,
                },
                [0.2, 0.2, 0.2, 1.0],
                1.0,
            );

            // Draw LED Segments
            let num_leds = 20;
            let led_height = (meter_h / num_leds as f32) - 1.0;
            let active_leds = (num_leds as f32 * meter_level) as usize;

            for led in 0..num_leds {
                let led_y = meter_y + meter_h - ((led + 1) as f32 * (led_height + 1.0));

                let led_color = if led < active_leds {
                    if led > 16 {
                        [0.9, 0.2, 0.2, 1.0] // Red Peak
                    } else if led > 12 {
                        [0.9, 0.8, 0.2, 1.0] // Yellow Mid
                    } else {
                        [0.2, 0.9, 0.4, 1.0] // Green Low
                    }
                } else {
                    [0.08, 0.08, 0.08, 1.0] // Inactive LED
                };

                renderer.fill_rect(
                    Rect {
                        x: strip_x + 42.0,
                        y: led_y,
                        width: 8.0,
                        height: led_height,
                    },
                    led_color,
                );
            }

            // Fader Track
            renderer.fill_rect(
                Rect {
                    x: strip_x + 20.0,
                    y: meter_y,
                    width: 6.0,
                    height: meter_h,
                },
                [0.02, 0.02, 0.02, 1.0],
            );
            renderer.stroke_rect(
                Rect {
                    x: strip_x + 20.0,
                    y: meter_y,
                    width: 6.0,
                    height: meter_h,
                },
                [0.2, 0.2, 0.2, 1.0],
                1.0,
            );

            // Mixer Fader Handle - INTERACTIVE
            let fader_rect = Rect {
                x: strip_x + 6.0,
                y: meter_y,
                width: 34.0,
                height: meter_h,
            };
            renderer.push_vnode(fader_rect, "MixerVolFader");
            let state_clone = self.state.clone();
            let on_move = std::sync::Arc::new(move |event| {
                if let Event::PointerDown { y, .. } | Event::PointerMove { y, .. } = event {
                    let relative = 1.0 - ((y - meter_y) / meter_h).clamp(0.0, 1.0);
                    state_clone.lock().unwrap().tracks[i].volume = relative;
                }
            });
            renderer.register_handler("pointerdown", on_move.clone());
            renderer.register_handler("pointermove", on_move);

            // Draw fader cap
            let track = &state.tracks[i];
            let fader_y = meter_y + meter_h - (meter_h * track.volume);

            // Handle Shadow
            renderer.fill_rect(
                Rect {
                    x: strip_x + 8.0,
                    y: fader_y - 8.0,
                    width: 30.0,
                    height: 20.0,
                },
                [0.0, 0.0, 0.0, 0.5],
            );

            // Handle Body
            renderer.fill_rect(
                Rect {
                    x: strip_x + 10.0,
                    y: fader_y - 10.0,
                    width: 26.0,
                    height: 20.0,
                },
                [0.35, 0.36, 0.38, 1.0],
            );
            renderer.stroke_rect(
                Rect {
                    x: strip_x + 10.0,
                    y: fader_y - 10.0,
                    width: 26.0,
                    height: 20.0,
                },
                [0.1, 0.1, 0.1, 1.0],
                1.0,
            );

            // Handle highlight bevel
            renderer.fill_rect(
                Rect {
                    x: strip_x + 11.0,
                    y: fader_y - 9.0,
                    width: 24.0,
                    height: 2.0,
                },
                [0.6, 0.6, 0.6, 1.0],
            );

            // White center indicator line
            renderer.fill_rect(
                Rect {
                    x: strip_x + 12.0,
                    y: fader_y - 1.0,
                    width: 22.0,
                    height: 2.0,
                },
                [0.9, 0.9, 0.95, 1.0],
            );
            renderer.pop_vnode();
        }

        // Draw Master Strip - INTERACTIVE
        if master_x > start_x {
            let strip_w = 80.0;
            renderer.fill_rect(
                Rect {
                    x: master_x,
                    y: mixer_y + 10.0,
                    width: strip_w,
                    height: bottom_mixer_height - 20.0,
                },
                [0.14, 0.15, 0.16, 1.0],
            );
            renderer.stroke_rect(
                Rect {
                    x: master_x,
                    y: mixer_y + 10.0,
                    width: strip_w,
                    height: bottom_mixer_height - 20.0,
                },
                [0.05, 0.05, 0.06, 1.0],
                1.0,
            );

            // Track Color Label at Bottom
            let label_rect = Rect {
                x: master_x + 1.0,
                y: mixer_y + bottom_mixer_height - 30.0,
                width: 78.0,
                height: 20.0,
            };
            renderer.fill_rect(label_rect, [0.8, 0.2, 0.2, 1.0]);
            renderer.draw_text_centered("MASTER", label_rect.x, label_rect.y, 11.0, text_light);

            let meter_h = bottom_mixer_height - 80.0;
            let meter_y = mixer_y + 30.0;

            // Dual master LEDs
            for j in 0..2 {
                let mx = master_x + 40.0 + (j as f32 * 14.0);
                renderer.fill_rect(
                    Rect {
                        x: mx,
                        y: meter_y,
                        width: 12.0,
                        height: meter_h,
                    },
                    [0.03, 0.03, 0.04, 1.0],
                );
                renderer.stroke_rect(
                    Rect {
                        x: mx,
                        y: meter_y,
                        width: 12.0,
                        height: meter_h,
                    },
                    [0.2, 0.2, 0.2, 1.0],
                    1.0,
                );

                let master_level = if state.playing {
                    (f32::sin(state.playhead_pos * 0.1).abs() * state.master_volume).min(1.0)
                } else {
                    0.0
                };
                let num_leds = 20;
                let led_height = (meter_h / num_leds as f32) - 1.0;
                let active_leds = (num_leds as f32 * master_level) as usize;

                for led in 0..num_leds {
                    let led_y = meter_y + meter_h - ((led + 1) as f32 * (led_height + 1.0));
                    let led_color = if led < active_leds {
                        if led > 16 {
                            [0.9, 0.2, 0.2, 1.0]
                        } else if led > 12 {
                            [0.9, 0.8, 0.2, 1.0]
                        } else {
                            [0.2, 0.9, 0.4, 1.0]
                        }
                    } else {
                        [0.08, 0.08, 0.08, 1.0]
                    };
                    renderer.fill_rect(
                        Rect {
                            x: mx + 2.0,
                            y: led_y,
                            width: 8.0,
                            height: led_height,
                        },
                        led_color,
                    );
                }
            }

            // Master Fader Track
            renderer.fill_rect(
                Rect {
                    x: master_x + 20.0,
                    y: meter_y,
                    width: 6.0,
                    height: meter_h,
                },
                [0.02, 0.02, 0.02, 1.0],
            );
            renderer.stroke_rect(
                Rect {
                    x: master_x + 20.0,
                    y: meter_y,
                    width: 6.0,
                    height: meter_h,
                },
                [0.2, 0.2, 0.2, 1.0],
                1.0,
            );

            // Master Fader interactive
            let master_rect = Rect {
                x: master_x + 8.0,
                y: meter_y,
                width: 30.0,
                height: meter_h,
            };
            renderer.push_vnode(master_rect, "master_vol");
            let state_clone = self.state.clone();
            let on_move = std::sync::Arc::new(move |event| {
                if let Event::PointerDown { y, .. } | Event::PointerMove { y, .. } = event {
                    let relative = 1.0 - ((y - meter_y) / meter_h).clamp(0.0, 1.0);
                    state_clone.lock().unwrap().master_volume = relative;
                }
            });
            renderer.register_handler("pointerdown", on_move.clone());
            renderer.register_handler("pointermove", on_move);

            let fader_y = meter_y + meter_h - (meter_h * state.master_volume);
            renderer.fill_rect(
                Rect {
                    x: master_x + 8.0,
                    y: fader_y - 10.0,
                    width: 28.0,
                    height: 20.0,
                },
                [0.8, 0.2, 0.2, 1.0],
            ); // Red master cap
            renderer.stroke_rect(
                Rect {
                    x: master_x + 8.0,
                    y: fader_y - 10.0,
                    width: 28.0,
                    height: 20.0,
                },
                [0.1, 0.1, 0.1, 1.0],
                1.0,
            );
            renderer.fill_rect(
                Rect {
                    x: master_x + 10.0,
                    y: fader_y - 1.0,
                    width: 24.0,
                    height: 2.0,
                },
                [1.0, 1.0, 1.0, 1.0],
            );
            renderer.pop_vnode();
        }

        // 6. Top Bar (Transport Controls)
        renderer.fill_rect(
            Rect {
                x: 0.0,
                y: 0.0,
                width: rect.width,
                height: top_bar_height,
            },
            panel_bg,
        );
        renderer.fill_rect(
            Rect {
                x: 0.0,
                y: top_bar_height - 1.0,
                width: rect.width,
                height: 1.0,
            },
            border_color,
        );

        // Digital Timecode Display Box
        let minutes = (elapsed / 60.0).floor() as u32;
        let seconds = (elapsed % 60.0).floor() as u32;
        let millis = (elapsed.fract() * 1000.0) as u32;

        let time_box_x = rect.width / 2.0 - 80.0;
        renderer.fill_rect(
            Rect {
                x: time_box_x,
                y: 8.0,
                width: 160.0,
                height: 34.0,
            },
            [0.05, 0.06, 0.07, 1.0],
        );
        renderer.stroke_rect(
            Rect {
                x: time_box_x,
                y: 8.0,
                width: 160.0,
                height: 34.0,
            },
            [0.2, 0.2, 0.22, 1.0],
            1.0,
        );

        renderer.draw_text(
            &format!("{:02}:{:02}.{:03}", minutes, seconds, millis),
            time_box_x + 15.0,
            16.0,
            20.0,
            [0.4, 0.95, 0.5, 1.0], // Sharp bright green LCD text
        );

        // Transport Controls Background Plate
        let trans_x = left_panel_width + 10.0;
        renderer.fill_rect(
            Rect {
                x: trans_x,
                y: 8.0,
                width: 160.0,
                height: 34.0,
            },
            [0.10, 0.11, 0.12, 1.0],
        );
        renderer.stroke_rect(
            Rect {
                x: trans_x,
                y: 8.0,
                width: 160.0,
                height: 34.0,
            },
            border_color,
            1.0,
        );

        let is_playing = state.playing;
        let state_clone = self.state.clone();

        // Custom Play/Pause Button - INTERACTIVE
        let btn_x = trans_x + 40.0;
        let btn_y = 12.0;
        let btn_rect = Rect {
            x: btn_x,
            y: btn_y,
            width: 80.0,
            height: 26.0,
        };
        renderer.push_vnode(btn_rect, "PlayButton");
        renderer.register_handler(
            "pointerdown",
            std::sync::Arc::new(move |_| {
                let mut s = state_clone.lock().unwrap();
                s.playing = !s.playing;
                if s.playing {
                    s.start_time = Instant::now()
                        - std::time::Duration::from_secs_f32((s.playhead_pos - 200.0) / 80.0);
                }
            }),
        );

        renderer.fill_rect(
            btn_rect,
            if is_playing {
                [0.2, 0.8, 0.4, 1.0]
            } else {
                [0.2, 0.22, 0.25, 1.0]
            },
        );
        renderer.stroke_rect(btn_rect, border_color, 1.0);

        // Draw PLAY/PAUSE label centered
        renderer.draw_text_centered(
            if is_playing { "PAUSE" } else { "PLAY" },
            btn_rect.x,
            btn_rect.y,
            12.0,
            if is_playing { text_dark } else { text_light },
        );
        renderer.pop_vnode();
    }
}

fn main() {
    let tracks = vec![
        TrackState {
            name: "Drums".into(),
            muted: false,
            soloed: false,
            armed: false,
            volume: 0.8,
            pan: 0.5,
            waveform_seed: 1.1,
            color: [0.95, 0.45, 0.45, 1.0],
        },
        TrackState {
            name: "Bass".into(),
            muted: false,
            soloed: false,
            armed: false,
            volume: 0.9,
            pan: 0.5,
            waveform_seed: 2.3,
            color: [0.45, 0.65, 0.95, 1.0],
        },
        TrackState {
            name: "Synth Pad".into(),
            muted: true,
            soloed: false,
            armed: false,
            volume: 0.6,
            pan: 0.2,
            waveform_seed: 0.5,
            color: [0.85, 0.35, 0.85, 1.0],
        },
        TrackState {
            name: "Lead Vocal".into(),
            muted: false,
            soloed: false,
            armed: true,
            volume: 0.85,
            pan: 0.8,
            waveform_seed: 3.7,
            color: [0.95, 0.85, 0.25, 1.0],
        },
    ];

    let state = Arc::new(Mutex::new(DawState {
        playhead_pos: 200.0,
        start_time: Instant::now(),
        playing: false,
        master_volume: 0.85,
        tracks,
    }));
    cvkg::native::NativeRenderer::run(DawView { state }, None);
}
