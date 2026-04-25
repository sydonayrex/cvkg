//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     — State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     — Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     — Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    — Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     — Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     — Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   — Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//!   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//!   CVKG Extended: Section 2 of the CVKG Design Specification

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
