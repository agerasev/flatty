use crate::{
    utils::{generic, type_list, FieldIter},
    Context,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Index};

pub fn align_const(ctx: &Context, input: &DeriveInput) -> TokenStream {
    let generic_args = generic::args(&input.generics);
    let align_as_ident = ctx.idents.align_as.as_ref().unwrap();
    let align_as_type = quote! { #align_as_ident<#generic_args> };
    quote! { const ALIGN: usize = ::core::mem::align_of::<#align_as_type>(); }
}

pub fn min_size_collect_fields<I: FieldIter>(fields: &I) -> TokenStream {
    let iter = fields.iter();
    if iter.len() > 0 {
        let type_list = type_list(iter);
        quote! { ::flatty::utils::iter::fold_min_size!(0; #type_list) }
    } else {
        quote! { 0 }
    }
}

pub fn min_size_const(_ctx: &Context, input: &DeriveInput) -> TokenStream {
    let value = match &input.data {
        Data::Struct(struct_data) => min_size_collect_fields(&struct_data.fields),
        Data::Enum(enum_data) => {
            let contents = enum_data.variants.iter().enumerate().fold(quote! {}, |accum, (index, _var)| {
                let var_min_size = quote! { Self::DATA_MIN_SIZES[#index] };
                if accum.is_empty() {
                    quote! { #var_min_size }
                } else {
                    quote! { ::flatty::utils::min(#accum, #var_min_size) }
                }
            });
            quote! {
                ::flatty::utils::ceil_mul(Self::DATA_OFFSET + #contents, <Self as ::flatty::traits::FlatBase>::ALIGN)
            }
        }
        Data::Union(..) => unimplemented!(),
    };

    quote! { const MIN_SIZE: usize = #value; }
}

fn size_method(ctx: &Context, input: &DeriveInput) -> TokenStream {
    let value = match &input.data {
        Data::Struct(struct_data) => {
            let last = struct_data.fields.iter().enumerate().last().map(|(i, f)| match &f.ident {
                Some(ident) => ident.to_token_stream(),
                None => Index::from(i).to_token_stream(),
            });
            match last {
                Some(last) => {
                    quote! { Self::LAST_FIELD_OFFSET + self.#last.size() }
                }
                None => quote! { 0 },
            }
        }
        Data::Enum(enum_data) => {
            let variants = enum_data.variants.iter().fold(quote! {}, |accum, variant| {
                let tag_type = ctx.idents.tag.as_ref().unwrap();
                let var_name = &variant.ident;
                let value = if !variant.fields.is_empty() {
                    let type_list = type_list(variant.fields.iter());
                    quote! { unsafe { iter::BytesIter::new_unchecked(&self.data, iter::type_list!(#type_list)).fold_size(0) } }
                } else {
                    quote! { 0 }
                };
                quote! {
                    #accum
                    #tag_type::#var_name => { #value }
                }
            });
            quote! {
                {
                    use ::flatty::utils::iter::{prelude::*, self};
                    Self::DATA_OFFSET + match self.tag {
                        #variants
                    }
                }
            }
        }
        Data::Union(_union_data) => unimplemented!(),
    };
    quote! {
        fn size(&self) -> usize {
            use ::flatty::{traits::*, utils::ceil_mul};
            ceil_mul(#value, Self::ALIGN)
        }
    }
}

pub fn self_impl(ctx: &Context, input: &DeriveInput) -> TokenStream {
    let self_ident = &input.ident;

    let generic_params = &input.generics.params;
    let generic_args = generic::args(&input.generics);
    let where_clause = &input.generics.where_clause;

    let mut items = quote! {};

    match &input.data {
        Data::Enum(data) => {
            let tag_type = ctx.info.tag_type.as_ref().unwrap();

            items = quote! {
                #items
                const DATA_OFFSET: usize = ::flatty::utils::ceil_mul(
                    <#tag_type as ::flatty::traits::FlatSized>::SIZE,
                    <Self as ::flatty::traits::FlatBase>::ALIGN,
                );
            };

            if !ctx.info.sized {
                let var_count = data.variants.len();
                let values = data.variants.iter().fold(quote! {}, |accum, variant| {
                    let var_min_size = min_size_collect_fields(&variant.fields);
                    quote! { #accum #var_min_size, }
                });
                items = quote! {
                    #items
                    const DATA_MIN_SIZES: [usize; #var_count] = [ #values ];
                }
            }
        }
        Data::Struct(data) => {
            if !data.fields.is_empty() && !ctx.info.sized {
                let value = if data.fields.len() > 1 {
                    let len = data.fields.len();
                    let type_list = type_list(data.fields.iter().take(len - 1));
                    let last_ty = &data.fields.iter().last().unwrap().ty;
                    quote! {
                        ::flatty::utils::ceil_mul(
                            ::flatty::utils::iter::fold_size!(0; #type_list),
                            <#last_ty as ::flatty::traits::FlatBase>::ALIGN,
                        )
                    }
                } else {
                    quote! { 0 }
                };
                items = quote! {
                    #items
                    const LAST_FIELD_OFFSET: usize = #value;
                }
            }
        }
        Data::Union(..) => unimplemented!(),
    }

    quote! {
        impl<#generic_params> #self_ident<#generic_args>
        #where_clause
        {
            #items
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
        quote! { ::flatty::traits::FlatBase + Sized },
        if ctx.info.sized {
            None
        } else {
            Some(quote! { ::flatty::traits::FlatBase })
        },
    );

    let align_const = align_const(ctx, input);
    let min_size_const = min_size_const(ctx, input);
    let size_method = size_method(ctx, input);

    quote! {
        unsafe impl<#generic_params> ::flatty::traits::FlatBase for #self_ident<#generic_args>
        #where_clause
        {
            #align_const
            #min_size_const

            #size_method
        }
    }
}
