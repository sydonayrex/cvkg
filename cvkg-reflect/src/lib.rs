//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     -- State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     -- Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     -- Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    -- Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     -- Read the target, its surrounding context, and its full call graph
//                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     -- Every major pub fn, unsafe block, and non-trivial algorithm in
//                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   -- Check every tool call / command for progress every 30 seconds.
//                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//   CVKG Extended: Section 2 of the CVKG Design Specification

//! CVKG Reflect — runtime type metadata and property access.
//!
//! # Why this exists
//! Finding #8 from the crosscrate audit: without reflection, property editors,
//! inspectors, and telemetry systems must hard-code every type they want to
//! display or serialize. This crate provides a lightweight runtime reflection
//! model that is explicit (not magic) — types opt in via the `Reflected` trait.
//!
//! # Design
//! - No proc-macros in this crate (macros belong in cvkg-macros)
//! - Types implement `Reflected` manually or via future `#[derive(Reflect)]`
//! - Property access is dynamic via `serde_json::Value` for simplicity
//! - Field metadata is static (built at compile time, stored in slices)

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ── FieldKind ─────────────────────────────────────────────────────────────────

/// The semantic type category of a reflected field.
///
/// # Why this exists
/// A plain Rust type name is not enough for a property editor or inspector — it
/// needs to know *how* to render a field (slider vs. color picker vs. text box).
/// `FieldKind` carries that intent without requiring the inspector to parse type
/// strings at runtime.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldKind {
    /// A boolean toggle (checkbox / switch).
    Bool,
    /// A 64-bit integer value.
    Integer,
    /// A 32-bit floating-point scalar.
    Float,
    /// A UTF-8 string.
    String,
    /// An RGBA color represented as four `f32` components in [0.0, 1.0].
    Color,
    /// A 2D vector (x, y).
    Vec2,
    /// A 3D vector (x, y, z).
    Vec3,
    /// An axis-aligned rectangle (x, y, width, height).
    Rect,
    /// A user-defined type identified by its canonical type name.
    /// Use this when none of the built-in kinds match.
    Custom(&'static str),
}

// ── FieldMeta ─────────────────────────────────────────────────────────────────

/// Compile-time metadata describing a single field of a reflected type.
///
/// # Contract
/// All string fields are `&'static str` so that `FieldMeta` values can live in
/// `static` storage with zero heap allocation. This is intentional: the schema
/// never changes at runtime.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FieldMeta {
    /// The field's identifier as it appears in the Rust struct.
    pub name: &'static str,
    /// The semantic kind used by property editors and inspectors.
    pub kind: FieldKind,
    /// Human-readable description shown in inspectors and hover tooltips.
    pub doc: &'static str,
    /// If `true`, `set_field` MUST return `ReflectError::ReadOnly` for this field.
    /// Inspectors should render this field as disabled.
    pub read_only: bool,
}

// ── TypeMeta ──────────────────────────────────────────────────────────────────

/// Compile-time metadata describing the complete schema of a reflected type.
///
/// # Contract
/// `fields` is a `&'static [FieldMeta]` so the entire schema lives in the
/// binary's read-only data segment with no heap cost. The `Reflected` trait
/// requires types to return a `&'static TypeMeta`, which enforces this.
#[derive(Debug)]
pub struct TypeMeta {
    /// Canonical Rust type name (e.g. `"ColorStop"`, `"NodeTransform"`).
    pub type_name: &'static str,
    /// Ordered slice of all reflected fields for this type.
    pub fields: &'static [FieldMeta],
}

impl TypeMeta {
    /// Look up a field by its identifier string.
    ///
    /// # Why this exists
    /// Property editors and telemetry paths address fields by name strings, not
    /// by index. This linear scan is acceptable because field counts are small
    /// (≤ ~64) and `TypeMeta` is static — there is no heap allocation.
    ///
    /// Returns `None` if no field with `name` exists.
    pub fn field(&self, name: &str) -> Option<&FieldMeta> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// Iterate the names of all reflected fields in declaration order.
    ///
    /// # Why this exists
    /// Needed by snapshot machinery and generic serializers that must enumerate
    /// all fields without knowing the concrete type at compile time.
    pub fn field_names(&self) -> impl Iterator<Item = &str> {
        self.fields.iter().map(|f| f.name)
    }
}

// ── ReflectError ──────────────────────────────────────────────────────────────

/// Errors that can occur during dynamic property access via `Reflected`.
///
/// # Why a plain enum rather than `thiserror`
/// `thiserror` is available in the workspace, but adding it as a dependency
/// here would be speculative. A manual `Display` implementation is trivial for
/// three variants and keeps the dependency surface minimal.
#[derive(Debug, Clone, PartialEq)]
pub enum ReflectError {
    /// The requested field name is not present in this type's `TypeMeta`.
    FieldNotFound(String),
    /// The field exists but is marked `read_only = true`.
    ReadOnly(String),
    /// The supplied `Value` variant does not match the field's `FieldKind`.
    TypeMismatch {
        field: String,
        expected: String,
        got: String,
    },
}

impl std::fmt::Display for ReflectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReflectError::FieldNotFound(name) => {
                write!(f, "reflect: field '{}' not found", name)
            }
            ReflectError::ReadOnly(name) => {
                write!(f, "reflect: field '{}' is read-only", name)
            }
            ReflectError::TypeMismatch {
                field,
                expected,
                got,
            } => {
                write!(
                    f,
                    "reflect: type mismatch on field '{}': expected {}, got {}",
                    field, expected, got
                )
            }
        }
    }
}

impl std::error::Error for ReflectError {}

// ── Reflected trait ───────────────────────────────────────────────────────────

/// Implemented by types that expose their fields for runtime introspection.
///
/// # Why this exists
/// Scene, Flow, Telemetry, Inspector, and Designer all need to read and write
/// properties on arbitrary types without hard-coding field names. `Reflected`
/// is the opt-in contract that makes this possible without proc-macros.
///
/// # Contract
/// - `type_meta()` must return a reference to a `'static` value. The simplest
///   approach is a `static` item in the impl block (see `ColorStop` example).
/// - `get_field` must return `None` for unknown field names (never panic).
/// - `set_field` must return `Err(ReflectError::FieldNotFound(_))` for unknown
///   names, `Err(ReflectError::ReadOnly(_))` for read-only fields, and
///   `Err(ReflectError::TypeMismatch { .. })` for incompatible value types.
/// - `snapshot` has a blanket default implementation that is correct for all
///   conforming implementations; do not override it unless you have a specific
///   performance need.
pub trait Reflected {
    /// Returns the static type metadata (field names, kinds, docs).
    /// This is the "schema" — it does not change at runtime.
    fn type_meta() -> &'static TypeMeta
    where
        Self: Sized;

    /// Get the current value of a named field, encoded as a `serde_json::Value`.
    ///
    /// Returns `None` if `name` is not a recognized field.
    fn get_field(&self, name: &str) -> Option<Value>;

    /// Set a named field from a `serde_json::Value`.
    ///
    /// # Errors
    /// - `ReflectError::FieldNotFound` — `name` is not a known field.
    /// - `ReflectError::ReadOnly` — the field is marked `read_only`.
    /// - `ReflectError::TypeMismatch` — the `Value` variant doesn't match the field kind.
    fn set_field(&mut self, name: &str, value: Value) -> Result<(), ReflectError>;

    /// Snapshot all fields as a `HashMap<name, Value>`.
    ///
    /// The default implementation calls `get_field` for every field listed in
    /// `type_meta()`. Fields that return `None` are silently omitted (this
    /// should not happen for a correct implementation, but it is safe to skip).
    fn snapshot(&self) -> HashMap<String, Value>
    where
        Self: Sized,
    {
        let meta = Self::type_meta();
        meta.fields
            .iter()
            .filter_map(|f| self.get_field(f.name).map(|v| (f.name.to_string(), v)))
            .collect()
    }
}

// ── ReflectRegistry ───────────────────────────────────────────────────────────

/// A runtime registry of reflected type schemas indexed by type name.
///
/// # Why this exists
/// Inspector panels and telemetry serializers need to discover all reflectable
/// types at startup without a linker-section trick. Crates register their types
/// during initialization by calling `register`. The registry does NOT own type
/// instances — it owns only `&'static TypeMeta` pointers, which are zero-cost.
///
/// # Contract
/// - `register` is idempotent: registering the same `TypeMeta` twice is safe
///   (the second call overwrites the first with the same pointer).
/// - `get` returns `None` for unknown type names; never panics.
#[derive(Default)]
pub struct ReflectRegistry {
    types: HashMap<&'static str, &'static TypeMeta>,
}

impl ReflectRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    /// Register a reflected type schema.
    ///
    /// The `meta.type_name` is used as the lookup key. Registering the same
    /// name twice overwrites the entry (last write wins).
    pub fn register(&mut self, meta: &'static TypeMeta) {
        self.types.insert(meta.type_name, meta);
    }

    /// Look up a type schema by its canonical name.
    ///
    /// Returns `None` if the type has not been registered.
    pub fn get(&self, type_name: &str) -> Option<&&'static TypeMeta> {
        self.types.get(type_name)
    }

    /// Iterate the canonical names of all registered types.
    ///
    /// Order is unspecified (HashMap iteration).
    pub fn type_names(&self) -> impl Iterator<Item = &&'static str> {
        self.types.keys()
    }
}

// ── Example implementation ────────────────────────────────────────────────────

/// A gradient color stop — used as a concrete example of the `Reflected` trait.
///
/// This type proves the API compiles and works end-to-end. It is also used in
/// the crate's test suite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorStop {
    /// Normalised position along the gradient axis (0.0 = start, 1.0 = end).
    pub position: f32,
    /// Optional human-readable label for the inspector.
    pub label: String,
}

impl ColorStop {
    /// Construct a new `ColorStop` with the given position and label.
    pub fn new(position: f32, label: impl Into<String>) -> Self {
        Self {
            position,
            label: label.into(),
        }
    }
}

impl Reflected for ColorStop {
    fn type_meta() -> &'static TypeMeta {
        // Static schema for ColorStop. Lives in the binary's rodata.
        static FIELDS: [FieldMeta; 2] = [
            FieldMeta {
                name: "position",
                kind: FieldKind::Float,
                doc: "Normalised position along the gradient axis (0.0–1.0).",
                read_only: false,
            },
            FieldMeta {
                name: "label",
                kind: FieldKind::String,
                doc: "Human-readable label shown in the gradient editor.",
                read_only: false,
            },
        ];
        static META: TypeMeta = TypeMeta {
            type_name: "ColorStop",
            fields: &FIELDS,
        };
        &META
    }

    /// Get a field value encoded as `serde_json::Value`.
    ///
    /// - `"position"` → `Value::Number` (f64)
    /// - `"label"` → `Value::String`
    /// - anything else → `None`
    fn get_field(&self, name: &str) -> Option<Value> {
        match name {
            "position" => serde_json::to_value(self.position).ok(),
            "label" => Some(Value::String(self.label.clone())),
            _ => None,
        }
    }

    /// Set a field from a `serde_json::Value`.
    ///
    /// # Errors
    /// Returns `ReflectError::TypeMismatch` if the value is not the right JSON
    /// variant for the field. Returns `ReflectError::FieldNotFound` for unknown names.
    fn set_field(&mut self, name: &str, value: Value) -> Result<(), ReflectError> {
        match name {
            "position" => {
                let v = value
                    .as_f64()
                    .ok_or_else(|| ReflectError::TypeMismatch {
                        field: "position".into(),
                        expected: "number".into(),
                        got: json_kind_name(&value).into(),
                    })?;
                self.position = v as f32;
                Ok(())
            }
            "label" => {
                let v = value
                    .as_str()
                    .ok_or_else(|| ReflectError::TypeMismatch {
                        field: "label".into(),
                        expected: "string".into(),
                        got: json_kind_name(&value).into(),
                    })?;
                self.label = v.to_string();
                Ok(())
            }
            other => Err(ReflectError::FieldNotFound(other.to_string())),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Return a human-readable name for the variant of a `serde_json::Value`.
///
/// Used in `TypeMismatch` error messages so they are actionable without
/// requiring callers to know the JSON type system.
fn json_kind_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── FieldMeta ──────────────────────────────────────────────────────────

    #[test]
    fn field_meta_construction() {
        let f = FieldMeta {
            name: "opacity",
            kind: FieldKind::Float,
            doc: "Alpha channel",
            read_only: false,
        };
        assert_eq!(f.name, "opacity");
        assert_eq!(f.kind, FieldKind::Float);
        assert!(!f.read_only);
    }

    #[test]
    fn field_meta_read_only_flag() {
        let f = FieldMeta {
            name: "id",
            kind: FieldKind::Integer,
            doc: "Immutable node ID",
            read_only: true,
        };
        assert!(f.read_only);
    }

    // ── TypeMeta ───────────────────────────────────────────────────────────

    #[test]
    fn type_meta_field_lookup_found() {
        let meta = ColorStop::type_meta();
        let field = meta.field("position");
        assert!(field.is_some());
        assert_eq!(field.unwrap().kind, FieldKind::Float);
    }

    #[test]
    fn type_meta_field_lookup_missing() {
        let meta = ColorStop::type_meta();
        assert!(meta.field("nonexistent").is_none());
    }

    #[test]
    fn type_meta_field_names() {
        let meta = ColorStop::type_meta();
        let names: Vec<&str> = meta.field_names().collect();
        assert_eq!(names, vec!["position", "label"]);
    }

    #[test]
    fn type_meta_type_name() {
        assert_eq!(ColorStop::type_meta().type_name, "ColorStop");
    }

    // ── Reflected::get_field ───────────────────────────────────────────────

    #[test]
    fn get_field_position() {
        let cs = ColorStop::new(0.5, "mid");
        let v = cs.get_field("position").expect("field exists");
        // serde_json encodes f32 as a Number
        assert!(v.is_number());
        let n = v.as_f64().unwrap();
        assert!((n - 0.5).abs() < 1e-5);
    }

    #[test]
    fn get_field_label() {
        let cs = ColorStop::new(0.0, "start");
        let v = cs.get_field("label").expect("field exists");
        assert_eq!(v, json!("start"));
    }

    #[test]
    fn get_field_unknown_returns_none() {
        let cs = ColorStop::new(0.0, "x");
        assert!(cs.get_field("bogus").is_none());
    }

    // ── Reflected::set_field ───────────────────────────────────────────────

    #[test]
    fn set_field_position_success() {
        let mut cs = ColorStop::new(0.0, "test");
        cs.set_field("position", json!(0.75)).unwrap();
        assert!((cs.position - 0.75).abs() < 1e-5);
    }

    #[test]
    fn set_field_label_success() {
        let mut cs = ColorStop::new(0.0, "old");
        cs.set_field("label", json!("new")).unwrap();
        assert_eq!(cs.label, "new");
    }

    #[test]
    fn set_field_unknown_returns_not_found() {
        let mut cs = ColorStop::new(0.0, "x");
        let err = cs.set_field("no_such", json!(1)).unwrap_err();
        assert_eq!(err, ReflectError::FieldNotFound("no_such".to_string()));
    }

    #[test]
    fn set_field_type_mismatch_position() {
        let mut cs = ColorStop::new(0.0, "x");
        let err = cs.set_field("position", json!("not_a_number")).unwrap_err();
        match err {
            ReflectError::TypeMismatch { field, expected, got } => {
                assert_eq!(field, "position");
                assert_eq!(expected, "number");
                assert_eq!(got, "string");
            }
            other => panic!("expected TypeMismatch, got {:?}", other),
        }
    }

    #[test]
    fn set_field_type_mismatch_label() {
        let mut cs = ColorStop::new(0.0, "x");
        let err = cs.set_field("label", json!(42)).unwrap_err();
        match err {
            ReflectError::TypeMismatch { field, .. } => assert_eq!(field, "label"),
            other => panic!("expected TypeMismatch, got {:?}", other),
        }
    }

    // ── Reflected::snapshot ────────────────────────────────────────────────

    #[test]
    fn snapshot_contains_all_fields() {
        let cs = ColorStop::new(0.25, "quarter");
        let snap = cs.snapshot();
        assert!(snap.contains_key("position"));
        assert!(snap.contains_key("label"));
        assert_eq!(snap.len(), 2);
    }

    #[test]
    fn snapshot_values_are_correct() {
        let cs = ColorStop::new(1.0, "end");
        let snap = cs.snapshot();
        assert_eq!(snap["label"], json!("end"));
        let pos = snap["position"].as_f64().unwrap();
        assert!((pos - 1.0).abs() < 1e-5);
    }

    // ── ReflectRegistry ────────────────────────────────────────────────────

    #[test]
    fn registry_register_and_get() {
        let mut reg = ReflectRegistry::new();
        reg.register(ColorStop::type_meta());
        let found = reg.get("ColorStop");
        assert!(found.is_some());
        assert_eq!(found.unwrap().type_name, "ColorStop");
    }

    #[test]
    fn registry_get_missing_returns_none() {
        let reg = ReflectRegistry::new();
        assert!(reg.get("NonExistent").is_none());
    }

    #[test]
    fn registry_type_names_contains_registered() {
        let mut reg = ReflectRegistry::new();
        reg.register(ColorStop::type_meta());
        let names: Vec<&&str> = reg.type_names().collect();
        assert!(names.contains(&&"ColorStop"));
    }

    #[test]
    fn registry_double_register_is_safe() {
        let mut reg = ReflectRegistry::new();
        reg.register(ColorStop::type_meta());
        reg.register(ColorStop::type_meta()); // should not panic or duplicate
        let count = reg.type_names().count();
        assert_eq!(count, 1);
    }

    // ── ReflectError Display ───────────────────────────────────────────────

    #[test]
    fn reflect_error_display_not_found() {
        let e = ReflectError::FieldNotFound("foo".into());
        assert_eq!(e.to_string(), "reflect: field 'foo' not found");
    }

    #[test]
    fn reflect_error_display_read_only() {
        let e = ReflectError::ReadOnly("bar".into());
        assert_eq!(e.to_string(), "reflect: field 'bar' is read-only");
    }

    #[test]
    fn reflect_error_display_type_mismatch() {
        let e = ReflectError::TypeMismatch {
            field: "x".into(),
            expected: "number".into(),
            got: "string".into(),
        };
        let s = e.to_string();
        assert!(s.contains("type mismatch"));
        assert!(s.contains("'x'"));
        assert!(s.contains("number"));
        assert!(s.contains("string"));
    }

    #[test]
    fn reflect_error_is_std_error() {
        // Verify the Error impl compiles and the source chain is available.
        let e: Box<dyn std::error::Error> =
            Box::new(ReflectError::FieldNotFound("z".into()));
        assert!(e.source().is_none());
    }

    // ── FieldKind variants ─────────────────────────────────────────────────

    #[test]
    fn field_kind_custom() {
        let k = FieldKind::Custom("NodeTransform");
        assert_eq!(k, FieldKind::Custom("NodeTransform"));
        assert_ne!(k, FieldKind::Custom("Other"));
    }

    #[test]
    fn field_kind_copy() {
        let k = FieldKind::Color;
        let k2 = k; // should compile — Copy is derived
        assert_eq!(k, k2);
    }
}
