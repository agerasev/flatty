use crate::utils::path;
use quote::quote;
use syn::{Attribute, DeriveInput, Meta, NestedMeta, Path};

fn has_gen<F>(input: &DeriveInput, pred: F) -> bool
where
    F: Fn(&Path) -> bool,
{
    for attr in &input.attrs {
        if attr.path.is_ident("derive") {
            if let Meta::List(meta_list) = attr.parse_meta().unwrap() {
                for nm in meta_list.nested {
                    if let NestedMeta::Meta(m) = nm {
                        if pred(m.path()) {
                            return true;
                        }
                    } else {
                        panic!();
                    }
                }
            } else {
                panic!();
            }
        }
    }
    false
}

pub fn has_path(input: &DeriveInput, path: &Path) -> bool {
    has_gen(input, |other_path| path::eq(path, other_path))
}

pub fn has(input: &DeriveInput, ident: &str) -> bool {
    has_gen(input, |path| path.is_ident(ident))
}

pub fn remove(attrs: &mut Vec<Attribute>, ident: &str) {
    attrs.retain_mut(|attr| {
        if attr.path.is_ident("derive") {
            let mut tokens = quote! {};
            let mut found = false;
            if let Meta::List(meta_list) = attr.parse_meta().unwrap() {
                for nm in &meta_list.nested {
                    if let NestedMeta::Meta(m) = nm {
                        if m.path().get_ident().unwrap() != ident {
                            tokens = quote! { #tokens #nm, };
                        } else {
                            found = true;
                        }
                    } else {
                        panic!();
                    }
                }
            } else {
                panic!();
            }
            if found {
                attr.tokens = quote! { ( #tokens ) };
            }
            !attr.tokens.is_empty()
        } else {
            true
        }
    });
}
