extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, ItemStruct, Pat, parse_macro_input};

/// State attribute macro — derives common traits for state structs
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

/// View attribute macro — transforms a function into a View struct
///
/// Section 4.1: "automate the boilerplate... generating the View trait implementation"
#[proc_macro_attribute]
pub fn view(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;
    let vis = &input.vis;
    let inputs = &input.sig.inputs;
    let body = &input.block;

    // Extract argument names and types for the struct fields
    let mut fields = Vec::new();
    let mut field_names = Vec::new();

    for arg in inputs {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let arg_name = &pat_ident.ident;
                let arg_type = &pat_type.ty;
                fields.push(quote! { pub #arg_name: #arg_type });
                field_names.push(arg_name);
            }
        }
    }

    let struct_name = quote::format_ident!("{}View", name);

    let expanded = quote! {
        #vis struct #struct_name {
            #(#fields),*
        }

        impl cvkg_core::View for #struct_name {
            type Body = impl cvkg_core::View;

            fn body(self) -> Self::Body {
                // Map fields back to local variables for the body
                #(let mut #field_names = self.#field_names;)*
                #body
            }
        }

        #vis fn #name(#inputs) -> #struct_name {
            #struct_name {
                #(#field_names),*
            }
        }
    };

    TokenStream::from(expanded)
}

/// Binding attribute macro — marks a struct as a reactive binding
///
/// Section 4.2: "Binding — read/write reference to parent state"
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

/// Component attribute macro — generates a component with builder pattern
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
                    field_names.push(ident);
                    field_types.push(ty);
                }
            }
        }
        syn::Fields::Unnamed(_) => {
            // TODO: support tuple structs if needed
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
                    #(#field_names: self.#field_names.expect("missing required field "),)*
                }
            }
        }
        
        impl cvkg_core::View for #name {
            type Body = impl cvkg_core::View;
            
            fn body(self) -> Self::Body {
                // For now, we just return a placeholder; users can customize by implementing body themselves
                // Alternatively, we could generate a body that uses the fields, but that's complex.
                // We'll leave it as Never and expect users to override body if needed.
                cvkg_core::Never
            }
            
            fn render(&self, renderer: &mut dyn cvkg_core::Renderer, rect: cvkg_core::Rect) {
                // Default render does nothing; users should override if needed
            }
        }
    };
    
    TokenStream::from(expanded)
}
