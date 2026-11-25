use cvkg_cli::component_registry::ComponentRegistry;

#[test]
fn registry_lists_components_and_returns_source() {
    let registry = ComponentRegistry::load_default().expect("manifest must parse");
    assert!(
        registry.entries.iter().any(|entry| entry.name == "Button"),
        "Button must be in the registry"
    );
    let source = registry.source("Button").expect("Button source");
    assert!(source.contains("pub struct Button"));
}

#[test]
fn add_and_update_copy_component_source() {
    use cvkg_cli::component_registry::ComponentRegistry;

    let registry = ComponentRegistry::load_default().expect("manifest must parse");
    let directory = tempfile::tempdir().expect("temp dir");
    let copied = registry.add("Button", directory.path()).expect("add");
    let canonical = std::fs::read_to_string(&copied).expect("copied source");

    std::fs::write(&copied, "stale").expect("overwrite copy");
    assert_eq!(registry.matches_local("Button", &copied), Some(false));

    assert!(registry.update("Button", &copied).expect("update"));
    assert_eq!(
        std::fs::read_to_string(&copied).expect("updated source"),
        canonical
    );
}
