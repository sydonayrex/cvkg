//! Crosscrate identity integration tests.
//!
//! Verifies that KvasirId from cvkg-core is the unified identity type
// across cvkg-scene, cvkg-vdom, and cvkg-flow. These crates previously
// had independent `NodeId(pub u64)` structs that were incompatible.
// After the crosscrate stabilization, all three expose NodeId as a type
// alias for cvkg_core::KvasirId.

use cvkg_core::KvasirId;
use cvkg_flow::types::NodeId as FlowNodeId;
use cvkg_scene::NodeId as SceneNodeId;
use cvkg_vdom::NodeId as VDomNodeId;

// ---------------------------------------------------------------------------
// Type identity: all NodeId aliases resolve to the same type
// ---------------------------------------------------------------------------

#[test]
fn test_type_alias_identity() {
    // If these compile, the type aliases are transparent to the compiler.
    let core_id = KvasirId(42);
    let scene_id: SceneNodeId = core_id;
    let vdom_id: VDomNodeId = core_id;
    let flow_id: FlowNodeId = core_id;

    // Round-trip back to KvasirId.
    let back: KvasirId = scene_id;
    let back2: KvasirId = vdom_id;
    let back3: KvasirId = flow_id;

    assert_eq!(back.0, 42);
    assert_eq!(back2.0, 42);
    assert_eq!(back3.0, 42);
}

// ---------------------------------------------------------------------------
// Constructor interoperability
// ---------------------------------------------------------------------------

#[test]
fn test_kvasir_id_constructor_works_for_all_aliases() {
    // KvasirId() can be used wherever a NodeId alias is expected.
    let scene_node: SceneNodeId = KvasirId(1);
    let vdom_node: VDomNodeId = KvasirId(2);
    let flow_node: FlowNodeId = KvasirId(3);

    assert_eq!(scene_node.0, 1);
    assert_eq!(vdom_node.0, 2);
    assert_eq!(flow_node.0, 3);
}

#[test]
fn test_node_id_alias_passes_to_kvasir_id_function() {
    fn process(id: KvasirId) -> u64 {
        id.0
    }

    let scene_id: SceneNodeId = KvasirId(100);
    let vdom_id: VDomNodeId = KvasirId(200);
    let flow_id: FlowNodeId = KvasirId(300);

    assert_eq!(process(scene_id), 100);
    assert_eq!(process(vdom_id), 200);
    assert_eq!(process(flow_id), 300);
}

// ---------------------------------------------------------------------------
// Conversion traits
// ---------------------------------------------------------------------------

#[test]
fn test_from_u64_conversion() {
    let id: KvasirId = 999u64.into();
    assert_eq!(id.0, 999);

    let scene_id: SceneNodeId = 888u64.into();
    assert_eq!(scene_id.0, 888);
}

#[test]
fn test_from_kvasir_id_to_u64() {
    let id = KvasirId(777);
    let val: u64 = id.into();
    assert_eq!(val, 777);
}

// ---------------------------------------------------------------------------
// Crosscrate data flow: scene -> vdom -> flow
// ---------------------------------------------------------------------------

#[test]
fn test_crosscrate_node_id_round_trip() {
    // Simulate a node ID flowing through the system:
    // 1. Created in cvkg-core as KvasirId
    // 2. Used in cvkg-scene as NodeId
    // 3. Referenced in cvkg-vdom as NodeId
    // 4. Referenced in cvkg-flow as NodeId
    // 5. Arrives back as KvasirId with the same value

    let original = KvasirId::new();
    let scene_node: SceneNodeId = original;
    let vdom_node: VDomNodeId = scene_node;
    let flow_node: FlowNodeId = vdom_node;
    let final_id: KvasirId = flow_node;

    assert_eq!(original.0, final_id.0);
}

#[test]
fn test_kvasir_id_null_is_consistent_across_crates() {
    let null_core = KvasirId::NULL;
    let null_scene: SceneNodeId = null_core;
    let null_vdom: VDomNodeId = null_core;
    let null_flow: FlowNodeId = null_core;

    // All nulls should have the same underlying value.
    assert_eq!(null_core.0, null_scene.0);
    assert_eq!(null_core.0, null_vdom.0);
    assert_eq!(null_core.0, null_flow.0);
}

#[test]
fn test_kvasir_id_display_format() {
    let id = KvasirId(12345);
    let s = format!("{}", id);
    assert!(s.contains("12345"));
}

// ---------------------------------------------------------------------------
// Collection interoperability
// ---------------------------------------------------------------------------

#[test]
fn test_mixed_crate_node_ids_in_same_collection() {
    // Because all NodeId aliases ARE KvasirId, they can coexist in a
    // single collection without conversion.
    let mut ids: Vec<KvasirId> = Vec::new();

    let scene_id: SceneNodeId = KvasirId(1);
    let vdom_id: VDomNodeId = KvasirId(2);
    let flow_id: FlowNodeId = KvasirId(3);

    ids.push(scene_id);
    ids.push(vdom_id);
    ids.push(flow_id);

    assert_eq!(ids.len(), 3);
    assert_eq!(ids[0].0, 1);
    assert_eq!(ids[1].0, 2);
    assert_eq!(ids[2].0, 3);
}

#[test]
fn test_hashmap_with_crosscrate_keys() {
    use std::collections::HashMap;

    let mut map: HashMap<KvasirId, &str> = HashMap::new();

    let scene_id: SceneNodeId = KvasirId(10);
    let vdom_id: VDomNodeId = KvasirId(20);
    let flow_id: FlowNodeId = KvasirId(30);

    map.insert(scene_id, "scene");
    map.insert(vdom_id, "vdom");
    map.insert(flow_id, "flow");

    // Lookup using any alias type.
    assert_eq!(map.get(&KvasirId(10)), Some(&"scene"));
    assert_eq!(map.get(&scene_id), Some(&"scene"));
    assert_eq!(map.get(&vdom_id), Some(&"vdom"));
    assert_eq!(map.get(&flow_id), Some(&"flow"));
}

// ---------------------------------------------------------------------------
// Equality across aliases
// ---------------------------------------------------------------------------

#[test]
fn test_equality_across_crate_aliases() {
    let scene_id: SceneNodeId = KvasirId(99);
    let vdom_id: VDomNodeId = KvasirId(99);
    let flow_id: FlowNodeId = KvasirId(99);
    let core_id = KvasirId(99);

    assert_eq!(scene_id, vdom_id);
    assert_eq!(scene_id, flow_id);
    assert_eq!(scene_id, core_id);
    assert_eq!(vdom_id, flow_id);
    assert_eq!(vdom_id, core_id);
    assert_eq!(flow_id, core_id);
}

#[test]
fn test_inequality_across_crate_aliases() {
    let scene_id: SceneNodeId = KvasirId(1);
    let vdom_id: VDomNodeId = KvasirId(2);
    let flow_id: FlowNodeId = KvasirId(3);

    assert_ne!(scene_id, vdom_id);
    assert_ne!(scene_id, flow_id);
    assert_ne!(vdom_id, flow_id);
}
