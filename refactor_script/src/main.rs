use std::fs;
use syn::{Item, ImplItem, spanned::Spanned};

fn main() {
    let content = fs::read_to_string("../cvkg-render-gpu/src/lib.rs").unwrap();
    let file = syn::parse_file(&content).unwrap();

    for item in file.items.iter() {
        let span = item.span();
        let start = span.start().line;
        let end = span.end().line;
        match item {
            Item::Struct(s) => {
                println!("STRUCT|{}|{}|{}", s.ident, start, end);
            }
            Item::Impl(i) => {
                // If it's a trait impl, print the trait impl range.
                // But for SurtrRenderer, we want to split the methods!
                let is_surtr = if let syn::Type::Path(p) = &*i.self_ty {
                    p.path.segments.last().unwrap().ident == "SurtrRenderer"
                } else { false };
                
                let is_trait = i.trait_.is_some();
                
                if is_surtr && !is_trait {
                    // For inherent impl blocks of SurtrRenderer, emit each method
                    for impl_item in &i.items {
                        if let ImplItem::Fn(f) = impl_item {
                            let f_start = f.span().start().line;
                            let f_end = f.span().end().line;
                            println!("METHOD|SurtrRenderer|{}|{}|{}", f.sig.ident, f_start, f_end);
                        }
                    }
                } else if is_surtr && is_trait {
                    let t_name = &i.trait_.as_ref().unwrap().1.segments.last().unwrap().ident;
                    println!("TRAIT_IMPL|{}|SurtrRenderer|{}|{}", t_name, start, end);
                } else {
                    println!("IMPL|OTHER|{}|{}", start, end);
                }
            }
            Item::Fn(f) => {
                println!("FN|{}|{}|{}", f.sig.ident, start, end);
            }
            Item::Mod(m) => {
                // We shouldn't move tests out entirely if they are tied to lib, but we can emit them
                println!("MOD|{}|{}|{}", m.ident, start, end);
            }
            _ => {
                println!("OTHER|NONE|{}|{}", start, end);
            }
        }
    }
}
