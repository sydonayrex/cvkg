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
