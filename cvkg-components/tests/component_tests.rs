use cvkg_components::{Button, Text, VStack};
use cvkg_core::{ElapsedTime, Rect, Renderer, View};

struct MockRenderer {
    commands: Vec<String>,
}

impl MockRenderer {
    fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
}

impl ElapsedTime for MockRenderer {
    fn elapsed_time(&self) -> f32 {
        0.0
    }
    fn delta_time(&self) -> f32 {
        1.0 / 60.0
    }
}

impl Renderer for MockRenderer {
    fn fill_rect(&mut self, rect: Rect, _color: [f32; 4]) {
        self.commands.push(format!("FillRect({:?})", rect));
    }
    fn fill_rounded_rect(&mut self, rect: Rect, _radius: f32, _color: [f32; 4]) {
        self.commands.push(format!("FillRoundedRect({:?})", rect));
    }
    fn fill_ellipse(&mut self, _rect: Rect, _color: [f32; 4]) {}

    fn stroke_rect(&mut self, _rect: Rect, _color: [f32; 4], _width: f32) {}
    fn stroke_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: [f32; 4], _width: f32) {}
    fn stroke_ellipse(&mut self, _rect: Rect, _color: [f32; 4], _width: f32) {}
    fn draw_line(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _color: [f32; 4], _width: f32) {
    }

    fn fill_polygon(&mut self, vertices: &[[f32; 2]], _color: [f32; 4]) {
        self.commands
            .push(format!("FillPolygon(points: {})", vertices.len()));
    }
    fn stroke_polygon(&mut self, vertices: &[[f32; 2]], _color: [f32; 4], _width: f32) {
        self.commands
            .push(format!("StrokePolygon(points: {})", vertices.len()));
    }

    fn draw_text(&mut self, text: &str, _x: f32, _y: f32, _size: f32, _color: [f32; 4]) {
        self.commands.push(format!("DrawText({})", text));
    }
    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        (text.len() as f32 * size * 0.6, size)
    }

    fn push_vnode(&mut self, _rect: Rect, name: &'static str) {
        self.commands.push(format!("PushVNode({})", name));
    }
    fn pop_vnode(&mut self) {
        self.commands.push("PopVNode".to_string());
    }

    fn set_key(&mut self, _key: &str) {}
    fn set_aria_role(&mut self, _role: &str) {}
    fn set_aria_label(&mut self, _label: &str) {}
    fn memoize(&mut self, _id: u64, _data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer)) {
        render_fn(self);
    }
}

#[test]
fn test_button_rendering() {
    let mut renderer = MockRenderer::new();
    let button = Button::new("Submit", || {});
    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 40.0,
    };

    button.render(&mut renderer, rect);

    assert!(
        renderer
            .commands
            .iter()
            .any(|c| c.contains("PushVNode(Button)"))
    );
    assert!(
        renderer
            .commands
            .iter()
            .any(|c| c.contains("DrawText(Submit)"))
    );
    assert!(renderer.commands.contains(&"PopVNode".to_string()));
}

#[test]
fn test_vstack_rendering() {
    let mut renderer = MockRenderer::new();
    let vstack = VStack::new(10.0)
        .child(Text::new("Line 1"))
        .child(Text::new("Line 2"));
    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
    };

    vstack.render(&mut renderer, rect);

    assert!(
        renderer
            .commands
            .iter()
            .any(|c| c.contains("PushVNode(VStack)"))
    );
    assert!(
        renderer
            .commands
            .iter()
            .any(|c| c.contains("DrawText(Line 1)"))
    );
    assert!(
        renderer
            .commands
            .iter()
            .any(|c| c.contains("DrawText(Line 2)"))
    );
}

#[test]
fn test_hvergelmir_rendering() {
    let mut renderer = MockRenderer::new();
    let hex = cvkg_components::Hvergelmir::new(100.0);
    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };

    hex.render(&mut renderer, rect);

    assert!(
        renderer
            .commands
            .iter()
            .any(|c| c.contains("FillPolygon(points: 6)"))
    );
    assert!(
        renderer
            .commands
            .iter()
            .any(|c| c.contains("StrokePolygon(points: 6)"))
    );
}

#[test]
fn test_skjaldborg_rendering() {
    let mut renderer = MockRenderer::new();
    let shield = cvkg_components::Skjaldborg::new([1.0, 0.0, 0.0, 1.0]);
    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
    };

    shield.render(&mut renderer, rect);

    assert!(
        renderer
            .commands
            .iter()
            .any(|c| c.contains("FillPolygon(points: 4)"))
    );
}

#[test]
fn test_seiðr_rendering() {
    let mut renderer = MockRenderer::new();
    let effect = cvkg_components::Seiðr::default();
    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };

    effect.render(&mut renderer, rect);

    // Should have a rounded rect for background and some lines for scanlines
    assert!(
        renderer
            .commands
            .iter()
            .any(|c| c.contains("FillRoundedRect"))
    );
}

#[test]
fn test_lokiglitch_rendering() {
    let mut renderer = MockRenderer::new();
    let glitch = cvkg_components::LokiGlitch::new("ERROR");
    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };

    glitch.render(&mut renderer, rect);

    assert!(
        renderer
            .commands
            .iter()
            .any(|c| c.contains("DrawText(ERROR)"))
    );
}

#[test]
fn test_notification_system() {
    // 1. Post a notification using the global/default handler
    let handler = cvkg_core::get_notification_handler();
    let notif = cvkg_core::Notification {
        id: "test_notif_1".to_string(),
        app_name: Some("TestApp".to_string()),
        title: "Test Alert".to_string(),
        body: "Something happened".to_string(),
        priority: cvkg_core::NotificationPriority::Active,
        ..Default::default()
    };

    let res = handler.show(notif);
    assert!(res.is_ok());

    // 2. Verify that it was stored in system state
    let state = cvkg_core::load_system_state();
    let stored_notif = state.notifications.iter().find(|n| n.id == "test_notif_1");
    assert!(stored_notif.is_some());
    let stored_notif = stored_notif.unwrap();
    assert_eq!(stored_notif.title, "Test Alert");
    assert_eq!(stored_notif.body, "Something happened");

    // 3. Test ToastManager ingestion
    let mut toast_mgr = cvkg_components::toast::ToastManager::new();
    assert_eq!(toast_mgr.len(), 0);
    toast_mgr.update(0.0);
    assert_eq!(toast_mgr.len(), 1);
    assert_eq!(toast_mgr.toasts[0].title, "Test Alert");

    // 4. Set Notification Center visible and render
    cvkg_core::update_system_state(|st| {
        let mut new_st = st.clone();
        new_st.notification_center_visible = true;
        new_st
    });

    let mut renderer = MockRenderer::new();
    let panel = cvkg_components::notification_center::NotificationCenterPanel::new();
    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 1024.0,
        height: 768.0,
    };
    panel.render(&mut renderer, rect);

    // Verify NotificationCenterPanel pushed its node and drew texts
    assert!(
        renderer
            .commands
            .iter()
            .any(|c| c.contains("PushVNode(NotificationCenterPanel)"))
    );
    assert!(
        renderer
            .commands
            .iter()
            .any(|c| c.contains("DrawText(Notification Center)"))
    );
    assert!(
        renderer
            .commands
            .iter()
            .any(|c| c.contains("DrawText(Test Alert)"))
    );

    // 5. Dismiss and verify
    let dismiss_res = handler.dismiss("test_notif_1");
    assert!(dismiss_res.is_ok());
    let state_after = cvkg_core::load_system_state();
    assert!(
        state_after
            .notifications
            .iter()
            .find(|n| n.id == "test_notif_1")
            .unwrap()
            .dismissed
    );
}

#[test]
fn test_form_binder() {
    #[derive(Clone, Debug, PartialEq)]
    struct MyFormState {
        username: String,
        age: u32,
    }

    let initial_state = MyFormState {
        username: "vikings".to_string(),
        age: 30,
    };

    let mut binder = cvkg_components::FormBinder::new(initial_state);

    binder.add_rule("username", |state| {
        if state.username.len() >= 3 {
            Ok(())
        } else {
            Err("Username must be at least 3 characters".to_string())
        }
    });

    binder.add_rule("age", |state| {
        if state.age >= 18 {
            Ok(())
        } else {
            Err("Must be 18 or older".to_string())
        }
    });

    assert!(binder.validate());
    assert!(binder.is_valid());

    // Test binding updates
    let last_state = std::sync::Arc::new(std::sync::Mutex::new(binder.state.clone()));
    let last_state_clone = last_state.clone();

    let username_binding = binder.bind_field(
        |state| state.username.clone(),
        |state, val| state.username = val,
        move |new_state| {
            *last_state_clone.lock().unwrap() = new_state;
        },
    );

    assert_eq!(username_binding.get(), "vikings");
    username_binding.set("ulf".to_string());

    let updated_state = last_state.lock().unwrap().clone();
    assert_eq!(updated_state.username, "ulf");

    // Check validation failure
    binder.state = MyFormState {
        username: "ab".to_string(),
        age: 15,
    };

    assert!(!binder.validate());
    assert!(!binder.is_valid());
    assert_eq!(
        binder.error_for("username").unwrap(),
        "Username must be at least 3 characters"
    );
    assert_eq!(binder.error_for("age").unwrap(), "Must be 18 or older");
}

#[test]
fn test_phone_input_rendering() {
    let mut renderer = MockRenderer::new();
    let phone_input = cvkg_components::PhoneInput::new()
        .country_code("+44")
        .phone_number("123456789");
    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 300.0,
        height: 44.0,
    };
    phone_input.render(&mut renderer, rect);

    assert!(renderer.commands.iter().any(|c| c.contains("PushVNode(PhoneInput)")));
    assert!(renderer.commands.iter().any(|c| c.contains("DrawText(123456789)")));
}

#[test]
fn test_mention_input_rendering() {
    let mut renderer = MockRenderer::new();
    let mention_input = cvkg_components::MentionInput::new()
        .value("hello @");
    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 300.0,
        height: 44.0,
    };
    mention_input.render(&mut renderer, rect);

    assert!(renderer.commands.iter().any(|c| c.contains("PushVNode(MentionInput)")));
    assert!(renderer.commands.iter().any(|c| c.contains("PushVNode(MentionOption)")));
}

#[test]
fn test_editable_rendering() {
    let mut renderer = MockRenderer::new();
    let editable = cvkg_components::Editable::new("Initial Text");
    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 40.0,
    };
    editable.render(&mut renderer, rect);

    assert!(renderer.commands.iter().any(|c| c.contains("PushVNode(Editable)")));
    assert!(renderer.commands.iter().any(|c| c.contains("DrawText(Initial Text)")));
}

#[test]
fn test_popconfirm_rendering() {
    let mut renderer = MockRenderer::new();
    let button = Button::new("Delete", || {});
    let popconfirm = cvkg_components::Popconfirm::new(button, "Are you sure?");
    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 40.0,
    };
    popconfirm.render(&mut renderer, rect);

    assert!(renderer.commands.iter().any(|c| c.contains("PushVNode(Popconfirm)")));
    assert!(renderer.commands.iter().any(|c| c.contains("PushVNode(Button)")));
}

#[test]
fn test_qrcode_rendering() {
    let mut renderer = MockRenderer::new();
    let qr = cvkg_components::QRCode::new("https://rust-lang.org");
    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };
    qr.render(&mut renderer, rect);

    assert!(renderer.commands.iter().any(|c| c.contains("PushVNode(QRCode)")));
}
