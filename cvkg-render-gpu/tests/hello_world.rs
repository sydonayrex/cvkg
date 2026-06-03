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
    let mut non_zero_count = 0usize;
    for y in 0..height as usize {
        for x in 0..width as usize {
            let idx = (y * width as usize + x) * 4;
            let r = pixels[idx];
            let g = pixels[idx + 1];
            let b = pixels[idx + 2];
            if r > 0 || g > 0 || b > 0 {
                non_zero_count += 1;
            }
            if r > max_r {
                max_r = r;
            }
        }
    }

    println!(
        "Pixels: {}x{}, non_zero={}, max_r={}",
        width, height, non_zero_count, max_r
    );
    println!(
        "Telemetry: draw_calls={}, vertices={}",
        renderer.telemetry.draw_calls, renderer.telemetry.vertices
    );

    let corners = [
        (0usize, 0usize),
        (0usize, height as usize - 1),
        (width as usize - 1, 0),
        (width as usize - 1, height as usize - 1),
        (width as usize / 2, height as usize / 2),
    ];
    for (x, y) in corners {
        let idx = (y * width as usize + x) * 4;
        println!(
            "  Pixel ({},{}): R={}, G={}, B={}, A={}",
            x,
            y,
            pixels[idx],
            pixels[idx + 1],
            pixels[idx + 2],
            pixels[idx + 3]
        );
    }

    assert!(
        max_r > 50,
        "Max red component should be > 50, got {}",
        max_r
    );
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

    println!("WGPU roundtrip: wrote {:?}, read {:?}", data, read_data);
    assert_eq!(data, read_data, "Buffer roundtrip failed");
}
