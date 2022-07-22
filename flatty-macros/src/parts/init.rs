//use crate::utils::fields_iter::FieldsIter;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{self, Data, DeriveInput, Fields, Ident};

pub fn make_type(input: &DeriveInput) -> (Ident, TokenStream2) {
    let ident = Ident::new(&format!("_{}Init", input.ident), input.ident.span());
    let contents = match &input.data {
        Data::Struct(struct_data) => match &struct_data.fields {
            Fields::Named(fields) => {
                let contents = fields.named.iter().fold(quote! {}, |accum, field| {
                    let field_ty = &field.ty;
                    let field_ident = field.ident.as_ref().unwrap();
                    quote! { #accum #field_ident: <#field_ty as ::flatty::FlatInit>::Init, }
                });
                quote! { { #contents } }
            }
            Fields::Unnamed(fields) => {
                let contents = fields.unnamed.iter().fold(quote! {}, |accum, field| {
                    let field_ty = &field.ty;
                    assert!(field.ident.is_none());
                    quote! { #accum <#field_ty as ::flatty::FlatInit>::Init, }
                });
                quote! { (#contents); }
            }
            Fields::Unit => quote! { ; },
        },
        Data::Enum(_enum_data) => unimplemented!(),
        Data::Union(_union_data) => unimplemented!(),
    };
    (ident, contents)
}

/*
fn make_fields<FI: FieldsIter>(fields: &FI) -> TokenStream2 {
    let iter = fields.fields_iter();
    let len = iter.len();
    iter.enumerate().fold(quote! {}, |accum, (i, field)| {
        let ty = &field.ty;
        let add_size = if i + 1 < len {
            quote! { offset += <#ty as ::flatty::FlatSized>::SIZE; }
        } else {
            quote! {}
        };
        quote! {
            #accum
            offset = ::flatty::utils::upper_multiple(offset, <#ty as ::flatty::FlatBase>::ALIGN);
            <#ty>::pre_validate(&mem[offset..])?;
            #add_size
        }
    })
}

pub fn make(input: &DeriveInput) -> TokenStream2 {
    let body = match &input.data {
        Data::Struct(struct_data) => make_fields(&struct_data.fields),
        Data::Enum(enum_data) => {
            let enum_body =
                enum_data
                    .variants
                    .iter()
                    .enumerate()
                    .fold(quote! {}, |accum, (i, variant)| {
                        let index = Index::from(i);
                        let code = make_fields(&variant.fields);
                        quote! {
                            #accum
                            #index => { #code },
                        }
                    });
            let enum_ty = quote! { u8 }; // TODO: Detect type from `#[repr(..)]`
            let enum_len = Index::from(enum_data.variants.len());
            quote! {
                let state = <#enum_ty>::interpret(mem).unwrap();
                if *state >= #enum_len {
                    return Err(::flatty::InterpretError::InvalidState);
                }
                offset += <#enum_ty as ::flatty::FlatSized>::SIZE;
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
*/
