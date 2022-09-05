use crate::{
    utils::{generic, match_, FieldIter},
    Context,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Ident, Index, Type};

pub fn align_const(ctx: &Context, input: &DeriveInput) -> TokenStream {
    fn collect_fields<I: FieldIter>(fields: &I) -> TokenStream {
        fields.field_iter().fold(quote! { 1 }, |accum, field| {
            let ty = &field.ty;
            quote! {
                ::flatty::utils::max(#accum, <#ty as ::flatty::FlatBase>::ALIGN)
            }
        })
    }

    let value = match &input.data {
        Data::Struct(struct_data) => collect_fields(&struct_data.fields),
        Data::Enum(enum_data) => {
            let enum_ty = ctx.info.enum_type.as_ref().unwrap();
            enum_data.variants.iter().fold(
                quote! { <#enum_ty as ::flatty::FlatBase>::ALIGN },
                |accum, variant| {
                    let variant_align = collect_fields(&variant.fields);
                    quote! { ::flatty::utils::max(#accum, #variant_align) }
                },
            )
        }
        Data::Union(union_data) => collect_fields(&union_data.fields),
    };

    quote! { const ALIGN: usize = #value; }
}

pub fn min_size_const(ctx: &Context, input: &DeriveInput) -> TokenStream {
    pub fn collect_fields<I: FieldIter>(fields: &I) -> TokenStream {
        let iter = fields.field_iter();
        let len = iter.len();
        iter.enumerate().fold(quote! { 0 }, |accum, (i, field)| {
            let ty = &field.ty;
            let size = if i + 1 < len {
                quote! { <#ty as ::flatty::FlatSized>::SIZE }
            } else {
                quote! { <#ty as ::flatty::FlatBase>::MIN_SIZE }
            };
            quote! {
                ::flatty::utils::ceil_mul(#accum, <#ty as ::flatty::FlatBase>::ALIGN) + #size
            }
        })
    }

    let value = match &input.data {
        Data::Struct(struct_data) => collect_fields(&struct_data.fields),
        Data::Enum(enum_data) => {
            let enum_ty = ctx.info.enum_type.as_ref().unwrap();
            let contents = enum_data
                .variants
                .iter()
                .fold(quote! { 0 }, |accum, variant| {
                    let variant_min_size = collect_fields(&variant.fields);
                    quote! { ::flatty::utils::min(#accum, #variant_min_size) }
                });
            quote! {
                ::flatty::utils::ceil_mul(
                    ::flatty::utils::ceil_mul(
                        <#enum_ty as ::flatty::FlatSized>::SIZE,
                        <Self as ::flatty::FlatBase>::ALIGN,
                    ) + #contents,
                    <Self as ::flatty::FlatBase>::ALIGN,
                )
            }
        }
        Data::Union(union_data) => collect_fields(&union_data.fields),
    };

    quote! { const MIN_SIZE: usize = #value; }
}

fn size_method(ctx: &Context, input: &DeriveInput) -> TokenStream {
    fn collect_fields<I: FieldIter, F>(fields: &I, map_ident: F) -> TokenStream
    where
        F: Fn(&Type, &TokenStream) -> TokenStream,
    {
        let iter = fields.field_iter();
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
                let mapped_ident = map_ident(ty, &ident);
                quote! { offset += #mapped_ident; }
            };
            quote! {
                #accum
                offset = ::flatty::utils::ceil_mul(offset, <#ty as ::flatty::FlatBase>::ALIGN);
                #add_size
            }
        })
    }

    pub fn template<F>(input: &DeriveInput, ident: &Ident, value: TokenStream) -> TokenStream
    where
        F: Fn(&Type, &TokenStream) -> TokenStream,
    {
        let body = match &input.data {
            Data::Struct(struct_data) => collect_fields(&struct_data.fields, |ty, fid| {
                quote! { #value.#fid.size() }
            }),
            Data::Enum(enum_data) => {
                let enum_body = enum_data.variants.iter().fold(quote! {}, |accum, variant| {
                    let var = &variant.ident;
                    let bs = match_::bindings(&variant.fields);
                    let (pattern, wrapper, prefix) = (bs.pattern, bs.wrapper, bs.prefix);
                    let code = collect_fields(&variant.fields, |ty, fid| {
                        quote! { #prefix #fid.size() }
                    });
                    quote! {
                        #accum
                        #ident::#var #pattern => {
                            #wrapper
                            #code
                        },
                    }
                });
                quote! {
                    offset += Self::DATA_OFFSET;
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
            offset = ::flatty::utils::ceil_mul(offset, <Self as ::flatty::FlatBase>::ALIGN);
            offset
        }
    }

    let body = match &input.data {
        Data::Struct(struct_data) => template(input, &input.ident, quote! { self }),
        Data::Enum(enum_data) => template(input, &ref_ident(input), quote! { self.as_ref() }),
        Data::Union(_union_data) => unimplemented!(),
    };
    quote! {
        fn size(this: &::flatty::mem::Muu<Self>) -> Result<(), ::flatty::Error> {
            use ::flatty::{mem::Muu, prelude::*, utils::ceil_mul};

            let mut pos: usize = 0;
            let bytes = this.as_bytes();

            #body

            Ok(())
        }
    }
}

pub fn impl_(ctx: &Context, input: &DeriveInput) -> TokenStream {
    assert!(!ctx.info.sized);

    let self_ident = &input.ident;

    let generic_params = &input.generics.params;
    let generic_args = generic::args(&input.generics);
    let where_clause = generic::where_clause(
        input,
        quote! { ::flatty::FlatBase + Sized },
        if ctx.info.sized {
            None
        } else {
            Some(quote! { ::flatty::FlatBase })
        },
    );

    let align_const = align_const(ctx, input);
    let min_size_const = min_size_const(ctx, input);

    quote! {
        impl<#generic_params> ::flatty::FlatCast for #self_ident<#generic_args>
        where
            #where_clause
        {
            #align_const
            #min_size_const
        }
    }
}
