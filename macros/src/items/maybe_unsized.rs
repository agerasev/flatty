use crate::{utils::generic, Context};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Index};

fn ptr_metadata_method(_ctx: &Context, input: &DeriveInput) -> TokenStream {
    let body = match &input.data {
        Data::Struct(struct_data) => {
            assert!(!struct_data.fields.is_empty());
            let last_ty = &struct_data.fields.iter().last().unwrap().ty;
            quote!(
                <#last_ty as FlatMaybeUnsized>::ptr_metadata(unsafe {
                    MaybeUninitUnsized::<#last_ty>::from_bytes_unchecked(&this.as_bytes()[Self::LAST_FIELD_OFFSET..])
                })
            )
        }
        Data::Enum(..) => quote! {
            use ::flatty::{utils::floor_mul};
            floor_mul(this.as_bytes().len() - Self::DATA_OFFSET, Self::ALIGN)
        },
        Data::Union(..) => unimplemented!(),
    };
    quote! {
        fn ptr_metadata(this: &::flatty::mem::MaybeUninitUnsized<Self>) -> usize {
            use ::flatty::{prelude::*, mem::MaybeUninitUnsized};
            #body
        }
    }
}

fn bytes_len_method(_ctx: &Context, input: &DeriveInput) -> TokenStream {
    let body = match &input.data {
        Data::Struct(struct_data) => {
            assert!(!struct_data.fields.is_empty());
            let (i, last_field) = struct_data.fields.iter().enumerate().last().unwrap();
            let last_ty = &last_field.ty;
            let last = match &last_field.ident {
                Some(ident) => ident.to_token_stream(),
                None => Index::from(i).to_token_stream(),
            };
            quote!(
                Self::LAST_FIELD_OFFSET + <#last_ty as FlatMaybeUnsized>::bytes_len(&this.#last)
            )
        }
        Data::Enum(..) => quote! {
            Self::DATA_OFFSET + this.data.len()
        },
        Data::Union(..) => unimplemented!(),
    };
    quote! {
        fn bytes_len(this: &Self) -> usize {
            use ::flatty::prelude::*;
            #body
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
            Some(quote! { ::flatty::FlatMaybeUnsized })
        },
    );

    let align_as_ident = ctx.idents.align_as.as_ref().unwrap();
    let align_as_type = quote! { #align_as_ident<#generic_args> };
    let ptr_metadata_method = ptr_metadata_method(ctx, input);
    let bytes_len_method = bytes_len_method(ctx, input);

    quote! {
        unsafe impl<#generic_params> ::flatty::FlatMaybeUnsized for #self_ident<#generic_args>
        #where_clause
        {
            type AlignAs = #align_as_type;

            #ptr_metadata_method
            #bytes_len_method

            ::flatty::impl_unsized_uninit_cast!();
        }
    }
}
