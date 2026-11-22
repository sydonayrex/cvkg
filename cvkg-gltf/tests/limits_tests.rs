use cvkg_core::Limits;
use cvkg_gltf::load_gltf_with_limits;

#[test]
fn gltf_loader_rejects_files_over_the_byte_budget() {
    let path = std::env::temp_dir().join("cvkg_gltf_limits_test.glb");
    std::fs::write(&path, vec![0u8; 64]).expect("write fixture");
    let limits = Limits {
        max_bytes: 16,
        ..Limits::default()
    };

    assert!(load_gltf_with_limits(&path, limits).is_err());

    std::fs::remove_file(&path).expect("remove fixture");
}
