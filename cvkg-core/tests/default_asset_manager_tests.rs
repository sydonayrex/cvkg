use cvkg_core::{AssetManager, AssetState, DefaultAssetManager};

#[test]
fn default_asset_manager_loads_an_existing_file() {
    let path = std::env::temp_dir().join("cvkg_default_asset_test.bin");
    std::fs::write(&path, vec![1u8, 2, 3]).expect("write test fixture");
    let manager = DefaultAssetManager::new();

    match manager.load_image(path.to_str().expect("utf8 path")) {
        AssetState::Ready(bytes) => assert_eq!(bytes.as_slice(), &[1u8, 2, 3]),
        AssetState::Loading => panic!("default manager stayed Loading"),
        AssetState::Error(message) => panic!("unexpected error: {message}"),
    }

    std::fs::remove_file(&path).expect("remove test fixture");
}
