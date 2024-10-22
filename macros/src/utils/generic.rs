use super::field_iter::FieldIter;
use proc_macro2::TokenStream;
use quote::quote;
use std::iter::Iterator;
use syn::{self, Data, DeriveInput, GenericParam, Generics};

pub fn without_defaults(generics: &Generics) -> Generics {
    let mut generics = generics.clone();
    for param in generics.params.iter_mut() {
        match param {
            GenericParam::Type(p) => {
                p.eq_token = None;
                p.default = None;
            }
            GenericParam::Const(p) => {
                p.eq_token = None;
                p.default = None;
            }
            GenericParam::Lifetime(_) => (),
        }
    }
    generics
}

pub fn args(generics: &Generics) -> TokenStream {
    generics.params.iter().fold(quote! {}, |accum, param| {
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
    })
}

pub fn where_clause(input: &DeriveInput, bound: TokenStream, last_bound: Option<TokenStream>) -> TokenStream {
    let existing = input.generics.where_clause.as_ref().map_or(quote! {}, |w| {
        let wp = &w.predicates;
        let comma = if wp.trailing_punct() {
            quote! {}
        } else {
            quote! {,}
        };
        quote! { #wp #comma }
    });

    fn collect_fields<I: FieldIter>(fields: &I, bound: &TokenStream, last_bound: Option<&TokenStream>) -> TokenStream {
        let iter = fields.iter();
        let len = iter.len();
        iter.enumerate().fold(quote! {}, |accum, (index, field)| {
            let ty = &field.ty;
            let b = if index + 1 < len { bound } else { last_bound.unwrap_or(bound) };
            quote! {
                #accum
                #ty: #b,
            }
        })
    }

    let generated = match &input.data {
        Data::Struct(struct_data) => collect_fields(&struct_data.fields, &bound, last_bound.as_ref()),
        Data::Enum(enum_data) => enum_data.variants.iter().fold(quote! {}, |accum, variant| {
            let variant_clause = collect_fields(&variant.fields, &bound, last_bound.as_ref());
            quote! { #accum #variant_clause }
        }),
        Data::Union(union_data) => collect_fields(&union_data.fields, &bound, last_bound.as_ref()),
    };

    quote! { where #existing #generated }
}
