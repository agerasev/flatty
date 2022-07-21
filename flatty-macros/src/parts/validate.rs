use crate::utils::fields_iter::FieldsIter;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::iter::Iterator;
use syn::{self, Data, DeriveInput};

fn make_pre_fields<FI: FieldsIter>(fields: &FI) -> TokenStream2 {
    let iter = fields.fields_iter();
    let len = iter.len();
    iter.enumerate().fold(quote! {}, |accum, (i, field)| {
        let ty = &field.ty;
        let add_size = if i + 1 < len {
            quote! { offset += <#ty as flatty::FlatSized>::SIZE; }
        } else {
            quote! {}
        };
        quote! {
            #accum
            offset = flatty::utils::upper_multiple(offset, <#ty as flatty::FlatBase>::ALIGN);
            <#ty>::pre_validate(&mem[offset..])?;
            #add_size
        }
    })
}

pub fn make_pre(input: &DeriveInput) -> TokenStream2 {
    let body = match &input.data {
        Data::Struct(struct_data) => make_pre_fields(&struct_data.fields),
        Data::Enum(enum_data) => {
            let enum_body =
                enum_data
                    .variants
                    .iter()
                    .enumerate()
                    .fold(quote! {}, |accum, (i, variant)| {
                        let variant_code = make_pre_fields(&variant.fields);
                        quote! {
                            #accum
                            #i => { #variant_code },
                        }
                    });
            let enum_ty = quote! { u8 }; // TODO: Detect type from `#[repr(..)]`
            let enum_len = enum_data.variants.len();
            quote! {
                let state = <#enum_ty>::interpret(data).unwrap();
                if state >= #enum_len {
                    return Err(flatty::InterpretError::InvalidState);
                }
                offset += <#enum_ty as flatty::FlatSized>::SIZE;
                match state {
                    #enum_body
                    _ => unreachable!(),
                };
            }
        }
        Data::Union(_) => quote! { panic!("Union cannot be validated alone"); },
    };
    quote! {
        let mut offset: usize = 0;
        #body
        Ok(())
    }
}

fn make_post_fields<FI: FieldsIter>(fields: &FI, prefix: TokenStream2) -> TokenStream2 {
    fields
        .fields_iter()
        .enumerate()
        .fold(quote! {}, |accum, (i, field)| {
            let ident = match &field.ident {
                Some(ident) => quote! { #ident },
                None => quote! { #i },
            };
            quote! {
                #accum
                #prefix #ident.post_validate()?;
            }
        })
}

pub fn make_post(input: &DeriveInput) -> TokenStream2 {
    let body = match &input.data {
        Data::Struct(struct_data) => make_post_fields(&struct_data.fields, quote! { self. }),
        Data::Enum(_enum_data) => unimplemented!(),
        Data::Union(_) => quote! { panic!("Union cannot be validated alone"); },
    };
    quote! {
        #body
        Ok(())
    }
}
