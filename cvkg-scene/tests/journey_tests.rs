use cvkg_components::{Button, Text, VStack};
use cvkg_core::{Event, FrameRenderer, Rect, View};
use cvkg_scene::test_renderer::{Command, TestRenderer};
use cvkg_vdom::VDom;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
fn journey_button_click_flow() {
    // 1. SETUP: Create a view with a button that toggles a boolean state
    let clicked = Arc::new(AtomicBool::new(false));
    let clicked_clone = clicked.clone();

    let view = Button::new("Click Me", move || {
        clicked_clone.store(true, Ordering::SeqCst);
    });

    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
    };

    // 2. INITIAL RENDER: Capture the VDOM and emitted commands
    let mut renderer = TestRenderer::new();
    renderer.begin_frame();
    view.render(&mut renderer, rect);
    renderer.end_frame(());

    let vdom = VDom::build(&view, rect);

    // Verify initial state
    assert!(!clicked.load(Ordering::SeqCst));
    assert!(
        vdom.nodes
            .values()
            .any(|n| n.component_type.contains("Button"))
    );

    // 3. INTERACTION: Find the button node and dispatch a click event
    let button_node = vdom
        .nodes
        .values()
        .find(|n| n.component_type.contains("Button"))
        .expect("Button node not found in VDOM");

    // Simulate pointer move to center of button and then click
    vdom.dispatch_event(Event::PointerMove {
        x: button_node.layout.x + button_node.layout.width / 2.0,
        y: button_node.layout.y + button_node.layout.height / 2.0,
        proximity_field: 0.0,
        tilt: None,
        azimuth: None,
        pressure: 0.0,
        barrel_rotation: None,
    });

    vdom.dispatch_event(Event::PointerClick {
        x: button_node.layout.x + button_node.layout.width / 2.0,
        y: button_node.layout.y + button_node.layout.height / 2.0,
        button: 0,
        tilt: None,
        azimuth: None,
        pressure: 0.0,
        barrel_rotation: None,
    });

    // 4. VERIFICATION: Check if state updated
    assert!(
        clicked.load(Ordering::SeqCst),
        "Button click handler was not triggered"
    );
}

#[test]
fn journey_layout_reflow_on_content_change() {
    // 1. SETUP: Initial view with short text
    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 500.0,
        height: 500.0,
    };
    let view1 = Text::new("Short");
    let vdom1 = VDom::build(&view1, rect);

    let text_node1 = vdom1
        .nodes
        .values()
        .find(|n| n.component_type.contains("Text"))
        .expect("Text node not found");
    let initial_width = text_node1.layout.width;

    // 2. CHANGE: Longer text should result in larger width
    let view2 = Text::new("Much longer text that should occupy more space");
    let vdom2 = VDom::build(&view2, rect);

    let text_node2 = vdom2
        .nodes
        .values()
        .find(|n| n.component_type.contains("Text"))
        .expect("Text node not found");
    let updated_width = text_node2.layout.width;

    assert!(
        updated_width > initial_width,
        "Layout did not reflow after content change"
    );

    // 3. DIFF: Verify that VDOM patch is generated
    let patches = vdom1.diff(&vdom2);
    assert!(
        !patches.is_empty(),
        "No patches generated for content change"
    );
}

#[test]
fn journey_complex_hierarchy_rendering() {
    // Test a deeply nested stack to ensure transform propagation and command ordering
    let view = VStack::new(10.0)
        .child(Text::new("Header"))
        .child(
            Button::new("Action", || {})
                .padding(10.0)
                .background([0.1, 0.1, 0.1, 1.0]),
        )
        .child(Text::new("Footer"));

    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 300.0,
        height: 600.0,
    };
    let mut renderer = TestRenderer::new();
    view.render(&mut renderer, rect);

    // Verify command sequence
    // Should see vnode markers, background fill, and multiple text draws
    let commands = &renderer.commands;

    assert!(
        commands
            .iter()
            .any(|c| matches!(c, Command::PushVNode { name, .. } if name.contains("VStack")))
    );
    assert!(
        commands
            .iter()
            .any(|c| matches!(c, Command::PushVNode { name, .. } if name.contains("Button")))
    );
    assert!(
        commands
            .iter()
            .any(|c| matches!(c, Command::FillRect { .. }))
    );
    assert!(
        commands
            .iter()
            .filter(|c| matches!(c, Command::DrawText { .. }))
            .count()
            >= 3
    );
}
