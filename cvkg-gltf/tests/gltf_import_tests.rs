//! Tests for cvkg-gltf glTF loading.
//!
//! These tests verify that the importer correctly converts glTF 2.0
//! assets into CVKG scene data. Tests construct in-memory glTF data
//! and exercise the conversion pipeline.

use cvkg_gltf::load_gltf;

/// Write a minimal valid .glb file and load it.
///
/// The glb contains a single triangle mesh, one material, and one node
/// with a translation of (1, 2, 3).
fn write_minimal_glb(path: &std::path::Path) {
    // Binary data:
    //   3 positions (36 bytes), 3 normals (36 bytes), 3 texcoords (24 bytes),
    //   3 indices (6 bytes) = 102 bytes total
    let mut bin = Vec::new();

    // Positions: 3 vertices of a triangle at z=0
    for v in &[[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
        bin.extend_from_slice(&v[0].to_le_bytes());
        bin.extend_from_slice(&v[1].to_le_bytes());
        bin.extend_from_slice(&v[2].to_le_bytes());
    }
    // Normals: all facing +Z
    for _ in 0..3 {
        bin.extend_from_slice(&0.0f32.to_le_bytes());
        bin.extend_from_slice(&0.0f32.to_le_bytes());
        bin.extend_from_slice(&1.0f32.to_le_bytes());
    }
    // Texcoords: 3 UV pairs
    for uv in &[[0.0f32, 0.0], [1.0, 0.0], [0.0, 1.0]] {
        bin.extend_from_slice(&uv[0].to_le_bytes());
        bin.extend_from_slice(&uv[1].to_le_bytes());
    }
    // Indices: 3 u16 values
    for idx in [0u16, 1, 2] {
        bin.extend_from_slice(&idx.to_le_bytes());
    }

    let bin_len = bin.len() as u32;

    // JSON describing the glTF scene — the binary buffer is inline via BIN chunk
    let json = serde_json::json!({
        "asset": {"version": "2.0", "generator": "cvkg-gltf-test"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{
            "mesh": 0,
            "translation": [1.0, 2.0, 3.0],
            "rotation": [0.0, 0.0, 0.0, 1.0],
            "scale": [1.0, 1.0, 1.0]
        }],
        "meshes": [{
            "primitives": [{
                "attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2},
                "indices": 3,
                "material": 0
            }]
        }],
        "accessors": [
            {"bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3",
             "max": [1.0, 0.0, 1.0], "min": [0.0, 0.0, 0.0]},
            {"bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC3"},
            {"bufferView": 2, "componentType": 5126, "count": 3, "type": "VEC2"},
            {"bufferView": 3, "componentType": 5123, "count": 3, "type": "SCALAR"}
        ],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962},
            {"buffer": 0, "byteOffset": 36, "byteLength": 36, "target": 34962},
            {"buffer": 0, "byteOffset": 72, "byteLength": 24, "target": 34962},
            {"buffer": 0, "byteOffset": 96, "byteLength": 6, "target": 34963}
        ],
        "buffers": [{"byteLength": bin_len}],
        "materials": [{
            "pbrMetallicRoughness": {
                "baseColorFactor": [0.5, 0.5, 0.5, 1.0],
                "metallicFactor": 1.0,
                "roughnessFactor": 0.25
            },
            "emissiveFactor": [0.1, 0.0, 0.0]
        }]
    });

    let json_bytes = serde_json::to_vec(&json).unwrap();
    // Pad JSON to 4-byte alignment
    let json_padded = pad4(json_bytes);
    // Pad BIN to 4-byte alignment
    let bin_padded = pad4(bin);

    // glb header: magic + version + total_length
    let total_len: u32 = 12                     // header
        + 8 + json_padded.len() as u32          // JSON chunk
        + 8 + bin_padded.len() as u32;          // BIN chunk

    let mut glb = Vec::with_capacity(total_len as usize);
    glb.extend_from_slice(b"glTF");             // magic
    glb.extend_from_slice(&2u32.to_le_bytes()); // version 2
    glb.extend_from_slice(&total_len.to_le_bytes());

    // JSON chunk
    glb.extend_from_slice(&(json_padded.len() as u32).to_le_bytes());
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // chunk type = JSON
    glb.extend_from_slice(&json_padded);

    // BIN chunk
    glb.extend_from_slice(&(bin_padded.len() as u32).to_le_bytes());
    glb.extend_from_slice(&0x004E4942u32.to_le_bytes()); // chunk type = BIN
    glb.extend_from_slice(&bin_padded);

    std::fs::write(path, &glb).unwrap();
}

fn pad4(mut data: Vec<u8>) -> Vec<u8> {
    let pad = (4 - data.len() % 4) % 4;
    data.extend(std::iter::repeat(b' ').take(pad));
    data
}

#[test]
fn test_load_minimal_glb() {
    let tmp = std::env::temp_dir().join(format!("test_{}.glb", unique_id()));
    write_minimal_glb(&tmp);

    let scene = load_gltf(&tmp).expect("Should load minimal glTF");
    assert!(!scene.nodes.is_empty(), "Should have at least one node");
    assert!(!scene.meshes.is_empty(), "Should have at least one mesh");

    // Check node
    let node = &scene.nodes[0];
    assert!(node.parent.is_none());
    assert_eq!(node.transform.position, glam::Vec3::new(1.0, 2.0, 3.0));

    // Check mesh
    let loaded = &scene.meshes[0];
    assert_eq!(
        loaded.mesh.vertices.len(),
        3,
        "Triangle should have 3 vertices"
    );
    assert_eq!(loaded.mesh.normals.len(), 3, "Should have 3 normals");
    assert_eq!(loaded.mesh.tex_coords.len(), 3, "Should have 3 UVs");
    assert_eq!(loaded.mesh.indices.len(), 3, "Should have 3 indices");
    assert_eq!(
        loaded.mesh.vertices[1],
        [1.0, 0.0, 0.0],
        "Second vertex should be (1,0,0)"
    );

    // Check material
    assert!(loaded.material_index.is_some());
    if let Some(mat_idx) = loaded.material_index {
        let mat = &scene.materials[mat_idx];
        assert_eq!(mat.base_color, [0.5, 0.5, 0.5, 1.0]);
        assert_eq!(mat.metallic, 1.0);
        assert_eq!(mat.roughness, 0.25);
        assert_eq!(mat.emissive, [0.1, 0.0, 0.0]);
    }

    // Clean up
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn test_nonexistent_file_error() {
    let result = load_gltf("/tmp/nonexistent_model_xyz.glb");
    assert!(result.is_err());
    if let Err(e) = result {
        let msg = e.to_string();
        assert!(msg.contains("Failed to load glTF") || msg.contains("No such file"));
    }
}

fn unique_id() -> u64 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
        ^ (id as u64)
}
