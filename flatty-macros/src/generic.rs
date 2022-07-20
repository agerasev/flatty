use crate::utils::FieldsIter;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::iter::Iterator;
use syn::{self, Data, DeriveInput};

fn make_where_clause_fields<FI: FieldsIter>(
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

pub fn make_where_clause(
    input: &DeriveInput,
    bound: TokenStream2,
    last_bound: Option<TokenStream2>,
) -> TokenStream2 {
    let generated = match &input.data {
        Data::Struct(struct_data) => {
            make_where_clause_fields(&struct_data.fields, &bound, last_bound.as_ref())
        }
        Data::Enum(enum_data) => enum_data.variants.iter().fold(quote! {}, |accum, variant| {
            let variant_clause =
                make_where_clause_fields(&variant.fields, &bound, last_bound.as_ref());
            quote! {
                #accum
                #variant_clause
            }
        }),
        Data::Union(union_data) => {
            make_where_clause_fields(&union_data.fields, &bound, last_bound.as_ref())
        }
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
