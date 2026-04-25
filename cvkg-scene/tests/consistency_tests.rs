use cvkg_components::{Shape, Text};
use cvkg_core::{FrameRenderer, Rect, View};
use cvkg_scene::test_renderer::{Command, TestRenderer};

#[test]
fn test_primitive_consistency() {
    // Test Text rendering
    let text = Text::new("Consistency Test");
    let mut renderer = TestRenderer::new();
    let rect = Rect::new(10.0, 20.0, 100.0, 30.0);

    text.render(&mut renderer, rect);

    assert_eq!(renderer.commands.len(), 1);
    if let Command::DrawText {
        text, x, y, size, ..
    } = &renderer.commands[0]
    {
        assert_eq!(text, "Consistency Test");
        assert_eq!(*x, 10.0);
        assert_eq!(*y, 20.0);
        assert_eq!(*size, 14.0);
    } else {
        panic!("Expected DrawText command, got {:?}", renderer.commands[0]);
    }

    // Test Shape rendering
    let shape = Shape::rounded_rect(5.0);
    let mut renderer = TestRenderer::new();
    shape.render(&mut renderer, rect);

    assert_eq!(renderer.commands.len(), 1);
    if let Command::FillRoundedRect { radius, .. } = &renderer.commands[0] {
        assert_eq!(*radius, 5.0);
    } else {
        panic!(
            "Expected FillRoundedRect command, got {:?}",
            renderer.commands[0]
        );
    }
}

#[test]
fn test_scene_snapshot() {
    let mut renderer = TestRenderer::new();
    let rect = Rect::new(0.0, 0.0, 100.0, 100.0);

    renderer.begin_frame();
    Shape::rounded_rect(8.0).render(&mut renderer, rect);
    Text::new("Overlay").render(&mut renderer, rect);
    renderer.end_frame(());

    // Serialize commands to JSON for snapshot testing
    let commands_json = serde_json::to_string_pretty(&renderer.commands).unwrap();

    insta::assert_snapshot!("Scene Graph Commands", commands_json);
}
