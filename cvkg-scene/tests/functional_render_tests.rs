use cvkg_core::{Rect, Renderer, FrameRenderer};
use cvkg_scene::test_renderer::{TestRenderer, Command};

#[test]
fn test_primitive_rendering() {
    let mut renderer = TestRenderer::new();
    let rect = Rect { x: 10.0, y: 10.0, width: 100.0, height: 50.0 };
    let color = [1.0, 0.0, 0.0, 1.0];
    
    renderer.fill_rect(rect, color);
    renderer.stroke_rect(rect, color, 2.0);
    renderer.fill_rounded_rect(rect, 5.0, color);
    
    assert_eq!(renderer.commands.len(), 3);
    assert_eq!(renderer.commands[0], Command::FillRect { rect, color });
    assert_eq!(renderer.commands[1], Command::StrokeRect { rect, color, stroke_width: 2.0 });
    assert_eq!(renderer.commands[2], Command::FillRoundedRect { rect, radius: 5.0, color });
}

#[test]
fn test_effect_stacking() {
    let mut renderer = TestRenderer::new();
    let rect = Rect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 };
    
    renderer.push_clip_rect(rect);
    renderer.push_opacity(0.5);
    renderer.bifrost(rect, 10.0, 1.0, 1.0);
    renderer.pop_opacity();
    renderer.pop_clip_rect();
    
    assert_eq!(renderer.commands.len(), 5);
    assert_eq!(renderer.commands[0], Command::PushClipRect { rect });
    assert_eq!(renderer.commands[1], Command::PushOpacity { opacity: 0.5 });
    assert_eq!(renderer.commands[2], Command::Bifrost { rect, blur: 10.0, saturation: 1.0, opacity: 1.0 });
    assert_eq!(renderer.commands[3], Command::PopOpacity);
    assert_eq!(renderer.commands[4], Command::PopClipRect);
}

#[test]
fn test_transform_hierarchy() {
    let mut renderer = TestRenderer::new();
    
    renderer.push_transform([10.0, 20.0], [2.0, 2.0], 1.57);
    renderer.fill_rect(Rect { x: 0.0, y: 0.0, width: 10.0, height: 10.0 }, [1.0, 1.0, 1.0, 1.0]);
    renderer.pop_transform();
    
    assert_eq!(renderer.commands.len(), 3);
    if let Command::PushTransform { translation, scale, rotation } = renderer.commands[0] {
        assert_eq!(translation, [10.0, 20.0]);
        assert_eq!(scale, [2.0, 2.0]);
        assert_eq!(rotation, 1.57);
    } else {
        panic!("Expected PushTransform");
    }
}

#[test]
fn test_frame_lifecycle() {
    let mut renderer = TestRenderer::new();
    
    renderer.begin_frame();
    renderer.fill_rect(Rect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 }, [0.0, 0.0, 0.0, 1.0]);
    renderer.end_frame(());
    
    assert_eq!(renderer.commands[0], Command::BeginFrame);
    assert_eq!(renderer.commands[2], Command::EndFrame);
}
