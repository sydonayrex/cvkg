use cvkg_core::KvasirId;
use cvkg_core::mesh::Transform3D;
use cvkg_materials::GlassMaterial;
use cvkg_vdom::{VNode, WorldSpacePanel};
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
