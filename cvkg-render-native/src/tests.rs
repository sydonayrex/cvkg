#[cfg(test)]
mod tests {
    use crate::asset_manager::NativeAssetManager;
    use crate::events::{convert_ime_event, convert_mouse_event};
    use cvkg_core::AssetManager;
    use cvkg_vdom::{AriaProps, LayoutRect, VDom, VNode};
    use std::collections::HashMap;
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    fn interactive_node(
        id: u64,
        component_type: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        aria_role: &str,
    ) -> VNode {
        VNode {
            id: cvkg_core::KvasirId(id),
            key: None,
            component_type: component_type.to_string(),
            props: HashMap::new(),
            state: None,
            layout: LayoutRect {
                x,
                y,
                width,
                height,
            },
            children: Vec::new(),
            aria_role: aria_role.to_string(),
            aria_props: AriaProps::default(),
            portal_target: None,
            sdf_shape: Some(cvkg_core::layout::SdfShape::Rect(cvkg_core::Rect {
                x,
                y,
                width,
                height,
            })),
        }
    }

    fn route_pointer_sequence_through_native_capture(
        pressed_vdom: &VDom,
        rebuilt_vdom: &VDom,
        x: f32,
        y: f32,
        button: u32,
    ) -> (
        cvkg_core::EventResponse,
        cvkg_core::EventResponse,
        cvkg_core::EventResponse,
    ) {
        let active_target = pressed_vdom.hit_test(x, y, 0.0).map(|(id, _)| id);
        let mut applied_vdom = VDom::new();
        applied_vdom.root = pressed_vdom.root;
        applied_vdom.nodes = pressed_vdom.nodes.clone();
        applied_vdom.parents = pressed_vdom.parents.clone();
        applied_vdom.event_handlers = pressed_vdom.event_handlers.clone();
        let down = active_target
            .map(|target| {
                applied_vdom.dispatch_event_to_target(
                    target,
                    cvkg_core::Event::PointerDown {
                        x,
                        y,
                        button,
                        proximity_field: 0.0,
                        tilt: None,
                        azimuth: None,
                        pressure: Some(1.0),
                        barrel_rotation: None,
                        pointer_precision: 0.0,
                    },
                )
            })
            .unwrap_or_else(|| {
                applied_vdom.dispatch_event(cvkg_core::Event::PointerDown {
                    x,
                    y,
                    button,
                    proximity_field: 0.0,
                    tilt: None,
                    azimuth: None,
                    pressure: Some(1.0),
                    barrel_rotation: None,
                    pointer_precision: 0.0,
                })
            });

        applied_vdom.apply_patches(pressed_vdom.diff(rebuilt_vdom));

        let fallback_target = applied_vdom.hit_test(x, y, 0.0).map(|(id, _)| id);
        let resolved_target = active_target
            .filter(|target| applied_vdom.nodes.contains_key(target))
            .or(fallback_target);

        let pointer_up = cvkg_core::Event::PointerUp {
            x,
            y,
            button,
            tilt: None,
            azimuth: None,
            pressure: Some(0.0),
            barrel_rotation: None,
            pointer_precision: 0.0,
        };
        let pointer_click = cvkg_core::Event::PointerClick {
            x,
            y,
            button,
            tilt: None,
            azimuth: None,
            pressure: Some(0.0),
            barrel_rotation: None,
            pointer_precision: 0.0,
        };

        let up = resolved_target
            .map(|target| applied_vdom.dispatch_event_to_target(target, pointer_up.clone()))
            .unwrap_or_else(|| applied_vdom.dispatch_event(pointer_up));
        let click = resolved_target
            .map(|target| applied_vdom.dispatch_event_to_target(target, pointer_click.clone()))
            .unwrap_or_else(|| applied_vdom.dispatch_event(pointer_click));

        (down, up, click)
    }

    #[test]
    fn test_native_asset_manager_loading() {
        let manager = NativeAssetManager::new();
        let temp_path = std::env::temp_dir().join("cvkg_test_asset_loading.png");
        let temp_file_path = temp_path
            .to_str()
            .expect("temp path contains invalid UTF-8");
        let test_data = b"fake-image-data";

        let mut file = std::fs::File::create(temp_file_path).unwrap();
        file.write_all(test_data).unwrap();
        drop(file);

        let mut state = manager.load_image(temp_file_path);

        let start = std::time::Instant::now();
        while matches!(state, cvkg_core::AssetState::Loading) && start.elapsed().as_secs() < 5 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            state = manager.load_image(temp_file_path);
        }

        if let cvkg_core::AssetState::Ready(data) = state {
            assert_eq!(&*data, test_data);
        } else {
            let _ = std::fs::remove_file(temp_file_path);
            panic!("Expected Ready state, got {:?}", state);
        }

        let state2 = manager.load_image(temp_file_path);
        if let cvkg_core::AssetState::Ready(data) = state2 {
            assert_eq!(&*data, test_data);
        } else {
            let _ = std::fs::remove_file(temp_file_path);
            panic!("Expected Ready state (cached), got {:?}", state2);
        }

        let _ = std::fs::remove_file(temp_file_path);
    }

    #[test]
    fn test_native_asset_manager_error() {
        let manager = NativeAssetManager::new();
        let path = "non_existent_file_cvkg_test.png";
        let mut state = manager.load_image(path);

        let start = std::time::Instant::now();
        while matches!(state, cvkg_core::AssetState::Loading) && start.elapsed().as_secs() < 5 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            state = manager.load_image(path);
        }

        if let cvkg_core::AssetState::Error(_) = state {
        } else {
            panic!("Expected Error state, got {:?}", state);
        }
    }

    #[test]
    fn test_event_conversion() {
        let event = convert_mouse_event(winit::event::ElementState::Pressed, [10.0, 20.0], 0);
        if let cvkg_core::Event::PointerDown { x, y, button, .. } = event {
            assert_eq!(x, 10.0);
            assert_eq!(y, 20.0);
            assert_eq!(button, 0);
        } else {
            panic!("Expected PointerDown");
        }

        let event = convert_ime_event(winit::event::Ime::Commit("hello".to_string()));
        if let Some(cvkg_core::Event::Ime(s)) = event {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected Ime event");
        }
    }

    #[test]
    fn native_pointer_capture_survives_rebuild_sequence() {
        let fired = Arc::new(Mutex::new(Vec::<&'static str>::new()));

        let mut pressed = VDom::new();
        let root_id = cvkg_core::KvasirId(1);
        let button_id = cvkg_core::KvasirId(2);
        let mut root = interactive_node(1, "Root", 0.0, 0.0, 240.0, 240.0, "group");
        root.children = vec![button_id];
        let button = interactive_node(2, "Button", 20.0, 20.0, 80.0, 40.0, "button");

        let fired_down = Arc::clone(&fired);
        let fired_up = Arc::clone(&fired);
        let fired_click = Arc::clone(&fired);
        pressed.event_handlers.insert(
            button_id,
            vec![
                (
                    "pointerdown".to_string(),
                    Arc::new(move |_| {
                        fired_down.lock().unwrap().push("down");
                    }) as _,
                ),
                (
                    "pointerup".to_string(),
                    Arc::new(move |_| {
                        fired_up.lock().unwrap().push("up");
                    }) as _,
                ),
                (
                    "pointerclick".to_string(),
                    Arc::new(move |_| {
                        fired_click.lock().unwrap().push("click");
                    }) as _,
                ),
            ]
            .into_iter()
            .collect(),
        );
        pressed.root = Some(root_id);
        pressed.nodes.insert(root_id, root);
        pressed.nodes.insert(button_id, button);
        pressed.parents.insert(button_id, root_id);

        let mut rebuilt = VDom::new();
        let mut rebuilt_root = interactive_node(1, "Root", 0.0, 0.0, 240.0, 240.0, "group");
        rebuilt_root.children = vec![button_id];
        let rebuilt_button = interactive_node(2, "Button", 20.0, 20.0, 80.0, 40.0, "button");
        rebuilt.event_handlers = pressed.event_handlers.clone();
        rebuilt.root = Some(root_id);
        rebuilt.nodes.insert(root_id, rebuilt_root);
        rebuilt.nodes.insert(button_id, rebuilt_button);
        rebuilt.parents.insert(button_id, root_id);

        let (down, up, click) =
            route_pointer_sequence_through_native_capture(&pressed, &rebuilt, 30.0, 30.0, 0);

        assert_eq!(down, cvkg_core::EventResponse::Handled);
        assert_eq!(up, cvkg_core::EventResponse::Handled);
        assert_eq!(click, cvkg_core::EventResponse::Handled);
        assert_eq!(*fired.lock().unwrap(), vec!["down", "up", "click"]);
    }

    #[test]
    fn native_pointer_capture_falls_back_to_rebuilt_target() {
        let fired = Arc::new(Mutex::new(Vec::<&'static str>::new()));

        let mut pressed = VDom::new();
        let root_id = cvkg_core::KvasirId(1);
        let old_button_id = cvkg_core::KvasirId(2);
        let mut root = interactive_node(1, "Root", 0.0, 0.0, 240.0, 240.0, "group");
        root.children = vec![old_button_id];
        let button = interactive_node(2, "Button", 20.0, 20.0, 80.0, 40.0, "button");

        let fired_down = Arc::clone(&fired);
        let fired_up = Arc::clone(&fired);
        let fired_click = Arc::clone(&fired);
        pressed.event_handlers.insert(
            old_button_id,
            vec![
                (
                    "pointerdown".to_string(),
                    Arc::new(move |_| {
                        fired_down.lock().unwrap().push("down");
                    }) as _,
                ),
                (
                    "pointerup".to_string(),
                    Arc::new(move |_| {
                        fired_up.lock().unwrap().push("up");
                    }) as _,
                ),
                (
                    "pointerclick".to_string(),
                    Arc::new(move |_| {
                        fired_click.lock().unwrap().push("click");
                    }) as _,
                ),
            ]
            .into_iter()
            .collect(),
        );
        pressed.root = Some(root_id);
        pressed.nodes.insert(root_id, root);
        pressed.nodes.insert(old_button_id, button);
        pressed.parents.insert(old_button_id, root_id);

        let mut rebuilt = VDom::new();
        let mut rebuilt_root = interactive_node(1, "Root", 0.0, 0.0, 240.0, 240.0, "group");
        let rebuilt_button_id = cvkg_core::KvasirId(3);
        rebuilt_root.children = vec![rebuilt_button_id];
        let rebuilt_button = interactive_node(3, "Button", 20.0, 20.0, 80.0, 40.0, "button");
        rebuilt.event_handlers = pressed.event_handlers.clone();
        rebuilt.root = Some(root_id);
        rebuilt.nodes.insert(root_id, rebuilt_root);
        rebuilt.nodes.insert(rebuilt_button_id, rebuilt_button);
        rebuilt.parents.insert(rebuilt_button_id, root_id);

        let (down, up, click) =
            route_pointer_sequence_through_native_capture(&pressed, &rebuilt, 30.0, 30.0, 0);

        assert_eq!(down, cvkg_core::EventResponse::Handled);
        assert_eq!(up, cvkg_core::EventResponse::Handled);
        assert_eq!(click, cvkg_core::EventResponse::Handled);
        assert_eq!(*fired.lock().unwrap(), vec!["down", "up", "click"]);
    }
}

#[cfg(test)]
mod p1_46_47_49_51_tests {
    use crate::contracts::{
        RenderingMode, SemanticRoleRegistry, StateSyncRegistry, SyncDirection,
        TranslationContractRegistry, WidgetVirtualizationConfig,
    };
    use crate::regression::VisualRegressionTracker;
    use crate::window::{MonitorConfig, MultiMonitorManager};

    #[test]
    fn test_multi_monitor_manager_basics() {
        let m1 = MonitorConfig {
            name: "Display 1".to_string(),
            position: (0, 0),
            size: (1920, 1080),
            scale_factor: 1.0,
            refresh_rate: 60,
        };
        let m2 = MonitorConfig {
            name: "Display 2".to_string(),
            position: (1920, 0),
            size: (3840, 2160),
            scale_factor: 2.0,
            refresh_rate: 120,
        };

        let mut manager = MultiMonitorManager::new(vec![m1, m2]);
        assert_eq!(manager.monitors().len(), 2);
        assert_eq!(manager.current_monitor().name, "Display 1");

        let scaled = manager.scale_dimensions(100.0, 200.0);
        assert_eq!(scaled, (100, 200));

        let idx = manager.update_window_position((1920 + 100, 100, 1000, 1000));
        assert_eq!(idx, Some(1));
        assert_eq!(manager.current_monitor().name, "Display 2");

        let scaled_m2 = manager.scale_dimensions(100.0, 200.0);
        assert_eq!(scaled_m2, (200, 400));

        assert!(manager.requires_dpi_adaptation(0, 1));
        assert!(!manager.requires_dpi_adaptation(0, 0));
    }

    #[test]
    fn test_visual_regression_tracker_comparison() {
        use image::{ImageFormat, RgbaImage};
        use std::io::Cursor;

        let mut img1 = RgbaImage::new(10, 10);
        for p in img1.pixels_mut() {
            *p = image::Rgba([255, 0, 0, 255]);
        }
        let mut png1 = Vec::new();
        img1.write_to(&mut Cursor::new(&mut png1), ImageFormat::Png)
            .unwrap();

        let temp_dir = std::env::temp_dir().join("cvkg_visual_regression_tests");
        let tracker = VisualRegressionTracker::new(temp_dir.clone(), 5, 1.0);

        let matched = tracker.verify_frame("test_red_rect", &png1);
        assert!(matched);

        let matched_again = tracker.verify_frame("test_red_rect", &png1);
        assert!(matched_again);

        let mut img2 = RgbaImage::new(10, 10);
        for (i, p) in img2.pixels_mut().enumerate() {
            if i == 0 {
                *p = image::Rgba([253, 0, 0, 255]);
            } else {
                *p = image::Rgba([255, 0, 0, 255]);
            }
        }
        let mut png2 = Vec::new();
        img2.write_to(&mut Cursor::new(&mut png2), ImageFormat::Png)
            .unwrap();

        let matched_tolerated = tracker.verify_frame("test_red_rect", &png2);
        assert!(matched_tolerated);

        let mut img3 = RgbaImage::new(10, 10);
        for p in img3.pixels_mut() {
            *p = image::Rgba([0, 255, 0, 255]);
        }
        let mut png3 = Vec::new();
        img3.write_to(&mut Cursor::new(&mut png3), ImageFormat::Png)
            .unwrap();

        let matched_fail = tracker.verify_frame("test_red_rect", &png3);
        assert!(!matched_fail);

        let _ = std::fs::remove_file(temp_dir.join("test_red_rect.png"));
    }

    #[test]
    fn translation_contract_registry_has_defaults() {
        let reg = TranslationContractRegistry::new();
        assert!(reg.find("Button").is_some());
        assert!(reg.find("Canvas").is_some());
        assert!(reg.find("Unknown").is_none());
    }

    #[test]
    fn button_uses_native_rendering() {
        let reg = TranslationContractRegistry::new();
        let contract = reg.find("Button").unwrap();
        assert_eq!(contract.rendering_mode, RenderingMode::Native);
        assert!(contract.native_accessibility);
    }

    #[test]
    fn canvas_uses_custom_rendering() {
        let reg = TranslationContractRegistry::new();
        let contract = reg.find("Canvas").unwrap();
        assert_eq!(contract.rendering_mode, RenderingMode::Custom);
    }

    #[test]
    fn window_capability_matrix_has_platform() {
        let matrix = crate::window::WindowCapabilityMatrix::for_current_platform();
        assert!(!matrix.platform.is_empty());
        assert!(!matrix.window_types.is_empty());
    }

    #[test]
    fn macos_has_sheets() {
        #[cfg(target_os = "macos")]
        {
            let matrix = crate::window::WindowCapabilityMatrix::for_current_platform();
            assert!(matrix.sheets);
            assert!(matrix.tabbed_windows);
        }
    }

    #[test]
    fn state_sync_registry_has_defaults() {
        let reg = StateSyncRegistry::new();
        assert!(reg.find("Button").is_some());
        assert!(reg.find("TextInput").is_some());
    }

    #[test]
    fn text_input_has_debounce() {
        let reg = StateSyncRegistry::new();
        let contract = reg.find("TextInput").unwrap();
        assert!(contract.debounce);
        assert_eq!(contract.debounce_ms, 50);
    }

    #[test]
    fn button_is_bidirectional() {
        let reg = StateSyncRegistry::new();
        let contract = reg.find("Button").unwrap();
        assert_eq!(contract.direction, SyncDirection::Bidirectional);
    }

    #[test]
    fn default_virtualization_config() {
        let config = WidgetVirtualizationConfig::default();
        assert_eq!(config.buffer_size, 5);
        assert!(config.recycle_handles);
        assert_eq!(config.max_active_handles, 100);
    }

    #[test]
    fn semantic_role_registry_has_button_and_text() {
        let reg = SemanticRoleRegistry::new();
        let button = reg.find(accesskit::Role::Button).unwrap();
        assert_eq!(button.mac_ax_role, "AXButton");
        assert_eq!(button.win_uia_control_type, "UIA_ButtonControlTypeId");
        assert_eq!(button.linux_atk_role, "ATK_ROLE_PUSH_BUTTON");

        let text = reg.find(accesskit::Role::TextInput).unwrap();
        assert_eq!(text.mac_ax_role, "AXTextField");
    }

    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn mutex_poison_recovery_via_unwrap_or_else() {
        let mutex = Arc::new(Mutex::new(42u32));
        let mutex_clone = Arc::clone(&mutex);

        let handle = thread::spawn(move || {
            let _guard = mutex_clone.lock().unwrap();
            panic!("simulated thread panic while holding lock");
        });

        let _ = handle.join();

        let value = mutex.lock().unwrap_or_else(|p| p.into_inner());
        assert_eq!(
            *value, 42,
            "poisoned mutex should still yield the inner value"
        );
    }

    #[test]
    fn mutex_poison_recovery_multiple_times() {
        let mutex = Arc::new(Mutex::new(String::from("hello")));

        for i in 0..5 {
            let m = Arc::clone(&mutex);
            let handle = thread::spawn(move || {
                let _guard = m.lock().unwrap();
                panic!("panic iteration {}", i);
            });
            let _ = handle.join();
        }

        let value = mutex.lock().unwrap_or_else(|p| p.into_inner());
        assert_eq!(*value, "hello");
    }

    struct RendererState {
        frame_count: u32,
    }

    #[test]
    fn gpu_mutex_poison_pattern() {
        let gpu = Arc::new(Mutex::new(RendererState { frame_count: 0 }));
        let gpu_clone = Arc::clone(&gpu);

        let handle = thread::spawn(move || {
            let mut state = gpu_clone.lock().unwrap();
            state.frame_count += 1;
            panic!("GPU render panic");
        });

        let _ = handle.join();

        let mut state = gpu.lock().unwrap_or_else(|p| p.into_inner());
        assert_eq!(state.frame_count, 1);
        state.frame_count += 1;
        assert_eq!(state.frame_count, 2);
    }

    #[test]
    fn poison_recovery_preserves_data_integrity() {
        let data = Arc::new(Mutex::new(vec![1, 2, 3, 4, 5]));
        let data_clone = Arc::clone(&data);

        let handle = thread::spawn(move || {
            let mut guard = data_clone.lock().unwrap();
            guard.push(6);
            panic!("mid-mutation panic");
        });

        let _ = handle.join();

        let recovered = data.lock().unwrap_or_else(|p| p.into_inner());
        assert!(recovered.len() >= 5);
        assert_eq!(&recovered[..5], &[1, 2, 3, 4, 5]);
    }
}
