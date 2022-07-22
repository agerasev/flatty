use crate::utils::fields_iter::FieldsIter;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{self, Data, DeriveInput, Ident};

fn make_fields<FI: FieldsIter>(fields: &FI) -> TokenStream2 {
    fields.fields_iter().fold(quote! {}, |accum, field| {
        let ty = &field.ty;
        quote! { #accum <#ty as ::flatty::FlatUnsized>::AlignAs, }
    })
}

pub fn make(input: &DeriveInput) -> (Ident, TokenStream2) {
    let ident = Ident::new(&format!("_{}AlignAs", input.ident), input.ident.span());
    let contents = match &input.data {
        Data::Struct(struct_data) => make_fields(&struct_data.fields),
        Data::Enum(enum_data) => {
            let enum_ty = quote! { u8 }; // TODO: Get enum index type
            enum_data
                .variants
                .iter()
                .fold(quote! { #enum_ty, }, |accum, variant| {
                    quote! { #accum #variant }
                })
        }
        Data::Union(union_data) => make_fields(&union_data.fields),
    };
    (ident, contents)
}
