/// Minimal headless rendering test — draws a red fullscreen quad.
#[test]
fn test_headless_minimal_render() {
    use cvkg_core::{FrameRenderer, Rect, Renderer};
    use cvkg_render_gpu::SurtrRenderer;

    let width: u32 = 64;
    let height: u32 = 64;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));

    let encoder = renderer.begin_frame_headless();

    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [1.0, 0.0, 0.0, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = pollster::block_on(renderer.capture_frame()).expect("Failed to capture frame");

    let mut max_r = 0u8;
    for y in 0..height as usize {
        for x in 0..width as usize {
            let idx = (y * width as usize + x) * 4;
            if pixels[idx] > max_r {
                max_r = pixels[idx];
            }
        }
    }

    println!(
        "Minimal render: max_r={}, draw_calls={}, vertices={}",
        max_r, renderer.telemetry.draw_calls, renderer.telemetry.vertices
    );

    assert!(
        max_r > 50,
        "Max red component should be > 50, got {}",
        max_r
    );
}

/// Test that draws directly to the output texture using a simple pipeline.
/// Bypasses the multi-pass pipeline entirely.
#[test]
fn test_headless_direct_draw() {
    use wgpu::util::DeviceExt;

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    }))
    .expect("No adapter");
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("No device");

    let width: u32 = 64;
    let height: u32 = 64;
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;

    let output_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("output"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("red"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
            r#"
            @vertex
            fn vs_main(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
                var pos = array<vec2<f32>, 3>(
                    vec2<f32>(-1.0, -1.0),
                    vec2<f32>( 3.0, -1.0),
                    vec2<f32>(-1.0,  3.0)
                );
                return vec4<f32>(pos[vi], 0.0, 1.0);
            }
            @fragment
            fn fs_main() -> @location(0) vec4<f32> {
                return vec4<f32>(1.0, 0.0, 0.0, 1.0);
            }
        "#,
        )),
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("red_pipeline"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("red_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            ..Default::default()
        });
        pass.set_pipeline(&pipeline);
        pass.draw(0..3, 0..1);
    }

    let u32_size = std::mem::size_of::<u32>() as u32;
    let bytes_per_row = width * u32_size;
    let padding = (256 - (bytes_per_row % 256)) % 256;
    let padded_bytes_per_row = bytes_per_row + padding;

    let read_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("read"),
        size: (padded_bytes_per_row as u64 * height as u64),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &output_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &read_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    );

    queue.submit(Some(encoder.finish()));

    let buffer_slice = read_buffer.slice(..);
    let (sender, receiver) = futures::channel::oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
        let _ = sender.send(v);
    });
    device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });

    let result = pollster::block_on(receiver).unwrap();
    assert!(result.is_ok());

    let mapped = buffer_slice.get_mapped_range();
    let data: Vec<u8> = mapped.to_vec();
    drop(mapped);
    read_buffer.unmap();

    let cx = (width / 2) as usize;
    let cy = (height / 2) as usize;
    let idx = (cy * width as usize + cx) * 4;
    let r = data[idx];

    println!("Direct draw center pixel: R={}", r);
    assert!(r > 200, "Direct draw should produce red pixel, got R={}", r);
}

/// Test that basic wgpu buffer write + readback works.
#[test]
fn test_wgpu_buffer_roundtrip() {
    use wgpu::util::DeviceExt;

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    }))
    .expect("No adapter");
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("No device");

    let data: Vec<u8> = vec![255, 0, 0, 255, 0, 255, 0, 255];
    let write_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("write"),
        contents: &data,
        usage: wgpu::BufferUsages::COPY_SRC,
    });

    let read_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("read"),
        size: data.len() as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("copy"),
    });
    encoder.copy_buffer_to_buffer(&write_buffer, 0, &read_buffer, 0, data.len() as u64);
    queue.submit(Some(encoder.finish()));

    let buffer_slice = read_buffer.slice(..);
    let (sender, receiver) = futures::channel::oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
        let _ = sender.send(v);
    });
    device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });

    let result = pollster::block_on(receiver).unwrap();
    assert!(result.is_ok());

    let mapped = buffer_slice.get_mapped_range();
    let read_data: Vec<u8> = mapped.to_vec();
    drop(mapped);
    read_buffer.unmap();

    assert_eq!(data, read_data, "Buffer roundtrip failed");
}
