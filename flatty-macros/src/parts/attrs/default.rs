use quote::quote;
use syn::{Attribute, Data, DeriveInput, Meta, NestedMeta};

pub fn has(input: &DeriveInput) -> bool {
    for attr in &input.attrs {
        if attr.path.is_ident("derive") {
            if let Meta::List(meta_list) = attr.parse_meta().unwrap() {
                for nm in meta_list.nested {
                    if let NestedMeta::Meta(m) = nm {
                        let ident = m.path().get_ident().unwrap();
                        if ident == "Default" {
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

pub fn remove_derive(attrs: &mut Vec<Attribute>) {
    attrs.retain_mut(|attr| {
        if attr.path.is_ident("derive") {
            let mut tokens = quote! {};
            let mut found = false;
            if let Meta::List(meta_list) = attr.parse_meta().unwrap() {
                for nm in &meta_list.nested {
                    if let NestedMeta::Meta(m) = nm {
                        if m.path().get_ident().unwrap() != "Default" {
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

pub fn filter_out(input: &DeriveInput) -> DeriveInput {
    let mut output = input.clone();
    remove_derive(&mut output.attrs);
    match &mut output.data {
        Data::Enum(enum_data) => {
            for variant in &mut enum_data.variants {
                variant.attrs.retain(|attr| !attr.path.is_ident("default"))
            }
        }
        _ => (),
    }
    output
}
