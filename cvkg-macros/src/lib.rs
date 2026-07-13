extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{DeriveInput, FnArg, ItemFn, ItemStruct, Pat, Token, parse_macro_input};

/// State attribute macro -- derives common traits for state structs
///
/// Section 4.2: "expressed as Rust attributes via procedural macros"
/// Automates: Clone, Debug, Default, Serialize, Deserialize
#[proc_macro_attribute]
pub fn state(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let expanded = quote! {
        #[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
        #input
    };

    TokenStream::from(expanded)
}

/// View derive macro -- automatically implements cvkg_core::View
///
/// If the struct has a `body` method defined in an `impl` block, it will be used.
/// Otherwise, it defaults to a primitive View (Body = Never).
///
/// # Warning
/// `#[derive(View)]` generates `Body = Never` with `body()` panicking at runtime.
/// You MUST implement `fn body(self) -> Self::Body` in a separate `impl View for MyType` block
/// if your view has children or content. Use `#[derive(View)]` only for leaf/primitive views
/// where `body()` is never called (e.g., simple wrappers that render via their View trait).
///
/// # Compile-time validation
/// Applying `#[derive(View)]` to a struct with fields is a compile error:
///
/// ```compile_fail
/// use cvkg_macros::View;
/// #[derive(View)]
/// struct BadView {
///     x: f32,
/// }
/// ```
///
/// Unit structs and empty structs are accepted:
///
/// ```
/// use cvkg_macros::View;
/// #[derive(View)]
/// struct GoodLeafView;
/// ```
#[proc_macro_derive(View)]
pub fn derive_view(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Compile-time check: reject structs with fields at derive time.
    // Types with fields always need a `body` method to describe their children.
    let has_fields = match &input.data {
        syn::Data::Struct(data) => !data.fields.is_empty(),
        _ => true,
    };

    if has_fields {
        return syn::Error::new(
            name.span(),
            format!(
                "`#[derive(View)]` cannot be applied to `{}` because it has fields.\n \
                 Types with fields must implement `fn body(self) -> Self::Body` manually.\n \
                 Use `#[derive(View)]` only for leaf/primitive views with no fields.",
                name
            ),
        )
        .to_compile_error()
        .into();
    }

    let expanded = quote! {
        impl cvkg_core::View for #name {
            type Body = cvkg_core::Never;
            fn body(self) -> Self::Body {
                // SAFETY: `Never` is uninhabitable. `body()` is only called on views
                // that have children. Leaf views (no fields) never call body().
                unreachable!()
            }
        }
    };

    TokenStream::from(expanded)
}

/// View component attribute macro -- transforms a function into a View struct
///
/// Section 4.1: "automate the boilerplate... generating the View trait implementation"
/// Supports `#[require(Type1, Type2)]` attribute on the function to auto-generate companion state initialization.
#[proc_macro_attribute]
pub fn view_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;
    let vis = &input.vis;
    let attrs = &input.attrs;
    let inputs = &input.sig.inputs;
    let body = &input.block;

    // Parse the #[require(...)] attribute from the function's attributes
    let mut required_types = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("require") {
            let parsed = attr.parse_args::<RequireAttr>();
            if let Ok(req) = parsed {
                required_types = req.types;
            }
        }
    }

    // Extract argument names and types for the struct fields
    let mut fields = Vec::new();
    let mut field_names = Vec::new();

    for arg in inputs {
        if let FnArg::Typed(pat_type) = arg
            && let Pat::Ident(pat_ident) = &*pat_type.pat
        {
            let arg_name = &pat_ident.ident;
            let arg_type = &pat_type.ty;
            fields.push(quote! { pub #arg_name: #arg_type });
            field_names.push(arg_name);
        }
    }

    let mut name_str = name.to_string();
    if let Some(first) = name_str.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    let struct_name = quote::format_ident!("{}View", name_str);

    // Generate companion_states() method if there are required types
    let companion_states_impl = if required_types.is_empty() {
        quote! {
            fn companion_states(&self) -> Vec<Box<dyn cvkg_core::Companion>> {
                vec![]
            }
        }
    } else {
        let required_types_ref = &required_types;
        quote! {
            fn companion_states(&self) -> Vec<Box<dyn cvkg_core::Companion>> {
                vec![
                    #(Box::new(#required_types_ref::default())),*
                ]
            }
        }
    };

    // Generate a unique instance-id counter name for this component type.
    let counter_name = quote::format_ident!("__{}_instance_counter", name_str.to_ascii_lowercase());

    let expanded = quote! {
            // Per-instance counter — each instance of this component type gets a
            // unique tracking ID so that signal→component dependency edges are
            // per-instance, not per-type.
            #[allow(non_upper_case_globals)]
            static #counter_name: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

            #vis struct #struct_name {
                __instance_id: u64,
                #(#fields),*
            }

    impl cvkg_core::View for #struct_name {
                type Body = cvkg_core::AnyView;

                fn body(self) -> Self::Body {
                    // Map fields back to local variables for the body
                    #(let #field_names = self.#field_names;)*

                    // Per-instance tracking ID avoids the N-instances-1-reactive bug
                    // where all instances share a single hash-of-type-name id.
                    let __tracking_id = self.__instance_id;

                    cvkg_vdom::signals::begin_tracking(__tracking_id);
                    let __result = cvkg_core::AnyView::new(#body);
                    let __reads = cvkg_vdom::signals::end_tracking();

                    for __signal_id in __reads {
                        cvkg_vdom::signals::dependency_graph().write().unwrap().register(__tracking_id, __signal_id);
                    }

                    __result
                }

                #companion_states_impl
            }

            #(#attrs)*
            #vis fn #name(#inputs) -> #struct_name {
                #struct_name {
                    __instance_id: #counter_name.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                    #(#field_names),*
                }
            }
        };

    TokenStream::from(expanded)
}

/// Helper struct for parsing `#[require(Type1, Type2)]` attribute.
struct RequireAttr {
    types: Vec<syn::Type>,
}

impl Parse for RequireAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut types = Vec::new();
        while !input.is_empty() {
            let ty: syn::Type = input.parse()?;
            types.push(ty);
            if input.is_empty() {
                break;
            }
            let _: Option<Token![,]> = input.parse()?;
        }
        Ok(Self { types })
    }
}

/// Binding attribute macro -- marks a struct as a reactive binding
///
/// Section 4.2: "Binding -- read/write reference to parent state"
/// This macro derives serialization traits for debug inspection.
#[proc_macro_attribute]
pub fn binding(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let expanded = quote! {
        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
        #input
    };

    TokenStream::from(expanded)
}

/// Require attribute -- marks companion types for a view component.
/// This attribute is parsed by `#[view_component]` and has no effect on its own.
#[proc_macro_attribute]
pub fn require(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item // Pass through unchanged
}

/// Component attribute macro -- generates a component with builder pattern
///
/// Section 7.2: "Reduce component boilerplate"
/// Generates: struct, View impl, builder pattern, and modifier-chain scaffolding
/// Target: a minimal component should be expressible in ~10 lines, not ~40.
#[proc_macro_attribute]
pub fn cvkg_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let name = &input.ident;
    let vis = &input.vis;

    // Extract fields from the struct
    let mut fields = Vec::new();
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();

    match &input.fields {
        syn::Fields::Named(fields_named) => {
            for field in &fields_named.named {
                if let Some(ident) = &field.ident {
                    let ty = &field.ty;
                    fields.push(quote! { #ident: #ty });
                    field_names.push(quote! { #ident });
                    field_types.push(quote! { #ty });
                }
            }
        }
        syn::Fields::Unnamed(fields_unnamed) => {
            for (i, field) in fields_unnamed.unnamed.iter().enumerate() {
                let ident = quote::format_ident!("_{}", i);
                let ty = &field.ty;
                fields.push(quote! { #ident: #ty });
                field_names.push(quote! { #ident });
                field_types.push(quote! { #ty });
            }
        }
        syn::Fields::Unit => {
            // unit struct
        }
    }

    // Builder struct
    let builder_name = quote::format_ident!("{}Builder", name);

    // Generate the expanded code
    let expanded = quote! {
        #vis struct #name {
            #(#fields),*
        }

        impl #name {
            /// Create a new builder for this component
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#field_names: Default::default(),)*
                }
            }
        }

        #vis struct #builder_name {
            #(#field_names: Option<#field_types>),*
        }

        impl #builder_name {
            #(
                pub fn #field_names(mut self, value: #field_types) -> Self {
                    self.#field_names = Some(value);
                    self
                }
            )*

            /// # Panics
            /// Panics if any required field was not set via its setter method.
            /// Use the `try_build()` variant (if available) for non-panicking error handling.
            pub fn build(self) -> #name {
                #name {
                    #(#field_names: self.#field_names.expect(
                        concat!("missing required field: ", stringify!(#field_names))
                    ),)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Frame manifest merge macro — evaluates const expressions at compile time.
/// Validates: duplicate pass IDs, unresolved `after` refs, cycles, budget overruns.
#[proc_macro]
pub fn merge_manifests(input: TokenStream) -> TokenStream {
    let manifests: syn::punctuated::Punctuated<syn::Expr, syn::token::Comma> = parse_macro_input!(input with syn::punctuated::Punctuated::<syn::Expr, syn::token::Comma>::parse_terminated);
    let manifests: Vec<syn::Expr> = manifests.into_iter().collect();

    // Build the expanded macro that validates at compile time
    let mut expanded = proc_macro2::TokenStream::new();

    // Add all the manifest expressions as statements
    for manifest in &manifests {
        expanded.extend(quote::quote! {
            #manifest;
        });
    }

    // Add compile-time validation
    expanded.extend(quote::quote! {
        const _: () = {
            cvkg_core::frame_manifest::validate_manifests(&[
                #(&#manifests),*
            ]);
        };
        
        // Build the merged manifest reference
        pub static _MERGED_MANIFEST: cvkg_core::frame_manifest::FrameManifest = cvkg_core::frame_manifest::build_merged_manifest(&[
            #(&#manifests),*
        ]);
    });

    TokenStream::from(expanded)
}

/// Component metadata derive macro — generates `ComponentMeta` for gallery auto-registration.
///
/// # Supported attributes
/// - `#[component_meta(category = "Forms", description = "Button description")]`
/// - `#[component_prop(name = "label", type_name = "string", default = "\"Click Me\"", description = "Button text")]`
///
/// # Example
/// ```ignore
/// #[derive(ComponentMeta)]
/// #[component_meta(category = "Forms", description = "Clickable button")]
/// struct Button {
///     #[component_prop(type_name = "string", default = "\"Click Me\"", description = "Button text")]
///     label: String,
///     #[component_prop(type_name = "bool", default = "false", description = "Disable button")]
///     disabled: bool,
/// }
/// ```
#[proc_macro_derive(ComponentMeta, attributes(component_meta, component_prop))]
pub fn derive_component_meta(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();

    // Parse #[component_meta(...)] attribute on the struct
    let mut category = "Uncategorized".to_string();
    let mut description = String::new();

    for attr in &input.attrs {
        if attr.path().is_ident("component_meta") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("category") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    category = value.value();
                } else if meta.path.is_ident("description") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    description = value.value();
                }
                Ok(())
            });
        }
    }

    // Extract fields with #[component_prop(...)] attributes
    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => {
            return syn::Error::new(name.span(), "ComponentMeta can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    let named_fields = match fields {
        syn::Fields::Named(f) => &f.named,
        _ => {
            return syn::Error::new(name.span(), "ComponentMeta requires named fields")
                .to_compile_error()
                .into();
        }
    };

    let mut prop_metas = Vec::new();

    for field in named_fields {
        let field_name = field.ident.as_ref().unwrap().to_string();
        let field_ty = &field.ty;

        // Parse #[component_prop(...)] attribute
        let mut prop_type = quote!(#field_ty).to_string();
        let mut default = String::new();
        let mut prop_description = String::new();

        for attr in &field.attrs {
            if attr.path().is_ident("component_prop") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("type_name") {
                        let value: syn::LitStr = meta.value()?.parse()?;
                        prop_type = value.value();
                    } else if meta.path.is_ident("default") {
                        let value: syn::LitStr = meta.value()?.parse()?;
                        default = value.value();
                    } else if meta.path.is_ident("description") {
                        let value: syn::LitStr = meta.value()?.parse()?;
                        prop_description = value.value();
                    }
                    Ok(())
                });
            }
        }

        prop_metas.push(quote! {
            crate::ComponentPropMeta {
                name: #field_name,
                type_name: #prop_type,
                default: #default,
                description: #prop_description,
            }
        });
    }

    let expanded = quote! {
        impl #name {
            /// Returns component metadata for gallery auto-registration
            pub fn component_meta() -> crate::ComponentMeta {
                crate::ComponentMeta {
                    name: #name_str,
                    category: #category,
                    description: #description,
                    props: vec![#(#prop_metas),*],
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Supporting types for component metadata
/// Note: These are re-exported from cvkg-components for use in the macro output
// These types are actually defined in cvkg-components to be shared
// They're referenced here for documentation purposes only
// The actual types live in cvkg-components/src/registry.rs
/*
mod component_meta_types {
    use std::borrow::Cow;

    /// Metadata for a single component property
    pub struct ComponentPropMeta {
        pub name: &'static str,
        pub type_name: &'static str,
        pub default: &'static str,
        pub description: &'static str,
    }

    /// Full component metadata for gallery registration
    pub struct ComponentMeta {
        pub name: &'static str,
        pub category: &'static str,
        pub description: &'static str,
        pub props: Vec<ComponentPropMeta>,
    }

    impl ComponentMeta {
        pub fn new(name: &'static str, category: &'static str, description: &'static str) -> Self {
            Self { name, category, description, props: Vec::new() }
        }

        pub fn with_props(mut self, props: Vec<ComponentPropMeta>) -> Self {
            self.props = props;
            self
        }
    }
}
*/

/// Reflect derive macro — generates a `cvkg_reflect::Reflected` implementation.
///
/// # Supported attributes
/// - `#[reflect(kind = "Vec3")]` — override inferred FieldKind
/// - `#[reflect(doc = "description")]` — set field documentation
/// - `#[reflect(read_only)]` — mark field as read-only
///
/// # Example
/// ```ignore
/// #[derive(Reflect)]
/// struct MyStruct {
///     enabled: bool,
///     #[reflect(kind = "Vec3", doc = "Light direction")]
///     light_dir: [f32; 3],
/// }
/// ```
#[proc_macro_derive(Reflect, attributes(reflect))]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();

    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => {
            return syn::Error::new(name.span(), "Reflect can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    let named_fields = match fields {
        syn::Fields::Named(f) => &f.named,
        _ => {
            return syn::Error::new(name.span(), "Reflect requires named fields")
                .to_compile_error()
                .into();
        }
    };

    let field_count = named_fields.len();

    let mut field_metas = Vec::new();
    let mut get_arms = Vec::new();
    let mut set_arms = Vec::new();

    for field in named_fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let ty = &field.ty;

        let mut kind_override = None;
        let mut doc_override = None;
        let mut read_only = false;

        for attr in &field.attrs {
            if attr.path().is_ident("reflect") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("kind") {
                        let value = meta.value()?;
                        let s: syn::LitStr = value.parse()?;
                        kind_override = Some(s.value());
                        Ok(())
                    } else if meta.path.is_ident("doc") {
                        let value = meta.value()?;
                        let s: syn::LitStr = value.parse()?;
                        doc_override = Some(s.value());
                        Ok(())
                    } else if meta.path.is_ident("read_only") {
                        read_only = true;
                        Ok(())
                    } else {
                        Ok(())
                    }
                });
            }
        }

        let kind_str = kind_override.unwrap_or_else(|| type_to_kind(ty));
        let kind_tokens = match kind_str.as_str() {
            "Bool" => quote! { cvkg_reflect::FieldKind::Bool },
            "Integer" => quote! { cvkg_reflect::FieldKind::Integer },
            "Float" => quote! { cvkg_reflect::FieldKind::Float },
            "String" => quote! { cvkg_reflect::FieldKind::String },
            "Color" => quote! { cvkg_reflect::FieldKind::Color },
            "Vec2" => quote! { cvkg_reflect::FieldKind::Vec2 },
            "Vec3" => quote! { cvkg_reflect::FieldKind::Vec3 },
            "Rect" => quote! { cvkg_reflect::FieldKind::Rect },
            other => quote! { cvkg_reflect::FieldKind::Custom(#other) },
        };

        let doc_str = doc_override.unwrap_or_default();

        field_metas.push(quote! {
            cvkg_reflect::FieldMeta {
                name: #field_name_str,
                kind: #kind_tokens,
                doc: #doc_str,
                read_only: #read_only,
            }
        });

        let get_tokens = match kind_str.as_str() {
            "Bool" => quote! { Some(serde_json::Value::Bool(self.#field_name)) },
            "Integer" => quote! { serde_json::to_value(self.#field_name).ok() },
            "Float" => quote! { serde_json::to_value(self.#field_name).ok() },
            "String" => quote! { Some(serde_json::Value::String(self.#field_name.clone())) },
            _ => quote! { serde_json::to_value(self.#field_name).ok() },
        };
        get_arms.push(quote! {
            #field_name_str => #get_tokens,
        });

        let set_conversion = match kind_str.as_str() {
            "Bool" => quote! {
                let v = value.as_bool().ok_or_else(|| cvkg_reflect::ReflectError::TypeMismatch {
                    field: #field_name_str.into(),
                    expected: "bool".into(),
                    got: value.to_string(),
                })?;
                self.#field_name = v;
            },
            "Integer" => quote! {
                let v = value.as_i64().ok_or_else(|| cvkg_reflect::ReflectError::TypeMismatch {
                    field: #field_name_str.into(),
                    expected: "integer".into(),
                    got: value.to_string(),
                })?;
                self.#field_name = v as _;
            },
            "Float" => quote! {
                let v = value.as_f64().ok_or_else(|| cvkg_reflect::ReflectError::TypeMismatch {
                    field: #field_name_str.into(),
                    expected: "number".into(),
                    got: value.to_string(),
                })?;
                self.#field_name = v as f32;
            },
            "String" => quote! {
                let v = value.as_str().ok_or_else(|| cvkg_reflect::ReflectError::TypeMismatch {
                    field: #field_name_str.into(),
                    expected: "string".into(),
                    got: value.to_string(),
                })?;
                self.#field_name = v.to_string();
            },
            _ => quote! {
                self.#field_name = serde_json::from_value(value).map_err(|e| {
                    cvkg_reflect::ReflectError::TypeMismatch {
                        field: #field_name_str.into(),
                        expected: "compatible value".into(),
                        got: e.to_string(),
                    }
                })?;
            },
        };

        let read_only_lit = read_only;
        set_arms.push(quote! {
            #field_name_str => {
                if #read_only_lit {
                    return Err(cvkg_reflect::ReflectError::ReadOnly(#field_name_str.into()));
                }
                #set_conversion
                Ok(())
            }
        });
    }

    let expanded = quote! {
        impl cvkg_reflect::Reflected for #name {
            fn type_meta() -> &'static cvkg_reflect::TypeMeta {
                static FIELDS: [cvkg_reflect::FieldMeta; #field_count] = [
                    #(#field_metas),*
                ];
                static META: cvkg_reflect::TypeMeta = cvkg_reflect::TypeMeta {
                    type_name: #name_str,
                    fields: &FIELDS,
                };
                &META
            }

            fn get_field(&self, name: &str) -> Option<serde_json::Value> {
                match name {
                    #(#get_arms)*
                    _ => None,
                }
            }

            fn set_field(&mut self, name: &str, value: serde_json::Value)
                -> Result<(), cvkg_reflect::ReflectError>
            {
                match name {
                    #(#set_arms)*
                    other => Err(cvkg_reflect::ReflectError::FieldNotFound(other.into())),
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Map a Rust type to a FieldKind name string.
fn type_to_kind(ty: &syn::Type) -> String {
    let type_str = quote!(#ty).to_string();
    match type_str.as_str() {
        "bool" => "Bool".to_string(),
        "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64" | "usize" => {
            "Integer".to_string()
        }
        "f32" | "f64" => "Float".to_string(),
        "String" => "String".to_string(),
        "[f32 ; 2]" | "[f32;2]" => "Vec2".to_string(),
        "[f32 ; 3]" | "[f32;3]" => "Vec3".to_string(),
        "[f32 ; 4]" | "[f32;4]" => "Color".to_string(),
        _ => type_str.replace(' ', ""),
    }
}

/// Include the component gallery registration macro
#[cfg(test)]
mod smoke_tests {
    #[test]
    fn test_compiles() {
        // Proc-macro crates cannot unit test macro expansion in-process.
        // This placeholder verifies the crate compiles and the test harness works.
    }
}

/// Component registration attribute macro — generates gallery metadata for auto-registration
///
/// This macro can be applied to a component struct to automatically generate
/// the metadata needed for the gallery registry, including:
/// - Component name, category, description
/// - Prop metadata (name, type, default, description)
/// - Usage example template
///
/// # Example
///
/// ```ignore
/// #[cvkg_gallery_component(
///     name = "Button",
///     category = "Forms",
///     description = "Interactive button with variants and states"
/// )]
/// #[derive(Reflect)]
/// pub struct Button {
///     #[reflect(doc = "Button text")]
///     pub label: String,
///     #[reflect(doc = "Click handler")]
///     pub on_click: Option<Callback<()>>,
///     #[reflect(kind = "enum", doc = "Primary|Secondary|Destructive|Ghost")]
///     pub variant: ButtonVariant,
///     #[reflect(doc = "Disable interaction")]
///     pub disabled: bool,
/// }
/// ```
#[proc_macro_attribute]
pub fn cvkg_gallery_component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    // Parse the attribute arguments
    let args = parse_macro_input!(attr with ComponentGalleryArgs::parse);

    let name = &input.ident;
    let name_str = name.to_string();
    let category = &args.category;
    let description = &args.description;

    // Extract fields with their attributes
    let mut prop_metas = Vec::new();

    if let syn::Fields::Named(fields_named) = &input.fields {
        for field in &fields_named.named {
            let field_name = field.ident.as_ref().unwrap().to_string();
            let field_type_str = quote!(#field.ty).to_string();

            // Parse #[reflect(...)] attributes
            let mut prop_description = String::new();
            let mut prop_kind = None;
            let mut read_only = false;

            for attr in &field.attrs {
                if attr.path().is_ident("reflect") {
                    let _ = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("doc") {
                            let value: syn::LitStr = meta.value()?.parse()?;
                            prop_description = value.value();
                        } else if meta.path.is_ident("kind") {
                            let value: syn::LitStr = meta.value()?.parse()?;
                            prop_kind = Some(value.value());
                        } else if meta.path.is_ident("read_only") {
                            read_only = true;
                        }
                        Ok(())
                    });
                }
            }

            // Determine type name for display
            let field_type_for_display = field_type_str.clone();
            let type_display = if let Some(kind) = prop_kind {
                kind
            } else if field_type_str.contains("bool") {
                "bool".to_string()
            } else if field_type_str.contains("String") {
                "string".to_string()
            } else if field_type_str.contains("f32") || field_type_str.contains("f64") {
                "number".to_string()
            } else if field_type_str.contains("Option") {
                "optional".to_string()
            } else {
                field_type_for_display
            };

            // Generate default value string
            let default = if read_only { "N/A".to_string() } else if field_type_str.contains("bool") { "false".to_string() } else if field_type_str.contains("String") { "\"\"".to_string() } else if field_type_str.contains("f32") || field_type_str.contains("f64") { "0".to_string() } else { "default()".to_string() };

            prop_metas.push(quote! {
                crate::ComponentPropMeta {
                    name: #field_name,
                    type_name: #type_display,
                    default: #default,
                    description: #prop_description,
                }
            });
        }
    }

    let expanded = quote! {
        impl #name {
            /// Returns component metadata for gallery auto-registration
            pub fn component_meta() -> crate::ComponentMeta {
                crate::ComponentMeta {
                    name: #name_str,
                    category: #category,
                    description: #description,
                    props: vec![#(#prop_metas),*],
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Arguments for the cvkg_gallery_component attribute
struct ComponentGalleryArgs {
    name: String,
    category: String,
    description: String,
}

impl Parse for ComponentGalleryArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = String::new();
        let mut category = String::new();
        let mut description = String::new();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            let _: syn::Token![=] = input.parse()?;

            if ident == "name" {
                let value: syn::LitStr = input.parse()?;
                name = value.value();
            } else if ident == "category" {
                let value: syn::LitStr = input.parse()?;
                category = value.value();
            } else if ident == "description" {
                let value: syn::LitStr = input.parse()?;
                description = value.value();
            } else {
                return Err(syn::Error::new_spanned(ident, "unknown argument"));
            }

            if !input.is_empty() {
                let _: syn::token::Comma = input.parse()?;
            }
        }

        Ok(Self { name, category, description })
    }
}

#[cfg(test)]
mod smoke_tests {
    #[test]
    fn test_compiles() {
        // Proc-macro crates cannot unit test macro expansion in-process.
        // This placeholder verifies the crate compiles and the test harness works.
    }
}
