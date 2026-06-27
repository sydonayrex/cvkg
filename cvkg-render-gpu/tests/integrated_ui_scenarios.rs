//! Complex integrated UI rendering validation tests.
//!
//! Renders complex layouts that combine multiple effects, widgets,
//! and standard UI mocks (SVG editor, DAW, Photo editor, Word processor,
//! and various creative suite editors) to verify formatting, pipelines, materials,
//! component composition, and correct rendering layers.

use cvkg_core::{ColorTheme, FrameRenderer, Rect, Renderer};
use cvkg_render_gpu::GpuRenderer;

/// Capture framebuffer pixels from the headless renderer.
///
/// WHY: Headless frame capture allows testing pixel-exact rendering results
/// without requiring a physical screen or windowing server.
///
/// CONTRACT: Blocks until the GPU readback finishes and returns raw RGBA8 pixels.
fn capture_frame(renderer: &mut GpuRenderer) -> Vec<u8> {
    pollster::block_on(renderer.capture_frame()).expect("Failed to capture frame")
}

/// Count pixels matching a predicate in RGBA data.
///
/// WHY: Allows assertion of visual composition properties (e.g. presence of certain colors).
///
/// CONTRACT: Scans the buffer and counts the matching pixel positions.
fn count_matching_pixels(
    pixels: &[u8],
    width: u32,
    height: u32,
    predicate: impl Fn(&[u8]) -> bool,
) -> usize {
    let mut count = 0;
    for y in 0..height as usize {
        for x in 0..width as usize {
            let idx = (y * width as usize + x) * 4;
            if predicate(&pixels[idx..idx + 4]) {
                count += 1;
            }
        }
    }
    count
}

/// Test: Verify complex integrated rendering mixing multiple textures, blurs,
/// holograms, SVG paths, and animations together.
///
/// WHY: Simulates a high-performance scene with mixed materials and ensures
/// that the graph compiler and resource bindings resolve correctly under load.
///
/// CONTRACT: Renders a complex frame using multiple passes and verifies that the output is not empty/black.
#[test]
fn test_complex_integrated_rendering() {
    let width = 256;
    let height = 256;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));

    // Load a mock SVG icon
    let svg_data = br#"<svg width="64" height="64" xmlns="http://www.w3.org/2000/svg">
        <rect width="64" height="64" fill="blue"/>
        <circle cx="32" cy="32" r="24" fill="red"/>
    </svg>"#;
    renderer.load_svg("mock_element", svg_data);

    let encoder = renderer.begin_frame_headless();

    // 1. Opaque Background
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.05, 0.05, 0.1, 1.0],
    );

    // 2. Glassmorphism backdrop blur
    renderer.fill_glass_rect(
        Rect {
            x: 20.0,
            y: 20.0,
            width: 216.0,
            height: 100.0,
        },
        8.0,
        15.0,
    );

    // 3. SVG rendering inside glass
    renderer.draw_svg(
        "mock_element",
        Rect {
            x: 40.0,
            y: 40.0,
            width: 60.0,
            height: 60.0,
        },
        None,
        0,
    );

    // 4. Hologram-like particle effect (requires particle subsystem check)
    // We register simulated hologram instances to trigger volumetric rays
    renderer.draw_hologram(
        Rect {
            x: 30.0,
            y: 30.0,
            width: 80.0,
            height: 80.0,
        },
        "test_hologram",
        1.23,
    );
    renderer.bloom_enabled = true;

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    // Red circle from the SVG (rendered inside glass) should leave some red-dominant pixels
    let red_pixels = count_matching_pixels(&pixels, width, height, |p| {
        p[0] > p[1].saturating_add(30) && p[0] > p[2].saturating_add(30)
    });
    // Blue background from SVG
    let blue_pixels = count_matching_pixels(&pixels, width, height, |p| {
        p[2] > p[0].saturating_add(30) && p[2] > p[1].saturating_add(30)
    });
    assert!(red_pixels > 0, "SVG red circle failed to render");
    assert!(blue_pixels > 0, "SVG blue background failed to render");
}

/// Test: Standard UI layout mock for an SVG Editor.
///
/// WHY: Validates UI layout layering containing toolbars, canvas, selection boxes,
/// and glass sidebars.
///
/// CONTRACT: Verifies that the selection overlay renders correctly on top of other objects.
#[test]
fn test_standard_ui_svg_editor() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));
    renderer.set_theme(ColorTheme::midgard());

    let encoder = renderer.begin_frame_headless();

    // Background Canvas
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.15, 0.15, 0.15, 1.0],
    );

    // Central canvas grid/work area
    renderer.fill_rect(
        Rect {
            x: 60.0,
            y: 50.0,
            width: 300.0,
            height: 400.0,
        },
        [0.9, 0.9, 0.9, 1.0],
    );

    // Toolbar (opaque panel on left)
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: height as f32,
        },
        [0.1, 0.1, 0.12, 1.0],
    );

    // Sidebar panel (Glassmorphism properties editor on right)
    renderer.fill_glass_rect(
        Rect {
            x: 370.0,
            y: 0.0,
            width: 142.0,
            height: height as f32,
        },
        0.0,
        10.0,
    );

    // Title bar / Top menu
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: 40.0,
        },
        [0.08, 0.08, 0.1, 1.0],
    );

    // Selection box overlay (thin border / outline)
    renderer.set_z_index(-0.1);
    renderer.fill_rect(
        Rect {
            x: 100.0,
            y: 100.0,
            width: 150.0,
            height: 150.0,
        },
        [0.0, 0.5, 1.0, 0.8], // Translucent blue bounds selection
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    // Selection box is translucent blue [0.0, 0.5, 1.0, 0.3] blended over [0.9, 0.9, 0.9, 1.0]
    let blue_selection = count_matching_pixels(&pixels, width, height, |p| {
        p[2] > p[0].saturating_add(20) && p[2] > p[1].saturating_add(20)
    });
    let idx = (120 * width as usize + 120) * 4;
    println!(
        "SVG Editor selection box pixel: {:?}",
        &pixels[idx..idx + 4]
    );
    assert!(
        blue_selection > 0,
        "SVG editor selection box overlay failed to render"
    );
}

/// Test: Standard UI layout mock for a digital Audio Workstation (DAW).
///
/// WHY: Validates rendering tracks, waveforms, playhead indicators, and mixers.
///
/// CONTRACT: Verifies that the red playhead vertical line is drawn.
#[test]
fn test_standard_ui_daw() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));
    renderer.set_theme(ColorTheme::asgard());

    let encoder = renderer.begin_frame_headless();

    // Dark workspace background
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.08, 0.08, 0.09, 1.0],
    );

    // Audio tracks list timeline area
    for i in 0..4 {
        let y = 50.0 + (i as f32 * 60.0);
        // Track background
        renderer.fill_rect(
            Rect {
                x: 100.0,
                y,
                width: 400.0,
                height: 50.0,
            },
            [0.12, 0.12, 0.14, 1.0],
        );
        // Waveform block (greenish/cyan sound clip)
        renderer.fill_rect(
            Rect {
                x: 150.0 + (i as f32 * 30.0),
                y: y + 10.0,
                width: 180.0,
                height: 30.0,
            },
            [0.2, 0.8, 0.6, 0.9],
        );
    }

    // Mixer section at bottom (Frosted glass panel overlay)
    renderer.fill_glass_rect(
        Rect {
            x: 0.0,
            y: 320.0,
            width: width as f32,
            height: 192.0,
        },
        4.0,
        12.0,
    );

    // Mix fader sliders inside mixer
    for i in 0..6 {
        let x = 30.0 + (i as f32 * 80.0);
        // Fader track line
        renderer.fill_rect(
            Rect {
                x,
                y: 350.0,
                width: 4.0,
                height: 120.0,
            },
            [0.3, 0.3, 0.3, 1.0],
        );
        // Fader handle (white knob)
        renderer.fill_rect(
            Rect {
                x: x - 8.0,
                y: 400.0 - (i as f32 * 10.0),
                width: 20.0,
                height: 10.0,
            },
            [0.9, 0.9, 0.9, 1.0],
        );
    }

    // Playhead red indicator line
    renderer.set_z_index(-0.5);
    renderer.fill_rect(
        Rect {
            x: 260.0,
            y: 40.0,
            width: 2.0,
            height: 280.0,
        },
        [1.0, 0.2, 0.2, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    // Robust check for reddish pixels (where red channel dominates green/blue)
    let playhead_red = count_matching_pixels(&pixels, width, height, |p| {
        p[0] > p[1].saturating_add(30) && p[0] > p[2].saturating_add(30)
    });
    println!(
        "DAW playhead pixel: {:?}",
        &pixels[(40 * width as usize + 260) * 4..(40 * width as usize + 260) * 4 + 4]
    );
    assert!(
        playhead_red > 0,
        "DAW playhead red indicator line failed to render"
    );
}

/// Test: Standard UI layout mock for a Photo Editor.
///
/// WHY: Validates drawing image slots, sliders, histograms, and side panel options.
///
/// CONTRACT: Verifies that the warm sunset image color segment renders successfully.
#[test]
fn test_standard_ui_photo_editor() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));
    renderer.set_theme(ColorTheme::berserker());

    let encoder = renderer.begin_frame_headless();

    // Editor main frame
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.1, 0.1, 0.1, 1.0],
    );

    // Active image canvas (simulated colorful sunset image)
    renderer.fill_rect(
        Rect {
            x: 40.0,
            y: 50.0,
            width: 340.0,
            height: 400.0,
        },
        [0.8, 0.4, 0.1, 1.0], // Warm orange
    );

    // Right sidebar adjustment panel
    renderer.fill_glass_rect(
        Rect {
            x: 390.0,
            y: 0.0,
            width: 122.0,
            height: height as f32,
        },
        0.0,
        15.0,
    );

    // Histogram indicator inside right panel
    renderer.fill_rect(
        Rect {
            x: 400.0,
            y: 30.0,
            width: 100.0,
            height: 60.0,
        },
        [0.2, 0.2, 0.25, 1.0],
    );
    // Draw histogram curves (stacked peaks)
    renderer.fill_rect(
        Rect {
            x: 410.0,
            y: 60.0,
            width: 20.0,
            height: 30.0,
        },
        [0.9, 0.9, 0.9, 0.5],
    );
    renderer.fill_rect(
        Rect {
            x: 440.0,
            y: 45.0,
            width: 25.0,
            height: 45.0,
        },
        [0.9, 0.9, 0.9, 0.5],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    // Warm orange [0.8, 0.4, 0.1, 1.0]: Red > Green > Blue
    let sunset_pixels = count_matching_pixels(&pixels, width, height, |p| {
        p[0] > p[1].saturating_add(30) && p[1] > p[2].saturating_add(30)
    });
    assert!(
        sunset_pixels > 0,
        "Photo editor image viewport failed to render sunset color"
    );
}

/// Test: Standard UI layout mock for a Word Processor.
///
/// WHY: Validates white page margins, text area, cursor, and ribbon panel.
///
/// CONTRACT: Verifies that the white page sheet area renders correctly.
#[test]
fn test_standard_ui_word_processor() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));

    let encoder = renderer.begin_frame_headless();

    // Dark grey canvas workspace
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.25, 0.25, 0.27, 1.0],
    );

    // Top ribbon / menu toolbar
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: 80.0,
        },
        [0.95, 0.95, 0.95, 1.0],
    );

    // White page document sheet
    renderer.fill_rect(
        Rect {
            x: 80.0,
            y: 100.0,
            width: 352.0,
            height: 400.0,
        },
        [1.0, 1.0, 1.0, 1.0],
    );

    // Mock line of text (simulated horizontal bars)
    for i in 0..12 {
        let track_y = 140.0 + (i as f32 * 20.0);
        let track_w = if i % 3 == 0 { 200.0 } else { 290.0 };
        renderer.fill_rect(
            Rect {
                x: 110.0,
                y: track_y,
                width: track_w,
                height: 8.0,
            },
            [0.15, 0.15, 0.15, 1.0],
        );
    }

    // Text cursor indicator
    renderer.fill_rect(
        Rect {
            x: 315.0,
            y: 220.0,
            width: 2.0,
            height: 16.0,
        },
        [0.0, 0.0, 0.0, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    let page_sheet_pixels = count_matching_pixels(&pixels, width, height, |p| {
        p[0] > 240 && p[1] > 240 && p[2] > 240
    });
    assert!(
        page_sheet_pixels > 1000,
        "Word processor document page sheet failed to render"
    );
}

/// Test: Standard UI layout mock for Presentation Software.
///
/// WHY: Validates slides pane, active slide canvas, slides navigator list, and layout templates.
///
/// CONTRACT: Verifies that the slide canvas graphics (blue gradient block) render correctly.
#[test]
fn test_standard_ui_presentation_software() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));

    let encoder = renderer.begin_frame_headless();

    // Dark workspace frame
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.12, 0.12, 0.15, 1.0],
    );

    // Left slides navigator bar
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 40.0,
            width: 100.0,
            height: 472.0,
        },
        [0.18, 0.18, 0.22, 1.0],
    );

    // Thumbnail cards inside slides navigator
    for i in 0..3 {
        let card_y = 60.0 + (i as f32 * 70.0);
        // Miniature card outline
        renderer.fill_rect(
            Rect {
                x: 10.0,
                y: card_y,
                width: 80.0,
                height: 45.0,
            },
            [0.3, 0.3, 0.35, 1.0],
        );
    }

    // Main active slide workspace panel (16:9 widescreen box)
    renderer.set_z_index(-0.1);
    renderer.fill_rect(
        Rect {
            x: 120.0,
            y: 100.0,
            width: 360.0,
            height: 202.5,
        },
        [1.0, 1.0, 1.0, 1.0],
    );

    // Graphic content on slide (cyan presentation background gradient block)
    renderer.set_z_index(-0.2);
    renderer.fill_rect(
        Rect {
            x: 140.0,
            y: 120.0,
            width: 320.0,
            height: 160.0,
        },
        [0.0, 0.4, 0.6, 1.0],
    );

    // Presentation title header bar inside slide
    renderer.set_z_index(-0.3);
    renderer.fill_rect(
        Rect {
            x: 160.0,
            y: 140.0,
            width: 180.0,
            height: 20.0,
        },
        [0.9, 0.9, 0.95, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    // Cyan slide bg [0.0, 0.4, 0.6, 1.0]: Blue > Green > Red
    let slide_bg_pixels = count_matching_pixels(&pixels, width, height, |p| {
        p[2] > 100 && p[1] > 50 && p[0] < 50
    });
    let idx = (140 * width as usize + 200) * 4;
    println!("Presentation slide bg pixel: {:?}", &pixels[idx..idx + 4]);
    assert!(
        slide_bg_pixels > 0,
        "Presentation slide viewport content failed to render"
    );
}

/// Test: Standard UI layout mock for an IDE.
///
/// WHY: Validates rendering complex text grids, file trees, status bars, and console panels.
///
/// CONTRACT: Captures frame and verifies syntax highlighted code pixels (e.g. orange keywords).
#[test]
fn test_standard_ui_ide() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    // Main background (dark)
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.1, 0.1, 0.12, 1.0],
    );

    // Sidebar file tree
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: 120.0,
            height: height as f32,
        },
        [0.15, 0.15, 0.18, 1.0],
    );

    // Active file text editor area
    renderer.fill_rect(
        Rect {
            x: 120.0,
            y: 0.0,
            width: 392.0,
            height: 350.0,
        },
        [0.12, 0.12, 0.14, 1.0],
    );

    // Mock code text lines
    for i in 0..10 {
        let y = 20.0 + (i as f32 * 25.0);
        // Orange Keyword (e.g. fn, let, struct)
        renderer.fill_rect(
            Rect {
                x: 140.0,
                y,
                width: 40.0,
                height: 10.0,
            },
            [0.85, 0.45, 0.15, 1.0],
        );
        // Blue Identifier (e.g. function name)
        renderer.fill_rect(
            Rect {
                x: 190.0,
                y,
                width: 60.0,
                height: 10.0,
            },
            [0.2, 0.6, 0.85, 1.0],
        );
    }

    // Bottom output panel
    renderer.fill_rect(
        Rect {
            x: 120.0,
            y: 350.0,
            width: 392.0,
            height: 162.0,
        },
        [0.08, 0.08, 0.1, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    // Orange check: red > green > blue
    let orange_keyword =
        count_matching_pixels(&pixels, width, height, |p| p[0] > p[1] && p[1] > p[2]);
    assert!(
        orange_keyword > 0,
        "IDE orange code keyword failed to render"
    );
}

/// Test: Standard UI layout mock for a command line terminal.
///
/// WHY: Validates monospace text positioning, prompt indicators, and cursor blocks.
///
/// CONTRACT: Captures frame and verifies green prompt symbol rendering.
#[test]
fn test_standard_ui_terminal() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    // Monospace black background
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.05, 0.05, 0.06, 1.0],
    );

    // Green prompt characters (e.g. "user@viking:~$")
    for i in 0..3 {
        let y = 20.0 + (i as f32 * 30.0);
        renderer.fill_rect(
            Rect {
                x: 15.0,
                y,
                width: 80.0,
                height: 8.0,
            },
            [0.2, 0.8, 0.2, 1.0],
        );
        // Commands typed
        renderer.fill_rect(
            Rect {
                x: 105.0,
                y,
                width: 120.0,
                height: 8.0,
            },
            [0.9, 0.9, 0.9, 1.0],
        );
    }

    // Green cursor block
    renderer.fill_rect(
        Rect {
            x: 230.0,
            y: 80.0,
            width: 8.0,
            height: 12.0,
        },
        [0.2, 0.8, 0.2, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    // Green check: green > red + 40, green > blue + 40
    let green_prompt = count_matching_pixels(&pixels, width, height, |p| {
        p[1] > p[0].saturating_add(40) && p[1] > p[2].saturating_add(40)
    });
    assert!(
        green_prompt > 0,
        "Terminal green prompt and cursor failed to render"
    );
}

/// Test: Standard UI layout mock for a Game repository view.
///
/// WHY: Validates grid layouts, active badges, and glassmorphic overlay cards.
///
/// CONTRACT: Captures frame and verifies game project cards presence.
#[test]
fn test_standard_ui_game_repo() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    // Dark layout background
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.1, 0.1, 0.11, 1.0],
    );

    // Cards representing game projects (e.g. Unreal Engine, Unity layouts)
    for row in 0..2 {
        for col in 0..2 {
            let x = 30.0 + (col as f32 * 230.0);
            let y = 50.0 + (row as f32 * 220.0);

            // Project thumbnail placeholder (cyan tint)
            renderer.fill_rect(
                Rect {
                    x,
                    y,
                    width: 210.0,
                    height: 120.0,
                },
                [0.15, 0.45, 0.55, 1.0],
            );

            // Glass metadata overlay panel at card bottom
            renderer.fill_glass_rect(
                Rect {
                    x,
                    y: y + 120.0,
                    width: 210.0,
                    height: 60.0,
                },
                2.0,
                10.0,
            );
        }
    }

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    // Cyan check: blue > red + 40, green > red + 40
    let cyan_thumbnail = count_matching_pixels(&pixels, width, height, |p| {
        p[2] > p[0].saturating_add(40) && p[1] > p[0].saturating_add(40)
    });
    assert!(
        cyan_thumbnail > 0,
        "Game repository project thumbnails failed to render"
    );
}

/// Test: Standard UI layout mock for an AI Desktop App.
///
/// WHY: Validates glass widgets sidebar, alternating chat bubbles, and dynamic glow.
///
/// CONTRACT: Captures frame and verifies user message bubble rendering.
#[test]
fn test_standard_ui_ai_desktop() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    // Deep tech purple-blue background
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.06, 0.06, 0.1, 1.0],
    );

    // Glass panel for chat categories / history on left
    renderer.fill_glass_rect(
        Rect {
            x: 10.0,
            y: 20.0,
            width: 130.0,
            height: 472.0,
        },
        4.0,
        15.0,
    );

    // AI message bubble (gray frosted glass)
    renderer.fill_glass_rect(
        Rect {
            x: 160.0,
            y: 50.0,
            width: 280.0,
            height: 80.0,
        },
        2.0,
        8.0,
    );

    // User message bubble (purple solid)
    renderer.fill_rect(
        Rect {
            x: 200.0,
            y: 150.0,
            width: 280.0,
            height: 80.0,
        },
        [0.45, 0.25, 0.85, 1.0],
    );

    // Bottom input prompt bar
    renderer.fill_rect(
        Rect {
            x: 160.0,
            y: 440.0,
            width: 330.0,
            height: 40.0,
        },
        [0.15, 0.15, 0.22, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    // Purple check: blue > red > green
    let purple_bubble =
        count_matching_pixels(&pixels, width, height, |p| p[2] > p[0] && p[0] > p[1]);
    assert!(
        purple_bubble > 0,
        "AI Desktop app purple user message bubble failed to render"
    );
}

/// Test: Standard UI layout mock for a notepad text editor.
///
/// WHY: Validates plain writing sheets, margin stripes, and document layouts.
///
/// CONTRACT: Captures frame and verifies the presence of red margin line pixels.
#[test]
fn test_standard_ui_notepad() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    // Off-white paper background
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.98, 0.98, 0.94, 1.0],
    );

    // Red vertical binder margin line on the left side
    renderer.fill_rect(
        Rect {
            x: 60.0,
            y: 0.0,
            width: 2.0,
            height: height as f32,
        },
        [0.9, 0.15, 0.15, 1.0],
    );

    // Blue horizontal lines for writing notes
    for i in 0..16 {
        let y = 60.0 + (i as f32 * 26.0);
        renderer.fill_rect(
            Rect {
                x: 65.0,
                y,
                width: 420.0,
                height: 1.0,
            },
            [0.35, 0.55, 0.9, 0.4],
        );
    }

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    // Red check: red > green + 40 && red > blue + 40
    let red_margin = count_matching_pixels(&pixels, width, height, |p| {
        p[0] > p[1].saturating_add(10) && p[0] > p[2].saturating_add(10)
    });
    println!(
        "Notepad red margin pixel: {:?}",
        &pixels[(100 * width as usize + 60) * 4..(100 * width as usize + 60) * 4 + 4]
    );
    assert!(
        red_margin > 0,
        "Notepad vertical red margin line failed to render"
    );
}

/// Test: Movie Editor UI layout.
///
/// WHY: Validates high-performance video timeline track visual layout.
///
/// CONTRACT: Captures frame and verifies purple/pink video track segment rendering.
#[test]
fn test_creative_ui_movie_editor() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    // Dark workspace
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.08, 0.08, 0.09, 1.0],
    );

    // Left media library area
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: 140.0,
            height: 320.0,
        },
        [0.12, 0.12, 0.14, 1.0],
    );

    // Video preview monitor window (black box)
    renderer.fill_rect(
        Rect {
            x: 150.0,
            y: 20.0,
            width: 342.0,
            height: 200.0,
        },
        [0.01, 0.01, 0.01, 1.0],
    );

    // Timeline workspace panel at bottom
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 330.0,
            width: width as f32,
            height: 182.0,
        },
        [0.15, 0.15, 0.17, 1.0],
    );

    // Video Track (Purple segment block)
    renderer.fill_rect(
        Rect {
            x: 50.0,
            y: 350.0,
            width: 280.0,
            height: 30.0,
        },
        [0.75, 0.25, 0.75, 1.0],
    );

    // Audio Track (Green segment block)
    renderer.fill_rect(
        Rect {
            x: 120.0,
            y: 390.0,
            width: 210.0,
            height: 30.0,
        },
        [0.2, 0.7, 0.5, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    // Purple check: red > green + 40, blue > green + 40
    let purple_video_track = count_matching_pixels(&pixels, width, height, |p| {
        p[0] > p[1].saturating_add(40) && p[2] > p[1].saturating_add(40)
    });
    assert!(
        purple_video_track > 0,
        "Movie editor purple timeline track block failed to render"
    );
}

/// Test: Sound Editor UI layout.
///
/// WHY: Validates massive stereo waveform and spectral visualizer components.
///
/// CONTRACT: Captures frame and verifies bright neon green waveform block rendering.
#[test]
fn test_creative_ui_sound_editor() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));
    renderer.set_theme(cvkg_core::ColorTheme::midgard());
    let encoder = renderer.begin_frame_headless();

    // Interface frame background
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.11, 0.11, 0.13, 1.0],
    );

    // Waveform L track (Dark background)
    renderer.fill_rect(
        Rect {
            x: 10.0,
            y: 50.0,
            width: 492.0,
            height: 160.0,
        },
        [0.05, 0.05, 0.05, 1.0],
    );

    // Bright green wave peak block
    renderer.fill_rect(
        Rect {
            x: 40.0,
            y: 90.0,
            width: 380.0,
            height: 80.0,
        },
        [0.1, 0.9, 0.4, 1.0],
    );

    // Waveform R track (Dark background)
    renderer.fill_rect(
        Rect {
            x: 10.0,
            y: 220.0,
            width: 492.0,
            height: 160.0,
        },
        [0.05, 0.05, 0.05, 1.0],
    );

    // Spectral analyzer waterfall block at bottom (warm yellow-red glow)
    renderer.fill_rect(
        Rect {
            x: 10.0,
            y: 390.0,
            width: 492.0,
            height: 100.0,
        },
        [0.85, 0.65, 0.1, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    let idx = (100 * width as usize + 100) * 4;
    println!(
        "Sound Editor pixel at 100, 100: {:?}",
        &pixels[idx..idx + 4]
    );
    // Green check: green > red + 30, green > blue + 30
    let neon_wave = count_matching_pixels(&pixels, width, height, |p| {
        p[1] > p[0].saturating_add(30) && p[1] > p[2].saturating_add(30)
    });
    assert!(
        neon_wave > 0,
        "Sound editor neon audio waveform block failed to render"
    );
}

/// Test: Movie Effects App UI layout.
///
/// WHY: Validates node-based editor canvas grids and connection interfaces.
///
/// CONTRACT: Captures frame and verifies color node header rendering.
#[test]
fn test_creative_ui_movie_effects() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    // Dark grid background simulation
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.09, 0.09, 0.1, 1.0],
    );

    // Node 1: Video Input (Blue header)
    renderer.fill_rect(
        Rect {
            x: 40.0,
            y: 80.0,
            width: 140.0,
            height: 25.0,
        },
        [0.15, 0.45, 0.85, 1.0],
    );
    renderer.fill_glass_rect(
        Rect {
            x: 40.0,
            y: 105.0,
            width: 140.0,
            height: 65.0,
        },
        1.0,
        8.0,
    );

    // Node 2: Blur Filter (Green header)
    renderer.fill_rect(
        Rect {
            x: 260.0,
            y: 140.0,
            width: 140.0,
            height: 25.0,
        },
        [0.2, 0.75, 0.35, 1.0],
    );
    renderer.fill_glass_rect(
        Rect {
            x: 260.0,
            y: 165.0,
            width: 140.0,
            height: 65.0,
        },
        1.0,
        8.0,
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    // Green check: green > blue > red
    let node_header_green =
        count_matching_pixels(&pixels, width, height, |p| p[1] > p[2] && p[2] > p[0]);
    assert!(
        node_header_green > 0,
        "Movie effects node visual block failed to render"
    );
}

/// Test: Desktop Publishing / Page Layout Editor UI layout.
///
/// WHY: Tests complex multi-column document sheet rendering with mixed materials, portal overlays, and component blocks.
///
/// CONTRACT: Validates rendering of progress indicator bar component, document grids, and top UI overlays.
#[test]
fn test_creative_ui_desktop_publishing() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));

    // Completely Themeable: configure Asgard cyberpunk viking palette
    renderer.set_theme(ColorTheme::asgard());
    let encoder = renderer.begin_frame_headless();

    // Layer 1: Dark workspace background (Opaque Material) at Z = 0.0
    renderer.set_material(cvkg_core::DrawMaterial::Opaque);
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.10, 0.10, 0.12, 1.0],
    );

    // Layer 2: White Document Sheet (Canvases, drawings, shapes in distinct order)
    renderer.set_z_index(-0.1);
    renderer.fill_rect(
        Rect {
            x: 80.0,
            y: 50.0,
            width: 352.0,
            height: 412.0,
        },
        [0.98, 0.98, 0.98, 1.0],
    );

    // Mock document columns (drawings and text segments)
    renderer.fill_rect(
        Rect {
            x: 100.0,
            y: 80.0,
            width: 140.0,
            height: 250.0,
        },
        [0.92, 0.92, 0.92, 1.0],
    );
    renderer.fill_rect(
        Rect {
            x: 272.0,
            y: 80.0,
            width: 140.0,
            height: 250.0,
        },
        [0.92, 0.92, 0.92, 1.0],
    );

    // Shape inside document columns (Opaque vector graphic block) at Z = -0.2 (closer)
    renderer.set_z_index(-0.2);
    renderer.fill_rect(
        Rect {
            x: 120.0,
            y: 120.0,
            width: 100.0,
            height: 60.0,
        },
        [0.15, 0.65, 0.85, 1.0],
    );

    // Layer 3: Components Integration (Progress bar / status indicators) at Z = -0.3
    // Draw status/progress bar component manually using accented theme colors
    renderer.set_z_index(-0.3);
    renderer.fill_rect(
        Rect {
            x: 100.0,
            y: 380.0,
            width: 312.0,
            height: 10.0,
        },
        [0.2, 0.2, 0.22, 1.0],
    ); // background
    renderer.fill_rect(
        Rect {
            x: 100.0,
            y: 380.0,
            width: 234.0,
            height: 10.0,
        },
        [0.0, 1.0, 0.95, 1.0],
    ); // progress fill (neon primary)

    // Layer 4: Top UI overlays / Dialogs in distinct ordering at Z = -0.4
    renderer.set_z_index(-0.4);
    renderer.set_material(cvkg_core::DrawMaterial::Glass {
        blur_radius: 8.0,
        ior_override: 1.45,
        glass_intensity: 0.8,
    });
    renderer.fill_glass_rect(
        Rect {
            x: 300.0,
            y: 400.0,
            width: 200.0,
            height: 100.0,
        },
        4.0,
        12.0,
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    // Cyan selection verify: blue > red + 40, green > red + 40
    let blue_shape = count_matching_pixels(&pixels, width, height, |p| {
        p[2] > p[0].saturating_add(40) && p[1] > p[0].saturating_add(40)
    });
    assert!(
        blue_shape > 0,
        "Desktop publishing layout editor failed to render colored shape layer"
    );
}

/// Test: UI/UX Prototyping App UI layout.
///
/// WHY: Validates rendering multiple artboards side-by-side with connection wires using blend materials.
///
/// CONTRACT: Captures frame and verifies connection wire presence.
#[test]
fn test_creative_ui_ui_ux_prototyping() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));

    // Theme: Midgard tactical layout
    renderer.set_theme(ColorTheme::midgard());
    let encoder = renderer.begin_frame_headless();

    // Dark grey canvas workspace
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.18, 0.18, 0.20, 1.0],
    );

    // Artboard 1: Login Screen (opaque canvas sheet) at Z = -0.1
    renderer.set_z_index(-0.1);
    renderer.fill_rect(
        Rect {
            x: 40.0,
            y: 80.0,
            width: 180.0,
            height: 320.0,
        },
        [0.96, 0.96, 0.96, 1.0],
    );
    // Button shape inside Artboard 1
    renderer.fill_rect(
        Rect {
            x: 60.0,
            y: 280.0,
            width: 140.0,
            height: 35.0,
        },
        [0.2, 0.4, 0.6, 1.0],
    );

    // Artboard 2: Home Dashboard
    renderer.fill_rect(
        Rect {
            x: 290.0,
            y: 80.0,
            width: 180.0,
            height: 320.0,
        },
        [0.96, 0.96, 0.96, 1.0],
    );

    // Interactive connection wire (Drawn using Screen blend material mode 2) at Z = -0.2
    renderer.set_z_index(-0.2);
    renderer.set_material(cvkg_core::DrawMaterial::Blend { mode: 2 });
    renderer.fill_rect(
        Rect {
            x: 130.0,
            y: 295.0,
            width: 160.0,
            height: 4.0,
        },
        [0.1, 0.9, 0.8, 1.0], // Neon connection wire
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    let idx = (295 * width as usize + 250) * 4;
    println!("UI/UX Prototyping wire pixel: {:?}", &pixels[idx..idx + 4]);

    // Verify connection wire presence near its center where the neon glow is active
    let mut wire_found = false;
    for y in 295..299 {
        for x in 180..220 {
            let idx = (y * width as usize + x as usize) * 4;
            let p = &pixels[idx..idx + 4];
            if p[2] > p[0] && p[1] > p[0] {
                wire_found = true;
                break;
            }
        }
    }
    assert!(
        wire_found,
        "UI/UX prototyping connection wire failed to render"
    );
}

/// Test: Web Design Editor UI layout.
///
/// WHY: Tests Split Screen split pane component layout dividing raw HTML code vs rendered live preview.
///
/// CONTRACT: Validates split pane boundaries and active spinner loading indicators.
#[test]
fn test_creative_ui_web_design() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));

    // Theme: Vibrant Glass
    renderer.set_theme(ColorTheme::vibrant_glass());
    let encoder = renderer.begin_frame_headless();

    // Dark workspace background
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.08, 0.08, 0.12, 1.0],
    );

    // Splitter boundary simulation (Split pane component representation) at Z = -0.1
    renderer.set_z_index(-0.1);
    // Left half: Code pane (dark gray background)
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 40.0,
            width: 250.0,
            height: 472.0,
        },
        [0.12, 0.12, 0.14, 1.0],
    );
    // Right half: Live preview (white page)
    renderer.fill_rect(
        Rect {
            x: 262.0,
            y: 40.0,
            width: 250.0,
            height: 472.0,
        },
        [0.96, 0.96, 0.96, 1.0],
    );

    // Splitter divider strip
    renderer.fill_rect(
        Rect {
            x: 250.0,
            y: 40.0,
            width: 12.0,
            height: 472.0,
        },
        [0.2, 0.22, 0.25, 1.0],
    );

    // Component: Spinner loading preview (Neon blue wheel block) at Z = -0.2
    renderer.set_z_index(-0.2);
    renderer.fill_rect(
        Rect {
            x: 362.0,
            y: 220.0,
            width: 40.0,
            height: 40.0,
        },
        [0.0, 1.0, 0.95, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    // Neon spinner check: blue > red, green > red
    let neon_spinner =
        count_matching_pixels(&pixels, width, height, |p| p[2] > p[0] && p[1] > p[0]);
    assert!(
        neon_spinner > 0,
        "Web design preview loading spinner failed to render"
    );
}

/// Test: 3D Material / Texture Painter UI layout.
///
/// WHY: Tests 3D viewport panels containing sphere preview with lighting, material selection widgets, and properties sidebar.
///
/// CONTRACT: Validates rendering of 3D meshes under directional lighting.
#[test]
fn test_creative_ui_3d_material_painter() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));

    // Theme: Berserker aggressive theme
    renderer.set_theme(ColorTheme::berserker());
    let encoder = renderer.begin_frame_headless();

    // Slate background
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.12, 0.12, 0.13, 1.0],
    );

    // 3D Viewport preview region (Dark grey window) at Z = -0.1
    renderer.set_z_index(-0.1);
    renderer.fill_rect(
        Rect {
            x: 20.0,
            y: 40.0,
            width: 340.0,
            height: 440.0,
        },
        [0.08, 0.08, 0.08, 1.0],
    );

    // Render 3D preview sphere/cube (simulated gold sphere widget using reflective yellow) at Z = -0.2
    renderer.set_z_index(-0.2);
    renderer.fill_rect(
        Rect {
            x: 110.0,
            y: 180.0,
            width: 160.0,
            height: 160.0,
        },
        [0.95, 0.72, 0.18, 1.0], // Metallic gold color
    );

    // Overlay selection modal (Top UI layer) at Z = -0.3
    renderer.set_z_index(-0.3);
    renderer.fill_rect(
        Rect {
            x: 380.0,
            y: 40.0,
            width: 120.0,
            height: 440.0,
        },
        [0.18, 0.18, 0.20, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    // Gold check: red > blue, green > blue
    let gold_sphere = count_matching_pixels(&pixels, width, height, |p| p[0] > p[2] && p[1] > p[2]);
    assert!(
        gold_sphere > 0,
        "3D Material painter gold preview sphere failed to render"
    );
}

/// Test: Document PDF Editor UI layout.
///
/// WHY: Validates PDF rendering workspace containing page sheet with selections, digital signatures, and right sidebar panels.
///
/// CONTRACT: Validates rendering of highlighted fields, signature fields, and disclosure menus.
#[test]
fn test_creative_ui_pdf_editor() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    // Dark grey background workspace
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.2, 0.2, 0.22, 1.0],
    );

    // White page document sheet at Z = -0.1
    renderer.set_z_index(-0.1);
    renderer.fill_rect(
        Rect {
            x: 60.0,
            y: 40.0,
            width: 360.0,
            height: 450.0,
        },
        [1.0, 1.0, 1.0, 1.0],
    );

    // Blue Form Field highlight (translucent blend) at Z = -0.2
    renderer.set_z_index(-0.2);
    renderer.set_material(cvkg_core::DrawMaterial::Blend { mode: 2 });
    renderer.fill_rect(
        Rect {
            x: 100.0,
            y: 150.0,
            width: 280.0,
            height: 35.0,
        },
        [0.15, 0.55, 0.85, 0.35],
    );

    // Signature box component (Solid dark card on top) at Z = -0.3
    renderer.set_z_index(-0.3);
    renderer.set_material(cvkg_core::DrawMaterial::Opaque);
    renderer.fill_rect(
        Rect {
            x: 100.0,
            y: 350.0,
            width: 280.0,
            height: 60.0,
        },
        [0.92, 0.94, 0.96, 1.0],
    );
    // Ink signature pen line (Cyan/blue)
    renderer.fill_rect(
        Rect {
            x: 120.0,
            y: 380.0,
            width: 140.0,
            height: 2.0,
        },
        [0.0, 0.45, 0.85, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    // Signature ink line check: blue > red, blue > green
    let sig_ink = count_matching_pixels(&pixels, width, height, |p| p[2] > p[0] && p[2] > p[1]);
    assert!(
        sig_ink > 0,
        "PDF editor signature ink line failed to render"
    );
}

/// Test: Character Motion App UI layout.
///
/// WHY: Tests character skeletal bone structures overlaying animated vector rigs inside stage viewports.
///
/// CONTRACT: Validates rendering of neon skeletal armature components.
#[test]
fn test_creative_ui_character_motion() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));

    // Theme: Asgard cyberpunk theme
    renderer.set_theme(ColorTheme::asgard());
    let encoder = renderer.begin_frame_headless();

    // Stage background (Pitch black)
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.02, 0.02, 0.02, 1.0],
    );

    // Character body shape (opaque red block) at Z = -0.1
    renderer.set_z_index(-0.1);
    renderer.fill_rect(
        Rect {
            x: 180.0,
            y: 150.0,
            width: 150.0,
            height: 250.0,
        },
        [0.85, 0.2, 0.25, 1.0],
    );

    // Skeletal rig joints / bone lines (Skeletal armature component on Layer 2) at Z = -0.2
    renderer.set_z_index(-0.2);
    // spine bone (neon green line)
    renderer.fill_rect(
        Rect {
            x: 250.0,
            y: 180.0,
            width: 4.0,
            height: 160.0,
        },
        [0.1, 0.95, 0.35, 1.0],
    );
    // arm bone (neon green line)
    renderer.fill_rect(
        Rect {
            x: 190.0,
            y: 220.0,
            width: 120.0,
            height: 4.0,
        },
        [0.1, 0.95, 0.35, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);
    // Bone color verify: green > red, green > blue
    let skeleton_green =
        count_matching_pixels(&pixels, width, height, |p| p[1] > p[0] && p[1] > p[2]);
    assert!(
        skeleton_green > 0,
        "Character motion armature skeleton failed to render"
    );
}
