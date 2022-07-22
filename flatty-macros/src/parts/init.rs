use crate::utils::fields_iter::FieldsIter;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{self, Data, DeriveInput, Fields, Ident, Index};

pub fn make_type(input: &DeriveInput) -> (Ident, TokenStream2) {
    let ident = Ident::new(&format!("{}Init", input.ident), input.ident.span());
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

fn make_fields<FI: FieldsIter>(fields: &FI) -> TokenStream2 {
    let iter = fields.fields_iter();
    let len = iter.len();
    iter.enumerate().fold(quote! {}, |accum, (i, field)| {
        let ty = &field.ty;
        let ident = match &field.ident {
            Some(x) => quote! { #x },
            None => {
                let index = Index::from(i);
                quote! { #index }
            }
        };
        let add_size = if i + 1 < len {
            quote! { offset += <#ty as ::flatty::FlatSized>::SIZE; }
        } else {
            quote! {}
        };
        quote! {
            #accum
            offset = ::flatty::utils::upper_multiple(offset, <#ty as ::flatty::FlatBase>::ALIGN);
            <#ty>::init_unchecked(&mut mem[offset..], init.#ident);
            #add_size
        }
    })
}

pub fn make(input: &DeriveInput) -> TokenStream2 {
    let body = match &input.data {
        Data::Struct(struct_data) => make_fields(&struct_data.fields),
        Data::Enum(_enum_data) => unimplemented!(),
        Data::Union(_union_data) => unimplemented!(),
    };
    quote! {
        let mut offset: usize = 0;
        #body
        Self::interpret_mut_unchecked(mem)
    }
}
