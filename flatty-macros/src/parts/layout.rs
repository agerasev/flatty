use crate::{parts::attrs, utils::fields_iter::FieldsIter};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::iter::Iterator;
use syn::{self, Data, DeriveInput, Index};

fn make_align_fields<FI: FieldsIter>(fields: &FI) -> TokenStream2 {
    fields.fields_iter().fold(quote! { 1 }, |accum, field| {
        let ty = &field.ty;
        quote! {
            ::flatty::utils::max(#accum, <#ty as ::flatty::FlatBase>::ALIGN)
        }
    })
}

fn make_min_size_fields<FI: FieldsIter>(fields: &FI) -> TokenStream2 {
    let iter = fields.fields_iter();
    let len = iter.len();
    iter.enumerate().fold(quote! { 0 }, |accum, (i, field)| {
        let ty = &field.ty;
        let size = if i + 1 < len {
            quote! { <#ty as ::flatty::FlatSized>::SIZE }
        } else {
            quote! { <#ty as ::flatty::FlatBase>::MIN_SIZE }
        };
        quote! {
            ::flatty::utils::upper_multiple(#accum, <#ty as ::flatty::FlatBase>::ALIGN) + #size
        }
    })
}

pub fn make_align(input: &DeriveInput) -> TokenStream2 {
    match &input.data {
        Data::Struct(struct_data) => make_align_fields(&struct_data.fields),
        Data::Enum(enum_data) => {
            let enum_ty = attrs::get_enum_type(input);
            enum_data.variants.iter().fold(
                quote! { <#enum_ty as ::flatty::FlatBase>::ALIGN },
                |accum, variant| {
                    let variant_align = make_align_fields(&variant.fields);
                    quote! { ::flatty::utils::max(#accum, #variant_align) }
                },
            )
        }
        Data::Union(union_data) => make_align_fields(&union_data.fields),
    }
}

pub fn make_min_size(input: &DeriveInput) -> TokenStream2 {
    let ty = &input.ident;
    match &input.data {
        Data::Struct(struct_data) => make_min_size_fields(&struct_data.fields),
        Data::Enum(enum_data) => {
            let enum_ty = attrs::get_enum_type(input);
            let contents = enum_data
                .variants
                .iter()
                .fold(quote! { 0 }, |accum, variant| {
                    let variant_align = make_min_size_fields(&variant.fields);
                    quote! { ::flatty::utils::max(#accum, #variant_align) }
                });
            quote! {
                ::flatty::utils::upper_multiple(
                    <#enum_ty as ::flatty::FlatBase>::SIZE,
                    <#ty as ::flatty::FlatBase>::ALIGN,
                ) + #contents
            }
        }
        Data::Union(union_data) => make_min_size_fields(&union_data.fields),
    }
}

pub fn make_size(input: &DeriveInput) -> TokenStream2 {
    match &input.data {
        Data::Struct(struct_data) => {
            let last_field = struct_data.fields.fields_iter().enumerate().last();
            match last_field {
                Some((i, field)) => {
                    let field_ty = &field.ty;
                    let field_path = match &field.ident {
                        Some(x) => quote! { #x },
                        None => {
                            let index = Index::from(i);
                            quote! { #index }
                        }
                    };
                    quote! { Self::MIN_SIZE - <#field_ty as ::flatty::FlatBase>::MIN_SIZE + self.#field_path.size() }
                }
                None => quote! { Self::MIN_SIZE },
            }
        }
        Data::Enum(_enum_data) => unimplemented!(),
        Data::Union(_union_data) => unimplemented!(),
    }
}

pub fn make_ptr_metadata(input: &DeriveInput) -> TokenStream2 {
    match &input.data {
        Data::Struct(struct_data) => match struct_data.fields.fields_iter().last() {
            Some(field) => {
                let field_ty = &field.ty;
                quote! {
                    <#field_ty as ::flatty::FlatUnsized>::ptr_metadata(
                        &mem[(<Self as ::flatty::FlatBase>::MIN_SIZE - <#field_ty as ::flatty::FlatBase>::MIN_SIZE)..],
                    )
                }
            }
            None => quote! { 0 },
        },
        Data::Enum(_enum_data) => unimplemented!(),
        Data::Union(_union_data) => unimplemented!(),
    }
}
