//! # Cyberpunk Viking Berzerker Shader — Rust Integration
//!
//! SwiftUI-inspired architecture:
//! - [`ColorTheme`] is your `@Environment` / `@Binding` — swap at runtime to retheme.
//! - [`SceneUniforms`] is your `@State` / `@Published` — update every frame.
//! - [`ShaderPipeline`] is your `View` compositor — owns the wgpu resources.
//!
//! ## Quick start
//! ```rust
//! let theme    = ColorTheme::cyberpunk_viking();   // or any preset / custom
//! let pipeline = ShaderPipeline::new(&device, &surface_config, theme);
//! // each frame:
//! pipeline.update_scene(&queue, &scene);
//! pipeline.render(&mut encoder, &view);
//! ```

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;


// =============================================================================
// UNIFORM STRUCTS — must match WGSL layout exactly (std140 / 16-byte aligned)
// =============================================================================

/// Fully themeable color palette.  
/// Analogous to a SwiftUI `EnvironmentKey` — set once, consumed everywhere.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ColorTheme {
    pub primary_neon:        [f32; 4],   // (R, G, B, intensity)
    pub shatter_neon:        [f32; 4],
    pub glass_base:          [f32; 4],
    pub glass_edge:          [f32; 4],
    pub rune_glow:           [f32; 4],
    pub ember_core:          [f32; 4],
    pub background_deep:     [f32; 4],
    pub glass_blur_strength: f32,
    pub shatter_edge_width:  f32,
    pub neon_bloom_radius:   f32,
    pub rune_opacity:        f32,
    pub _pad:                [f32; 3],   // align to 16 bytes
    pub _pad2:               f32,
}

impl ColorTheme {
    // -------------------------------------------------------------------------
    // PRESET THEMES — swap these at runtime via `ShaderPipeline::set_theme()`
    // -------------------------------------------------------------------------

    /// Default: Obsidian black glass · Neon cyan lights · Magenta shatter
    pub fn cyberpunk_viking() -> Self {
        Self {
            primary_neon:        [0.0,  1.0,  0.95, 1.2],
            shatter_neon:        [1.0,  0.0,  0.75, 1.5],
            glass_base:          [0.04, 0.04, 0.06, 0.82],
            glass_edge:          [0.0,  0.45, 0.55, 0.6],
            rune_glow:           [0.75, 0.98, 1.0,  0.9],
            ember_core:          [0.95, 0.12, 0.12, 1.0],
            background_deep:     [0.01, 0.01, 0.03, 1.0],
            glass_blur_strength: 0.6,
            shatter_edge_width:  1.8,
            neon_bloom_radius:   0.022,
            rune_opacity:        0.55,
            _pad:                [0.0; 3],
            _pad2:               0.0,
        }
    }

    /// Jade Samurai: emerald neon · onyx glass · crimson shatter
    pub fn jade_samurai() -> Self {
        Self {
            primary_neon:        [0.0,  1.0,  0.3,  1.3],
            shatter_neon:        [1.0,  0.05, 0.05, 1.5],
            glass_base:          [0.02, 0.05, 0.03, 0.85],
            glass_edge:          [0.0,  0.55, 0.2,  0.5],
            rune_glow:           [0.6,  1.0,  0.6,  0.8],
            ember_core:          [1.0,  0.2,  0.0,  1.0],
            background_deep:     [0.01, 0.02, 0.01, 1.0],
            glass_blur_strength: 0.7,
            shatter_edge_width:  1.4,
            neon_bloom_radius:   0.018,
            rune_opacity:        0.45,
            _pad:                [0.0; 3],
            _pad2:               0.0,
        }
    }

    /// Solar Deity: amber neon · charcoal glass · electric blue shatter
    pub fn solar_deity() -> Self {
        Self {
            primary_neon:        [1.0,  0.75, 0.0,  1.4],
            shatter_neon:        [0.0,  0.5,  1.0,  1.6],
            glass_base:          [0.06, 0.04, 0.02, 0.80],
            glass_edge:          [0.6,  0.4,  0.0,  0.5],
            rune_glow:           [1.0,  0.9,  0.5,  0.85],
            ember_core:          [1.0,  0.4,  0.0,  1.0],
            background_deep:     [0.02, 0.01, 0.00, 1.0],
            glass_blur_strength: 0.5,
            shatter_edge_width:  2.0,
            neon_bloom_radius:   0.026,
            rune_opacity:        0.6,
            _pad:                [0.0; 3],
            _pad2:               0.0,
        }
    }

    /// Void Wraith: pure white neon · near-black glass · violet shatter
    pub fn void_wraith() -> Self {
        Self {
            primary_neon:        [0.9,  0.95, 1.0,  1.0],
            shatter_neon:        [0.65, 0.0,  1.0,  1.5],
            glass_base:          [0.03, 0.03, 0.04, 0.90],
            glass_edge:          [0.5,  0.5,  0.6,  0.4],
            rune_glow:           [0.9,  0.9,  1.0,  0.7],
            ember_core:          [0.7,  0.0,  1.0,  1.0],
            background_deep:     [0.0,  0.0,  0.02, 1.0],
            glass_blur_strength: 0.8,
            shatter_edge_width:  1.2,
            neon_bloom_radius:   0.020,
            rune_opacity:        0.35,
            _pad:                [0.0; 3],
            _pad2:               0.0,
        }
    }

    /// Construct a fully custom theme from individual RGBA values.
    /// Each `intensity` component (`.w`) multiplies that color's brightness.
    pub fn custom(
        primary_neon:    [f32; 4],
        shatter_neon:    [f32; 4],
        glass_base:      [f32; 4],
        glass_edge:      [f32; 4],
        rune_glow:       [f32; 4],
        ember_core:      [f32; 4],
        background_deep: [f32; 4],
    ) -> Self {
        Self {
            primary_neon,
            shatter_neon,
            glass_base,
            glass_edge,
            rune_glow,
            ember_core,
            background_deep,
            glass_blur_strength: 0.6,
            shatter_edge_width:  1.8,
            neon_bloom_radius:   0.022,
            rune_opacity:        0.55,
            _pad:                [0.0; 3],
            _pad2:               0.0,
        }
    }
}


/// Per-frame scene state.  
/// Analogous to `@State` + `@Published` in a SwiftUI ViewModel.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SceneUniforms {
    pub time:           f32,
    pub delta_time:     f32,
    pub resolution:     [f32; 2],
    pub mouse:          [f32; 2],
    pub mouse_velocity: [f32; 2],
    pub shatter_origin: [f32; 2],
    pub shatter_time:   f32,
    pub shatter_force:  f32,
    pub berzerker_rage: f32,
    pub scroll_offset:  f32,
    pub _pad:           [f32; 2],
}

impl SceneUniforms {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            time:           0.0,
            delta_time:     0.016,
            resolution:     [width as f32, height as f32],
            mouse:          [0.5, 0.5],
            mouse_velocity: [0.0, 0.0],
            shatter_origin: [0.5, 0.5],
            shatter_time:   -100.0,  // far in the past = no active shatter
            shatter_force:  0.0,
            berzerker_rage: 0.0,
            scroll_offset:  0.0,
            _pad:           [0.0; 2],
        }
    }

    /// Trigger a shatter event at normalized UV position `origin`.
    /// `force` is 0.0–1.0 (intensity of the break).
    pub fn trigger_shatter(&mut self, origin: [f32; 2], force: f32) {
        self.shatter_origin = origin;
        self.shatter_time   = self.time;
        self.shatter_force  = force.clamp(0.0, 1.0);
    }

    /// Set berzerker rage level (0.0 = calm, 1.0 = full fury).
    pub fn set_rage(&mut self, rage: f32) {
        self.berzerker_rage = rage.clamp(0.0, 1.0);
    }
}


// =============================================================================
// SHADER PIPELINE — wgpu resource ownership and render execution
// =============================================================================

pub struct ShaderPipeline {
    render_pipeline:    wgpu::RenderPipeline,
    theme_buffer:       wgpu::Buffer,
    scene_buffer:       wgpu::Buffer,
    bind_group:         wgpu::BindGroup,
    bind_group_layout:  wgpu::BindGroupLayout,
    current_theme:      ColorTheme,
}

impl ShaderPipeline {
    pub fn new(
        device:         &wgpu::Device,
        surface_format:  wgpu::TextureFormat,
        initial_theme:   ColorTheme,
        width:           u32,
        height:          u32,
    ) -> Self {
        // --- Shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label:  Some("cyberpunk_viking_berzerker"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shader.wgsl").into()
            ),
        });

        // --- Uniform buffers
        let theme_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("theme_uniforms"),
            contents: bytemuck::bytes_of(&initial_theme),
            usage:    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let initial_scene = SceneUniforms::new(width, height);
        let scene_buffer  = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("scene_uniforms"),
            contents: bytemuck::bytes_of(&initial_scene),
            usage:    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // --- Bind group layout (group 0: theme @ 0, scene @ 1)
        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label:   Some("shader_bgl"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding:    0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty:         wgpu::BindingType::Buffer {
                            ty:                 wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size:   None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding:    1,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty:         wgpu::BindingType::Buffer {
                            ty:                 wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size:   None,
                        },
                        count: None,
                    },
                ],
            }
        );

        // --- Bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label:   Some("shader_bg"),
            layout:  &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding:  0,
                    resource: theme_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding:  1,
                    resource: scene_buffer.as_entire_binding(),
                },
            ],
        });

        // --- Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label:                Some("shader_pipeline_layout"),
                bind_group_layouts:   &[&bind_group_layout],
                push_constant_ranges: &[],
            }
        );

        // --- Render pipeline (fullscreen quad, no vertex buffer)
        let render_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label:  Some("cyberpunk_viking_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module:      &shader,
                    entry_point: Some("vs_main"),
                    buffers:     &[],          // vertex_index only
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module:      &shader,
                    entry_point: Some("fs_main"),
                    targets:     &[Some(wgpu::ColorTargetState {
                        format:     surface_format,
                        blend:      Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive:    wgpu::PrimitiveState {
                    topology:          wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face:        wgpu::FrontFace::Ccw,
                    cull_mode:         None,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample:   wgpu::MultisampleState::default(),
                multiview:     None,
                cache:         None,
            }
        );

        Self {
            render_pipeline,
            theme_buffer,
            scene_buffer,
            bind_group,
            bind_group_layout,
            current_theme: initial_theme,
        }
    }

    // -------------------------------------------------------------------------
    // RUNTIME UPDATES — SwiftUI analogy: state mutation triggers re-render
    // -------------------------------------------------------------------------

    /// Hot-swap the entire color theme (e.g. user picks a preset in the UI).
    pub fn set_theme(&mut self, queue: &wgpu::Queue, theme: ColorTheme) {
        self.current_theme = theme;
        queue.write_buffer(&self.theme_buffer, 0, bytemuck::bytes_of(&theme));
    }

    /// Mutate a single theme color without rebuilding the whole struct.
    pub fn set_theme_color(
        &mut self,
        queue:  &wgpu::Queue,
        field:  ThemeField,
        color:  [f32; 4],
    ) {
        let offset = field.byte_offset();
        queue.write_buffer(
            &self.theme_buffer,
            offset as u64,
            bytemuck::cast_slice(&color),
        );
    }

    /// Upload the current frame's scene state (call once per frame).
    pub fn update_scene(&self, queue: &wgpu::Queue, scene: &SceneUniforms) {
        queue.write_buffer(&self.scene_buffer, 0, bytemuck::bytes_of(scene));
    }

    /// Record the draw call into an existing command encoder.
    /// Draws 6 vertices (fullscreen quad) — no vertex buffer needed.
    pub fn render(
        &self,
        encoder:     &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("cyberpunk_viking_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view:           target_view,
                resolve_target: None,
                ops:            wgpu::Operations {
                    load:  wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes:         None,
            occlusion_query_set:      None,
        });

        pass.set_pipeline(&self.render_pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..6, 0..1);   // 6 vertices = 2 triangles = fullscreen quad
    }
}


// =============================================================================
// THEME FIELD ENUM — for granular runtime color mutation
// =============================================================================

/// Identifies a specific color slot in [`ColorTheme`] for partial updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeField {
    PrimaryNeon,
    ShatterNeon,
    GlassBase,
    GlassEdge,
    RuneGlow,
    EmberCore,
    BackgroundDeep,
}

impl ThemeField {
    /// Byte offset within the [`ColorTheme`] uniform buffer.
    pub fn byte_offset(self) -> usize {
        match self {
            Self::PrimaryNeon    =>  0,
            Self::ShatterNeon    => 16,
            Self::GlassBase      => 32,
            Self::GlassEdge      => 48,
            Self::RuneGlow       => 64,
            Self::EmberCore      => 80,
            Self::BackgroundDeep => 96,
        }
    }
}


// =============================================================================
// EXAMPLE USAGE (main.rs / app loop)
// =============================================================================
//
// ```rust
// use winit::event::*;
// use std::time::Instant;
//
// fn main() {
//     // ... wgpu init, window creation omitted for brevity ...
//
//     let mut theme    = ColorTheme::cyberpunk_viking();
//     let mut pipeline = ShaderPipeline::new(&device, surface_format, theme, 1280, 720);
//     let mut scene    = SceneUniforms::new(1280, 720);
//     let     start    = Instant::now();
//     let mut prev     = Instant::now();
//
//     event_loop.run(move |event, _, control_flow| {
//         match event {
//             Event::MainEventsCleared => {
//                 let now  = Instant::now();
//                 scene.time       = start.elapsed().as_secs_f32();
//                 scene.delta_time = (now - prev).as_secs_f32();
//                 prev = now;
//
//                 // Example: ramp berzerker rage with the spacebar
//                 // scene.set_rage(scene.berzerker_rage + 0.01);
//
//                 pipeline.update_scene(&queue, &scene);
//
//                 let output  = surface.get_current_texture().unwrap();
//                 let view    = output.texture.create_view(&Default::default());
//                 let mut enc = device.create_command_encoder(&Default::default());
//                 pipeline.render(&mut enc, &view);
//                 queue.submit([enc.finish()]);
//                 output.present();
//             }
//             Event::WindowEvent { event: WindowEvent::MouseInput {
//                 state: ElementState::Pressed, button: MouseButton::Left, ..
//             }, .. } => {
//                 // Shatter on click at current mouse position
//                 scene.trigger_shatter(scene.mouse, 0.85);
//             }
//             Event::WindowEvent { event: WindowEvent::KeyboardInput {
//                 input: KeyboardInput {
//                     virtual_keycode: Some(VirtualKeyCode::Key1), ..
//                 }, ..
//             }, .. } => {
//                 // Swap to preset 1
//                 pipeline.set_theme(&queue, ColorTheme::cyberpunk_viking());
//             }
//             Event::WindowEvent { event: WindowEvent::KeyboardInput {
//                 input: KeyboardInput {
//                     virtual_keycode: Some(VirtualKeyCode::Key2), ..
//                 }, ..
//             }, .. } => {
//                 // Swap to preset 2
//                 pipeline.set_theme(&queue, ColorTheme::jade_samurai());
//             }
//             _ => {}
//         }
//     });
// }
// ```
