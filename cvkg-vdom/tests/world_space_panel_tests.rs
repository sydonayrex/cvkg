use cvkg_core::KvasirId;
use cvkg_core::mesh::Transform3D;
use cvkg_materials::GlassMaterial;
use cvkg_vdom::{AriaProps, LayoutRect, ResolvedPosition, VDom, VNode, WorldSpacePanel};
use glam::{Quat, Vec3};
use std::collections::HashMap;

#[test]
fn test_default_panel() {
    let p = WorldSpacePanel::default();
    assert_eq!(p.transform, Transform3D::default());
    assert_eq!(p.world_size, (1.0, 1.0));
    assert_eq!(p.pixels_per_unit, 200.0);
    assert!(p.glass.is_none());
}

#[test]
fn test_texture_resolution_at_200ppu() {
    let p = WorldSpacePanel {
        world_size: (1.0, 0.5),
        pixels_per_unit: 200.0,
        ..Default::default()
    };
    let (w, h) = p.texture_resolution();
    assert_eq!(w, 200);
    assert_eq!(h, 100);
}

#[test]
fn test_vnode_carries_world_space() {
    let panel = WorldSpacePanel::default();
    let v = VNode {
        id: KvasirId::new(),
        key: None,
        component_type: "div".to_string(),
        props: Default::default(),
        state: None,
        layout: Default::default(),
        children: Default::default(),
        aria_role: "div".to_string(),
        aria_props: Default::default(),
        portal_target: None,
        world_space: Some(panel.clone()),
        theme_override: None,
        color_palette: u16::MAX,
        sdf_shape: None,
        companions: HashMap::new(),
    };
    assert!(v.world_space.is_some());
    assert_eq!(v.world_space.as_ref().unwrap(), &panel);
}

#[test]
fn test_world_space_panel_with_glass() {
    let panel = WorldSpacePanel {
        glass: Some(GlassMaterial::default()),
        ..Default::default()
    };
    assert!(panel.glass.is_some());
}

#[test]
fn test_world_space_panel_transform() {
    let panel = WorldSpacePanel {
        transform: Transform3D {
            position: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        ..Default::default()
    };
    assert_eq!(panel.transform.position.x, 1.0);
    assert_eq!(panel.transform.position.y, 2.0);
    assert_eq!(panel.transform.position.z, 3.0);
}

// ── Phase 6 tests ────────────────────────────────────────────────────────

/// Helper: create a minimal VNode with only the fields we care about.
fn make_node(
    id: KvasirId,
    layout: LayoutRect,
    children: Vec<KvasirId>,
    world_space: Option<WorldSpacePanel>,
) -> VNode {
    VNode {
        id,
        key: None,
        component_type: "div".to_string(),
        props: HashMap::new(),
        state: None,
        layout,
        children,
        aria_role: "div".to_string(),
        aria_props: AriaProps::default(),
        portal_target: None,
        world_space,
        theme_override: None,
        color_palette: u16::MAX,
        sdf_shape: None,
        companions: HashMap::new(),
    }
}

/// Helper: build a minimal VDom with a panel root → child → grandchild tree.
fn build_panel_vdom() -> (VDom, KvasirId, KvasirId, KvasirId) {
    let root_id = KvasirId::new();
    let child_id = KvasirId::new();
    let grandchild_id = KvasirId::new();

    let panel = WorldSpacePanel {
        transform: Transform3D {
            position: Vec3::new(5.0, 10.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        pixels_per_unit: 100.0,
        world_size: (2.0, 1.0),
        ..Default::default()
    };

    let root_node = make_node(
        root_id,
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
        },
        vec![child_id],
        Some(panel),
    );
    let child_node = make_node(
        child_id,
        LayoutRect {
            x: 10.0,
            y: 20.0,
            width: 80.0,
            height: 40.0,
        },
        vec![grandchild_id],
        None,
    );
    let grandchild_node = make_node(
        grandchild_id,
        LayoutRect {
            x: 5.0,
            y: 5.0,
            width: 30.0,
            height: 15.0,
        },
        vec![],
        None,
    );

    let mut vdom = VDom::new();
    vdom.root = Some(root_id);
    vdom.parents.insert(child_id, root_id);
    vdom.parents.insert(grandchild_id, child_id);
    vdom.nodes.insert(root_id, root_node);
    vdom.nodes.insert(child_id, child_node);
    vdom.nodes.insert(grandchild_id, grandchild_node);

    (vdom, root_id, child_id, grandchild_id)
}

#[test]
fn test_world_space_position_child_composes_local_offsets() {
    let (vdom, _, child_id, _) = build_panel_vdom();
    let pos = vdom
        .world_space_position(child_id)
        .expect("child must resolve");
    // child.layout = (10, 20) relative to panel, no parent between child and panel
    assert_eq!(pos.local_offset, (10.0, 20.0));
    assert_eq!(pos.size, (80.0, 40.0));
    assert_eq!(pos.pixels_per_unit, 100.0);
    assert_eq!(pos.world_size, (2.0, 1.0));
    assert_eq!(pos.panel_transform.position, Vec3::new(5.0, 10.0, 0.0));
}

#[test]
fn test_world_space_position_grandchild_composes_three_levels() {
    let (vdom, _, _, grandchild_id) = build_panel_vdom();
    let pos = vdom
        .world_space_position(grandchild_id)
        .expect("grandchild must resolve");
    // grandchild.layout = (5, 5) + child.layout = (10, 20) = (15, 25)
    assert_eq!(pos.local_offset, (15.0, 25.0));
    assert_eq!(pos.size, (30.0, 15.0));
}

#[test]
fn test_world_space_position_panel_root_returns_zero_offset() {
    let (vdom, root_id, _, _) = build_panel_vdom();
    let pos = vdom
        .world_space_position(root_id)
        .expect("panel root must resolve");
    // panel root's own layout is (0,0) and we don't add it to itself
    assert_eq!(pos.local_offset, (0.0, 0.0));
    assert_eq!(pos.size, (200.0, 100.0));
}

#[test]
fn test_resolved_position_returns_world_space_for_panel_child() {
    let (vdom, _, child_id, _) = build_panel_vdom();
    match vdom.resolved_position(child_id) {
        Some(ResolvedPosition::WorldSpace(pos)) => {
            assert_eq!(pos.local_offset, (10.0, 20.0));
        }
        other => panic!("expected WorldSpace, got {:?}", other),
    }
}

#[test]
fn test_resolved_position_returns_screen_space_for_plain_node() {
    let root_id = KvasirId::new();
    let child_id = KvasirId::new();

    let root_node = make_node(
        root_id,
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
        },
        vec![child_id],
        None,
    );
    let child_node = make_node(
        child_id,
        LayoutRect {
            x: 10.0,
            y: 20.0,
            width: 50.0,
            height: 25.0,
        },
        vec![],
        None,
    );

    let mut vdom = VDom::new();
    vdom.root = Some(root_id);
    vdom.parents.insert(child_id, root_id);
    vdom.nodes.insert(root_id, root_node);
    vdom.nodes.insert(child_id, child_node);

    match vdom.resolved_position(child_id) {
        Some(ResolvedPosition::ScreenSpace(rect)) => {
            assert_eq!(rect.x, 10.0);
            assert_eq!(rect.y, 20.0);
        }
        other => panic!("expected ScreenSpace, got {:?}", other),
    }
}

#[test]
fn test_world_space_position_none_for_plain_node() {
    let root_id = KvasirId::new();
    let root_node = make_node(
        root_id,
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        },
        vec![],
        None,
    );
    let mut vdom = VDom::new();
    vdom.root = Some(root_id);
    vdom.nodes.insert(root_id, root_node);

    assert!(vdom.world_space_position(root_id).is_none());
}
