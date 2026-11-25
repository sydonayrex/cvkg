#![cfg(feature = "hot-patching")]

use cvkg_render_gpu::hot_patch::PatchRegistry;

#[test]
fn patch_registry_tracks_revisions() {
    let mut registry = PatchRegistry::new();
    let first = registry.put("pbr.wgsl", "base shader");
    let second = registry.put("pbr.wgsl", "patched shader");

    assert_eq!(first, 1);
    assert_eq!(second, 2);
    assert_eq!(
        registry.get("pbr.wgsl").map(|patch| patch.payload.as_str()),
        Some("patched shader")
    );
    assert_eq!(registry.take_updates(first).len(), 1);
    assert!(registry.take_updates(second).is_empty());
}
