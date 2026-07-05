/// Kvasir node tests — verifies that 3D Kvasir nodes implement the trait correctly.
use cvkg_render_gpu::passes::shadow::DirectionalLight;
use cvkg_render_3d::{Opaque3dNode, ShadowNode};
use cvkg_render_gpu::kvasir::{KvasirNode, ResourceId};

fn make_shadow_node() -> ShadowNode {
    ShadowNode {
        light: DirectionalLight::default(),
        shadow_map: ResourceId(0),
        mesh_instances: vec![],
        scene_radius: 100.0,
    }
}

fn make_opaque3d_node() -> Opaque3dNode {
    Opaque3dNode {
        mesh_instances: vec![],
        light: DirectionalLight::default(),
        shadow_map: ResourceId(0),
    }
}

#[test]
fn test_shadow_node_label() {
    let node = make_shadow_node();
    assert_eq!(node.label(), "ShadowPass");
}

#[test]
fn test_shadow_node_outputs_shadow_map() {
    let node = make_shadow_node();
    assert_eq!(
        node.outputs().len(),
        1,
        "shadow node must output exactly one resource (shadow map)"
    );
}

#[test]
fn test_opaque_3d_node_label() {
    let node = make_opaque3d_node();
    assert_eq!(node.label(), "Opaque3d");
}

#[test]
fn test_opaque_3d_node_outputs_empty() {
    let node = make_opaque3d_node();
    assert_eq!(
        node.outputs().len(),
        0,
        "opaque3d node has no output resources"
    );
}

#[test]
fn test_shadow_node_inputs_empty() {
    let node = make_shadow_node();
    assert_eq!(node.inputs().len(), 0, "shadow node has no input resources");
}

#[test]
fn test_opaque_3d_node_inputs_empty() {
    let node = make_opaque3d_node();
    assert_eq!(
        node.inputs().len(),
        0,
        "opaque3d node has no input resources"
    );
}
