use cvkg_core::{KvasirId, Transform3D};
use cvkg_render_3d_hierarchy::{TransformNode3D, propagate_transforms};
use glam::{Mat4, Vec3};

fn make_node(
    id: u64,
    parent: Option<u64>,
    position: Vec3,
    children: Vec<u64>,
) -> TransformNode3D {
    TransformNode3D {
        id: KvasirId(id),
        parent: parent.map(KvasirId),
        children: children.into_iter().map(KvasirId).collect(),
        local: Transform3D {
            position,
            ..Transform3D::default()
        },
        global: Mat4::IDENTITY,
    }
}

#[test]
fn test_root_node_global_equals_local() {
    let pos = Vec3::new(1.0, 2.0, 3.0);
    let mut nodes = vec![make_node(1, None, pos, vec![])];
    propagate_transforms(&mut nodes);

    let expected = Transform3D {
        position: pos,
        ..Transform3D::default()
    }
    .to_mat4();
    assert_eq!(nodes[0].global, expected);
}

#[test]
fn test_child_inherits_parent_transform() {
    let mut nodes = vec![
        make_node(1, None, Vec3::new(10.0, 0.0, 0.0), vec![2]),
        make_node(2, Some(1), Vec3::new(0.0, 5.0, 0.0), vec![]),
    ];
    propagate_transforms(&mut nodes);

    let parent_global = nodes[0].global;
    let child_local = Transform3D {
        position: Vec3::new(0.0, 5.0, 0.0),
        ..Transform3D::default()
    }
    .to_mat4();
    let expected = parent_global * child_local;
    assert_eq!(nodes[1].global, expected);
}

#[test]
fn test_grandchild_inherits_chain() {
    let mut nodes = vec![
        make_node(1, None, Vec3::new(1.0, 0.0, 0.0), vec![2]),
        make_node(2, Some(1), Vec3::new(0.0, 2.0, 0.0), vec![3]),
        make_node(3, Some(2), Vec3::new(0.0, 0.0, 3.0), vec![]),
    ];
    propagate_transforms(&mut nodes);

    let root_mat = Transform3D {
        position: Vec3::new(1.0, 0.0, 0.0),
        ..Transform3D::default()
    }
    .to_mat4();
    let child_mat = Transform3D {
        position: Vec3::new(0.0, 2.0, 0.0),
        ..Transform3D::default()
    }
    .to_mat4();
    let grandchild_mat = Transform3D {
        position: Vec3::new(0.0, 0.0, 3.0),
        ..Transform3D::default()
    }
    .to_mat4();

    assert_eq!(nodes[0].global, root_mat);
    assert_eq!(nodes[1].global, root_mat * child_mat);
    assert_eq!(nodes[2].global, root_mat * child_mat * grandchild_mat);
}

#[test]
fn test_empty_scene_no_panic() {
    let mut nodes: Vec<TransformNode3D> = vec![];
    propagate_transforms(&mut nodes);
    assert!(nodes.is_empty());
}

#[test]
fn test_multiple_roots_independent() {
    let mut nodes = vec![
        make_node(1, None, Vec3::new(10.0, 0.0, 0.0), vec![]),
        make_node(2, None, Vec3::new(0.0, 20.0, 0.0), vec![]),
    ];
    propagate_transforms(&mut nodes);

    let expected_a = Transform3D {
        position: Vec3::new(10.0, 0.0, 0.0),
        ..Transform3D::default()
    }
    .to_mat4();
    let expected_b = Transform3D {
        position: Vec3::new(0.0, 20.0, 0.0),
        ..Transform3D::default()
    }
    .to_mat4();

    assert_eq!(nodes[0].global, expected_a);
    assert_eq!(nodes[1].global, expected_b);
}
