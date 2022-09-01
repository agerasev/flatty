use crate::utils::fields_iter::FieldsIter;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::iter::Iterator;
use syn::{self, Data, DeriveInput, GenericParam};

fn make_fields<FI: FieldsIter>(
    fields: &FI,
    bound: &TokenStream2,
    last_bound: Option<&TokenStream2>,
) -> TokenStream2 {
    let iter = fields.fields_iter();
    let len = iter.len();
    iter.enumerate().fold(quote! {}, |accum, (index, field)| {
        let ty = &field.ty;
        let b = if index + 1 < len {
            bound
        } else {
            last_bound.unwrap_or(bound)
        };
        quote! {
            #accum
            #ty: #b,
        }
    })
}

pub fn make_params(input: &DeriveInput) -> (TokenStream2, TokenStream2) {
    let params = input
        .generics
        .params
        .iter()
        .fold(quote! {}, |accum, param| {
            let param = match param {
                GenericParam::Type(type_param) => {
                    let param = &type_param.ident;
                    quote! { #param }
                }
                GenericParam::Lifetime(lifetime_def) => {
                    let param = &lifetime_def.lifetime;
                    quote! { #param }
                }
                GenericParam::Const(const_param) => {
                    let param = &const_param.ident;
                    quote! { #param }
                }
            };
            quote! { #accum #param, }
        });

    let generated = &input.generics.params;
    (params, quote! { #generated })
}

pub fn make_bounds(
    input: &DeriveInput,
    bound: TokenStream2,
    last_bound: Option<TokenStream2>,
) -> TokenStream2 {
    let generated = match &input.data {
        Data::Struct(struct_data) => make_fields(&struct_data.fields, &bound, last_bound.as_ref()),
        Data::Enum(enum_data) => enum_data.variants.iter().fold(quote! {}, |accum, variant| {
            let variant_clause = make_fields(&variant.fields, &bound, last_bound.as_ref());
            quote! { #accum #variant_clause }
        }),
        Data::Union(union_data) => make_fields(&union_data.fields, &bound, last_bound.as_ref()),
    };
    let existing = input.generics.where_clause.as_ref().map_or(quote! {}, |w| {
        let wp = &w.predicates;
        let comma = if wp.trailing_punct() {
            quote! {}
        } else {
            quote! {,}
        };
        quote! { #wp #comma }
    });
    quote! { #existing #generated }
}

pub fn where_(bounds: TokenStream2) -> TokenStream2 {
    if !bounds.is_empty() {
        quote! { where #bounds }
    } else {
        quote! {}
    }
}
