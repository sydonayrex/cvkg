use cvkg_macros::Reflect;
use cvkg_reflect::Reflected;

#[derive(Reflect)]
struct TestStruct {
    enabled: bool,
    #[reflect(kind = "Float", doc = "Opacity value")]
    opacity: f32,
    #[reflect(read_only)]
    name: String,
}

#[test]
fn test_type_meta() {
    let meta = TestStruct::type_meta();
    assert_eq!(meta.type_name, "TestStruct");
    assert_eq!(meta.fields.len(), 3);
    assert_eq!(meta.fields[0].name, "enabled");
    assert_eq!(meta.fields[0].kind, cvkg_reflect::FieldKind::Bool);
    assert_eq!(meta.fields[1].name, "opacity");
    assert_eq!(meta.fields[1].kind, cvkg_reflect::FieldKind::Float);
    assert_eq!(meta.fields[1].doc, "Opacity value");
    assert!(meta.fields[2].read_only);
}

#[test]
fn test_get_field() {
    let s = TestStruct {
        enabled: true,
        opacity: 0.5,
        name: "test".into(),
    };
    assert_eq!(
        s.get_field("enabled"),
        Some(serde_json::Value::Bool(true))
    );
    assert!(s.get_field("nonexistent").is_none());
}

#[test]
fn test_set_field() {
    let mut s = TestStruct {
        enabled: false,
        opacity: 0.0,
        name: "old".into(),
    };
    s.set_field("enabled", serde_json::Value::Bool(true))
        .unwrap();
    assert!(s.enabled);
}

#[test]
fn test_set_field_float() {
    let mut s = TestStruct {
        enabled: false,
        opacity: 0.0,
        name: "old".into(),
    };
    s.set_field("opacity", serde_json::json!(0.75)).unwrap();
    assert!((s.opacity - 0.75).abs() < 1e-5);
}

#[test]
fn test_set_field_string() {
    let mut s = TestStruct {
        enabled: false,
        opacity: 0.0,
        name: "old".into(),
    };
    // "name" is read_only, so this should fail
    let err = s
        .set_field("name", serde_json::json!("new"))
        .unwrap_err();
    match err {
        cvkg_reflect::ReflectError::ReadOnly(n) => assert_eq!(n, "name"),
        other => panic!("expected ReadOnly, got {:?}", other),
    }
    assert_eq!(s.name, "old");
}

#[test]
fn test_set_field_not_found() {
    let mut s = TestStruct {
        enabled: false,
        opacity: 0.0,
        name: "old".into(),
    };
    let err = s
        .set_field("nonexistent", serde_json::Value::Bool(true))
        .unwrap_err();
    match err {
        cvkg_reflect::ReflectError::FieldNotFound(n) => assert_eq!(n, "nonexistent"),
        other => panic!("expected FieldNotFound, got {:?}", other),
    }
}

#[test]
fn test_set_field_type_mismatch() {
    let mut s = TestStruct {
        enabled: false,
        opacity: 0.0,
        name: "old".into(),
    };
    let err = s
        .set_field("enabled", serde_json::json!("not_a_bool"))
        .unwrap_err();
    match err {
        cvkg_reflect::ReflectError::TypeMismatch {
            field,
            expected,
            ..
        } => {
            assert_eq!(field, "enabled");
            assert_eq!(expected, "bool");
        }
        other => panic!("expected TypeMismatch, got {:?}", other),
    }
}

#[test]
fn test_snapshot() {
    let s = TestStruct {
        enabled: true,
        opacity: 0.5,
        name: "test".into(),
    };
    let snap = s.snapshot();
    assert_eq!(snap.len(), 3);
    assert_eq!(snap["enabled"], serde_json::json!(true));
    assert_eq!(snap["name"], serde_json::json!("test"));
}

#[test]
fn test_field_names_via_meta() {
    let meta = TestStruct::type_meta();
    let names: Vec<&str> = meta.field_names().collect();
    assert_eq!(names, vec!["enabled", "opacity", "name"]);
}

#[derive(Reflect)]
struct CustomKindStruct {
    tags: [f32; 3],
}

#[test]
fn test_custom_field_kind_not_double_wrapped() {
    let meta = CustomKindStruct::type_meta();
    match &meta.fields[0].kind {
        cvkg_reflect::FieldKind::Custom(s) => assert_eq!(*s, "[f32;3]"),
        other => panic!("expected Custom, got {:?}", other),
    }
}

#[test]
fn test_read_only_blocks_writes() {
    let mut s = TestStruct {
        enabled: true,
        opacity: 1.0,
        name: "immutable".into(),
    };
    // read_only field rejects writes
    let err = s
        .set_field("name", serde_json::json!("changed"))
        .unwrap_err();
    assert!(matches!(err, cvkg_reflect::ReflectError::ReadOnly(_)));
    assert_eq!(s.name, "immutable");

    // non-read_only field still works
    s.set_field("enabled", serde_json::json!(false)).unwrap();
    assert!(!s.enabled);
}
