use cvkg_core::{KvasirId, Transform3D};
use cvkg_render_3d_hierarchy::{HierarchyError, TransformNode3D, propagate_transforms};
use glam::{Mat4, Vec3};

fn make_node(id: u64, parent: Option<u64>, position: Vec3, children: Vec<u64>) -> TransformNode3D {
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
    propagate_transforms(&mut nodes).unwrap();

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
    propagate_transforms(&mut nodes).unwrap();

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
    propagate_transforms(&mut nodes).unwrap();

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
    propagate_transforms(&mut nodes).unwrap();
    assert!(nodes.is_empty());
}

#[test]
fn test_multiple_roots_independent() {
    let mut nodes = vec![
        make_node(1, None, Vec3::new(10.0, 0.0, 0.0), vec![]),
        make_node(2, None, Vec3::new(0.0, 20.0, 0.0), vec![]),
    ];
    propagate_transforms(&mut nodes).unwrap();

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

#[test]
fn test_duplicate_id_returns_error() {
    let mut nodes = vec![
        make_node(1, None, Vec3::ZERO, vec![]),
        make_node(1, None, Vec3::ZERO, vec![]),
    ];
    assert_eq!(
        propagate_transforms(&mut nodes),
        Err(HierarchyError::DuplicateId(KvasirId(1)))
    );
}

#[test]
fn test_self_reference_returns_error() {
    let mut nodes = vec![TransformNode3D {
        id: KvasirId(1),
        parent: Some(KvasirId(1)),
        children: vec![],
        local: Transform3D::default(),
        global: Mat4::IDENTITY,
    }];
    assert_eq!(
        propagate_transforms(&mut nodes),
        Err(HierarchyError::SelfReference(KvasirId(1)))
    );
}

#[test]
fn test_parent_not_found_returns_error() {
    let mut nodes = vec![make_node(1, Some(99), Vec3::ZERO, vec![])];
    assert_eq!(
        propagate_transforms(&mut nodes),
        Err(HierarchyError::ParentNotFound {
            child: KvasirId(1),
            parent: KvasirId(99),
        })
    );
}

#[test]
fn test_cycle_returns_error() {
    let mut nodes = vec![
        make_node(1, Some(2), Vec3::ZERO, vec![]),
        make_node(2, Some(1), Vec3::ZERO, vec![]),
    ];
    let result = propagate_transforms(&mut nodes);
    assert!(matches!(result, Err(HierarchyError::CycleDetected(_))));
}

#[test]
fn test_parent_after_child_is_correct() {
    // Child appears before parent in the slice — should still compute correctly.
    let mut nodes = vec![
        make_node(2, Some(1), Vec3::new(0.0, 5.0, 0.0), vec![]),
        make_node(1, None, Vec3::new(10.0, 0.0, 0.0), vec![2]),
    ];
    propagate_transforms(&mut nodes).unwrap();

    let root_mat = Transform3D {
        position: Vec3::new(10.0, 0.0, 0.0),
        ..Transform3D::default()
    }
    .to_mat4();
    let child_local = Transform3D {
        position: Vec3::new(0.0, 5.0, 0.0),
        ..Transform3D::default()
    }
    .to_mat4();

    // Slice order is unchanged, but transforms are correct.
    let root = nodes.iter().find(|n| n.id == KvasirId(1)).unwrap();
    let child = nodes.iter().find(|n| n.id == KvasirId(2)).unwrap();
    assert_eq!(root.global, root_mat);
    assert_eq!(child.global, root_mat * child_local);
}

#[test]
fn test_three_level_cycle() {
    let mut nodes = vec![
        make_node(1, Some(3), Vec3::ZERO, vec![]),
        make_node(2, Some(1), Vec3::ZERO, vec![]),
        make_node(3, Some(2), Vec3::ZERO, vec![]),
    ];
    let result = propagate_transforms(&mut nodes);
    assert!(matches!(result, Err(HierarchyError::CycleDetected(_))));
}

#[test]
fn test_diamond_hierarchy() {
    //     1
    //    / \
    //   2   3
    //    \ /
    //     4
    let mut nodes = vec![
        make_node(1, None, Vec3::new(1.0, 0.0, 0.0), vec![2, 3]),
        make_node(2, Some(1), Vec3::new(0.0, 1.0, 0.0), vec![4]),
        make_node(3, Some(1), Vec3::new(0.0, 0.0, 1.0), vec![4]),
        make_node(4, Some(2), Vec3::new(0.0, 0.0, 0.0), vec![]),
    ];
    propagate_transforms(&mut nodes).unwrap();

    // Node 4's parent is node 2, not node 3.
    let root = Transform3D {
        position: Vec3::new(1.0, 0.0, 0.0),
        ..Transform3D::default()
    }
    .to_mat4();
    let branch2 = Transform3D {
        position: Vec3::new(0.0, 1.0, 0.0),
        ..Transform3D::default()
    }
    .to_mat4();

    let node4 = nodes.iter().find(|n| n.id == KvasirId(4)).unwrap();
    assert_eq!(node4.global, root * branch2);
}
