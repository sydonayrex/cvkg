//! Composed WGSL program assembly — single source of truth (P0-3).
//!
//! The GPU renderer does not compile individual `.wgsl` files. It concatenates
//! a shared preamble (`common.wgsl`) with a material/stage body, several
//! effect modules and the runtime-generated material dispatch functions into a
//! *single* shader module per pipeline. A file that parses on its own can still
//! produce an invalid composed program (duplicate symbols, missing bindings,
//! conflicting `enable` directives), so validating individual files is not
//! enough.
//!
//! Keeping the concatenation in one place serves two purposes:
//!
//! 1. `renderer/init.rs` uses [`composed_variants`] to build exactly the strings
//!    handed to `create_shader_module`, so the validated output cannot drift
//!    from the runtime output.
//! 2. Tests can compose both the native and the wasm shader sets on the host
//!    and feed every variant to the naga frontend + validator.
//!
//! WGSL is the source of truth. There are no hand-authored SPIR-V artifacts in
//! this repository; see `docs/quality/shader-validation.md` for the explicit
//! backend-support statement.

/// The WGSL source files that make up one platform's shader set.
///
/// Three files have separate wasm variants because WebGL2/WebGPU on wasm32
/// cannot use 32-element texture binding arrays (see the `cfg` block in
/// `lib.rs` around the `WGSL_*` constants). All other files are shared.
pub(crate) struct ShaderSet {
    pub name: &'static str,
    pub common: &'static str,
    pub shapes: &'static str,
    pub opaque: &'static str,
    pub glass: &'static str,
    pub pbr: &'static str,
    pub shadow: &'static str,
    pub bifrost: &'static str,
    pub bloom: &'static str,
    pub color_blind: &'static str,
}

/// The shader set selected by the active compile target — this is what the
/// renderer actually compiles.
pub(crate) fn active_shader_set() -> ShaderSet {
    #[cfg(target_arch = "wasm32")]
    {
        wasm_shader_set()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        native_shader_set()
    }
}

/// Native shader set: `t_diffuse` is a 32-element `binding_array`.
pub(crate) fn native_shader_set() -> ShaderSet {
    ShaderSet {
        name: "native",
        common: include_str!("shaders/common.wgsl"),
        shapes: include_str!("shaders/shapes.wgsl"),
        opaque: include_str!("shaders/material_opaque.wgsl"),
        glass: include_str!("shaders/material_glass.wgsl"),
        pbr: include_str!("shaders/material_pbr.wgsl"),
        shadow: include_str!("shaders/material_shadow.wgsl"),
        bifrost: include_str!("shaders/bifrost.wgsl"),
        bloom: include_str!("shaders/bloom.wgsl"),
        color_blind: include_str!("shaders/color_blind.wgsl"),
    }
}

/// wasm shader set: `t_diffuse` is a single texture (WebGL2 binding-array limit).
pub(crate) fn wasm_shader_set() -> ShaderSet {
    ShaderSet {
        name: "wasm",
        common: include_str!("shaders/common_wasm.wgsl"),
        shapes: include_str!("shaders/shapes.wgsl"),
        opaque: include_str!("shaders/material_opaque_wasm.wgsl"),
        glass: include_str!("shaders/material_glass.wgsl"),
        pbr: include_str!("shaders/material_pbr.wgsl"),
        shadow: include_str!("shaders/material_shadow.wgsl"),
        bifrost: include_str!("shaders/bifrost.wgsl"),
        bloom: include_str!("shaders/bloom_wasm.wgsl"),
        color_blind: include_str!("shaders/color_blind.wgsl"),
    }
}

/// One fully composed shader program and its stable label.
pub(crate) struct ComposedVariant {
    pub name: &'static str,
    pub source: String,
}

/// Compose every shader program the renderer creates.
///
/// `materials_generated` is the output of
/// [`crate::material::generate_builtins_wgsl`] (or the safe fallback from
/// [`crate::material::generate_builtins_wgsl_lossy`]). It is appended last so
/// generated functions can call helpers declared in the preamble.
///
/// The order here MUST match the order used by the pipelines; both the runtime
/// and the tests call this function, which is what makes that guarantee
/// mechanically enforceable.
pub(crate) fn compose(set: &ShaderSet, materials_generated: &str) -> [ComposedVariant; 5] {
    let with_effects = |body: &str| -> String {
        format!(
            "{}{}{}{}{}{}",
            set.common, body, set.bifrost, set.bloom, set.color_blind, materials_generated
        )
    };

    [
        ComposedVariant {
            name: "main",
            source: with_effects(set.shapes),
        },
        ComposedVariant {
            name: "opaque",
            source: with_effects(set.opaque),
        },
        ComposedVariant {
            name: "glass",
            source: with_effects(set.glass),
        },
        ComposedVariant {
            name: "pbr",
            source: with_effects(set.pbr),
        },
        ComposedVariant {
            // Shadow pass intentionally omits the effect modules: it only
            // needs the common preamble, the shadow body and material dispatch.
            name: "shadow",
            source: format!("{}{}{}", set.common, set.shadow, materials_generated),
        },
    ]
}

/// Generate the material dispatch WGSL with the texture-binding form that
/// matches `set`.
///
/// This must be used instead of a platform-blind generator: the native
/// `t_diffuse` is a 32-element binding array while the wasm one is a single
/// texture, and generated material functions sample it directly (P0-3).
pub(crate) fn generated_materials_for(set: &ShaderSet) -> String {
    crate::material::generate_builtins_wgsl_lossy_for(set.name == "wasm")
}

/// Compose the variants for the active target, as used by `init.rs`.
pub(crate) fn composed_variants(materials_generated: &str) -> [ComposedVariant; 5] {
    compose(&active_shader_set(), materials_generated)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse and validate a composed program. Individual files are not
    /// standalone-valid by design, so validation must happen on the composed
    /// string — this is the core P0-3 check.
    fn assert_valid_wgsl(label: &str, source: &str) {
        let mut frontend = naga::front::wgsl::Frontend::new();
        let module = match frontend.parse(source) {
            Ok(module) => module,
            Err(error) => panic!(
                "[{label}] composed WGSL failed to parse:\n{}",
                error.emit_to_string(source)
            ),
        };

        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        if let Err(error) = validator.validate(&module) {
            panic!("[{label}] composed WGSL failed naga validation: {error:?}");
        }
    }

    #[test]
    fn native_composed_variants_are_valid() {
        let set = native_shader_set();
        let generated = generated_materials_for(&set);
        assert!(
            generated.contains("fn dispatch_material"),
            "generated material WGSL must contain the dispatch function"
        );
        assert!(
            generated.contains("t_diffuse["),
            "native generated materials must use the texture binding array"
        );
        for variant in compose(&set, &generated) {
            assert_valid_wgsl(&format!("native/{}", variant.name), &variant.source);
        }
    }

    #[test]
    fn wasm_composed_variants_are_valid() {
        let set = wasm_shader_set();
        let generated = generated_materials_for(&set);
        assert!(
            !generated.contains("t_diffuse["),
            "wasm generated materials must not index t_diffuse (single texture)"
        );
        for variant in compose(&set, &generated) {
            assert_valid_wgsl(&format!("wasm/{}", variant.name), &variant.source);
        }
    }

    #[test]
    fn generated_material_wgsl_is_in_every_variant() {
        for set in [native_shader_set(), wasm_shader_set()] {
            let generated = generated_materials_for(&set);
            for variant in compose(&set, &generated) {
                assert!(
                    variant.source.contains("fn dispatch_material"),
                    "{}:{} must embed generated material WGSL",
                    set.name,
                    variant.name
                );
            }
        }
    }

    #[test]
    fn native_and_wasm_sets_use_expected_texture_declarations() {
        assert!(
            native_shader_set()
                .common
                .contains("binding_array<texture_2d<f32>, 32>")
        );
        assert!(
            !wasm_shader_set()
                .common
                .contains("binding_array<texture_2d<f32>, 32>")
        );
        assert!(wasm_shader_set().common.contains("t_diffuse:"));
    }

    /// Regression guard: if someone changes the concatenation in `init.rs`
    /// without changing this module, the runtime string and the validated
    /// string diverge. This test pins the composition order explicitly.
    #[test]
    fn composition_order_matches_documented_pipeline_order() {
        let generated = "/*MATERIALS*/";
        let variants = compose(&native_shader_set(), generated);
        let expected_main = format!(
            "{}{}{}{}{}{}",
            include_str!("shaders/common.wgsl"),
            include_str!("shaders/shapes.wgsl"),
            include_str!("shaders/bifrost.wgsl"),
            include_str!("shaders/bloom.wgsl"),
            include_str!("shaders/color_blind.wgsl"),
            generated
        );
        assert_eq!(variants[0].source, expected_main);
        let expected_shadow = format!(
            "{}{}{}",
            include_str!("shaders/common.wgsl"),
            include_str!("shaders/material_shadow.wgsl"),
            generated
        );
        assert_eq!(variants[4].source, expected_shadow);
    }
}
