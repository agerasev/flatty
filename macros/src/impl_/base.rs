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

pub fn min_size_const(_ctx: &Context, input: &DeriveInput) -> TokenStream {
    pub fn collect_fields<I: FieldIter>(fields: &I) -> TokenStream {
        let type_list = type_list(fields.iter());
        quote! { ::flatty::iter::fold_min_size!(0; #type_list) }
    }

    let value = match &input.data {
        Data::Struct(struct_data) => collect_fields(&struct_data.fields),
        Data::Enum(enum_data) => {
            let contents = enum_data.variants.iter().fold(quote! {}, |accum, variant| {
                let var_min_size = collect_fields(&variant.fields);
                if accum.is_empty() {
                    quote! { #var_min_size }
                } else {
                    quote! { ::flatty::utils::min(#accum, #var_min_size) }
                }
            });
            quote! {
                ::flatty::utils::ceil_mul(
                    Self::DATA_OFFSET + #contents,
                    <Self as ::flatty::FlatBase>::ALIGN,
                )
            }
        }
        Data::Union(..) => unimplemented!(),
    };

    quote! { const MIN_SIZE: usize = #value; }
}

fn size_method(ctx: &Context, input: &DeriveInput) -> TokenStream {
    let value = match &input.data {
        Data::Struct(struct_data) => {
            let last = struct_data
                .fields
                .iter()
                .enumerate()
                .last()
                .map(|(i, f)| match &f.ident {
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
                let type_list = type_list(variant.fields.iter());
                quote! {
                    #accum
                    #tag_type::#var_name => unsafe {
                        RefIter::new_unchecked(&self.data, type_list!(#type_list)).fold_size(0)
                    }
                }
            });
            quote! {
                {
                    use ::flatty::iter::{prelude::*, RefIter, type_list};
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
            use ::flatty::{prelude::*, utils::ceil_mul};
            ceil_mul(#value, Self::ALIGN)
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
    let size_method = size_method(ctx, input);

    quote! {
        unsafe impl<#generic_params> ::flatty::FlatBase for #self_ident<#generic_args>
        #where_clause
        {
            #align_const
            #min_size_const

            #size_method
        }
    }
}
