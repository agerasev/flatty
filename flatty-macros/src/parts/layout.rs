use crate::{
    parts::{attrs, match_},
    utils::fields_iter::FieldsIter,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::iter::Iterator;
use syn::{self, Data, DeriveInput, Ident, Index};

fn make_align_fields<FI: FieldsIter>(fields: &FI) -> TokenStream2 {
    fields.fields_iter().fold(quote! { 1 }, |accum, field| {
        let ty = &field.ty;
        quote! {
            ::flatty::utils::max(#accum, <#ty as ::flatty::FlatBase>::ALIGN)
        }
    })
}

pub fn make_align(input: &DeriveInput) -> TokenStream2 {
    match &input.data {
        Data::Struct(struct_data) => make_align_fields(&struct_data.fields),
        Data::Enum(enum_data) => {
            let enum_ty = attrs::repr::get_enum_type(input);
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

pub fn make_min_size_fields<FI: FieldsIter>(fields: &FI) -> TokenStream2 {
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

pub fn make_min_size(input: &DeriveInput) -> TokenStream2 {
    match &input.data {
        Data::Struct(struct_data) => make_min_size_fields(&struct_data.fields),
        Data::Enum(enum_data) => {
            let enum_ty = attrs::repr::get_enum_type(input);
            let contents = enum_data
                .variants
                .iter()
                .fold(quote! { 0 }, |accum, variant| {
                    let variant_min_size = make_min_size_fields(&variant.fields);
                    quote! { ::flatty::utils::min(#accum, #variant_min_size) }
                });
            quote! {
                ::flatty::utils::upper_multiple(
                    ::flatty::utils::upper_multiple(
                        <#enum_ty as ::flatty::FlatSized>::SIZE,
                        Self::ALIGN,
                    ) + #contents,
                    Self::ALIGN,
                )
            }
        }
        Data::Union(union_data) => make_min_size_fields(&union_data.fields),
    }
}

fn make_size_fields<FI: FieldsIter>(fields: &FI, prefix: TokenStream2) -> TokenStream2 {
    let iter = fields.fields_iter();
    let len = iter.len();
    iter.enumerate().fold(quote! {}, |accum, (i, field)| {
        let ty = &field.ty;
        let index = Index::from(i);
        let ident = match &field.ident {
            Some(ident) => quote! { #ident },
            None => quote! { #index },
        };
        let add_size = if i + 1 < len {
            quote! { offset += <#ty as ::flatty::FlatSized>::SIZE; }
        } else {
            quote! { offset += (#prefix #ident).size(); }
        };
        quote! {
            #accum
            offset = ::flatty::utils::upper_multiple(offset, <#ty as ::flatty::FlatBase>::ALIGN);
            #add_size
        }
    })
}

pub fn make_size_gen(input: &DeriveInput, ident: &Ident, value: TokenStream2) -> TokenStream2 {
    let body = match &input.data {
        Data::Struct(struct_data) => make_size_fields(&struct_data.fields, quote! { self. }),
        Data::Enum(enum_data) => {
            let enum_body = enum_data.variants.iter().fold(quote! {}, |accum, variant| {
                let var = &variant.ident;
                let bs = match_::make_bindings(&variant.fields);
                let (bindings, wrapper) = (bs.bindings, bs.wrapper);
                let code = make_size_fields(&variant.fields, bs.prefix);
                quote! {
                    #accum
                    #ident::#var #bindings => {
                        #wrapper
                        #code
                    },
                }
            });
            quote! {
                match (#value) {
                    #enum_body
                }
            }
        }
        Data::Union(_) => quote! { panic!("Union size cannot be determined alone"); },
    };
    quote! {
        let mut offset: usize = 0;
        #body
        offset = ::flatty::utils::upper_multiple(offset, Self::ALIGN);
        offset
    }
}

pub fn make_size(input: &DeriveInput) -> TokenStream2 {
    make_size_gen(input, &input.ident, quote! { self })
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
