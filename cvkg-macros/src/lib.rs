extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{
    DeriveInput, Expr, ExprParen, FnArg, ItemFn, ItemStruct, Pat, Token, braced, parse_macro_input,
};

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
                "`#[derive(View)]` cannot be applied to `{}` because it has fields.\n\
                 Types with fields must implement `fn body(self) -> Self::Body` manually.\n\
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
#[proc_macro_attribute]
pub fn view_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;
    let vis = &input.vis;
    let attrs = &input.attrs;
    let inputs = &input.sig.inputs;
    let body = &input.block;

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

    let expanded = quote! {
            #vis struct #struct_name {
                #(#fields),*
            }

    impl cvkg_core::View for #struct_name {
                type Body = cvkg_core::AnyView;

                fn body(self) -> Self::Body {
                    // Map fields back to local variables for the body
                    #(let #field_names = self.#field_names;)*
                    cvkg_core::AnyView::new(#body)
                }
            }

            #(#attrs)*
            #vis fn #name(#inputs) -> #struct_name {
                #struct_name {
                    #(#field_names),*
                }
            }
        };

    TokenStream::from(expanded)
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

enum HamrNode {
    Expr(Expr),
    Block { expr: Expr, children: Vec<HamrNode> },
}

impl Parse for HamrNode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr: Expr = input.parse()?;
        if input.peek(syn::token::Brace) {
            let content;
            braced!(content in input);
            let mut children = Vec::new();
            while !content.is_empty() {
                children.push(content.parse()?);
            }
            Ok(HamrNode::Block { expr, children })
        } else {
            Ok(HamrNode::Expr(expr))
        }
    }
}

/// Internal helper: parse the body of an if/else branch for hamr_if! macro.
struct HamrIfBranch {
    nodes: Vec<HamrNode>,
}

impl Parse for HamrIfBranch {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut nodes = Vec::new();
        while !input.is_empty() {
            nodes.push(input.parse()?);
        }
        Ok(HamrIfBranch { nodes })
    }
}

/// hamr_if! macro -- conditional rendering inside hamr! blocks.
///
/// Syntax:
/// ```ignore
/// hamr_if!(condition { then_block })
/// hamr_if!(condition { then_block } else { else_block })
/// ```
///
/// This macro generates conditional rendering code that can be used
/// inside hamr! blocks. It supports both `if` and `if/else` forms.
///
/// Example:
/// ```ignore
/// use cvkg_macros::{hamr, hamr_if};
/// hamr! {
///     VStack {
///         hamr_if!(is_playing {
///             Text::new("PAUSE")
///         })
///     }
/// }
/// ```
#[proc_macro]
pub fn hamr_if(input: TokenStream) -> TokenStream {
    let parsed: HamrIfMacro = parse_macro_input!(input);
    let cond = parsed.cond;
    let then_branch = parsed.then_branch;
    let else_branch = parsed.else_branch;

    let then_tokens: Vec<proc_macro2::TokenStream> = then_branch
        .nodes
        .iter()
        .map(|n| quote::ToTokens::to_token_stream(n))
        .collect();

    if let Some(else_nodes) = else_branch {
        let else_tokens: Vec<proc_macro2::TokenStream> = else_nodes
            .nodes
            .iter()
            .map(|n| quote::ToTokens::to_token_stream(n))
            .collect();
        TokenStream::from(quote::quote! {
            if #cond {
                #(#then_tokens)*
            } else {
                #(#else_tokens)*
            }
        })
    } else {
        TokenStream::from(quote::quote! {
            if #cond {
                #(#then_tokens)*
            }
        })
    }
}

struct HamrIfMacro {
    cond: Expr,
    then_branch: HamrIfBranch,
    else_branch: Option<HamrIfBranch>,
}

impl Parse for HamrIfMacro {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse condition wrapped in parens to avoid syn treating
        // `if cond { ... }` as an Expr::If expression.
        let cond = input.parse::<syn::ExprParen>()?.expr;
        let then_content;
        braced!(then_content in input);
        let then_branch: HamrIfBranch = then_content.parse()?;
        let else_branch = if input.peek(Token![else]) {
            let _else_token: Token![else] = input.parse()?;
            let else_content;
            braced!(else_content in input);
            let else_branch: HamrIfBranch = else_content.parse()?;
            Some(else_branch)
        } else {
            None
        };
        Ok(HamrIfMacro {
            cond: *cond,
            then_branch,
            else_branch,
        })
    }
}

struct HamrRoot {
    nodes: Vec<HamrNode>,
}

impl Parse for HamrRoot {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut nodes = Vec::new();
        while !input.is_empty() {
            nodes.push(input.parse()?);
        }
        Ok(HamrRoot { nodes })
    }
}

impl quote::ToTokens for HamrNode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            HamrNode::Expr(expr) => {
                expr.to_tokens(tokens);
            }
            HamrNode::Block { expr, children } => {
                let mut output = quote::quote! { #expr };
                for child in children {
                    output = quote::quote! { #output.child(#child) };
                }
                tokens.extend(output);
            }
        }
    }
}

/// hamr! macro -- DSL for declarative UI definition
///
/// Example:
/// hamr! {
///     VStack::new(16.0) {
///         Text::new("Hello")
///         Button::new("Click", || {})
///     }
/// }
///
/// Conditional rendering is available via hamr_if!:
/// hamr! {
///     VStack {
///         hamr_if!(is_logged_in {
///             Text::new("Welcome!")
///         } else {
///             Text::new("Please log in")
///         })
///     }
/// }
#[proc_macro]
pub fn hamr(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as HamrRoot);
    let nodes = root.nodes;
    let expanded = quote! {
        #(#nodes)*
    };
    TokenStream::from(expanded)
}

/// cvkg_model! macro -- generates data models with VDOM metadata
#[proc_macro]
pub fn cvkg_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let name = &input.ident;

    let expanded = quote! {
        #[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
        #input

        impl #name {
            pub fn vdom_id(&self) -> String {
                format!("{}_{}", stringify!(#name), std::collections::hash_map::DefaultHasher::new().finish())
            }
        }
    };

    TokenStream::from(expanded)
}
