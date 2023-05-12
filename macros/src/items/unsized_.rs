use crate::{utils::generic, Context};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

fn ptr_from_bytes_method(_ctx: &Context, input: &DeriveInput) -> TokenStream {
    let body = match &input.data {
        Data::Struct(struct_data) => {
            assert!(!struct_data.fields.is_empty());
            let last_ty = &struct_data.fields.iter().last().unwrap().ty;
            quote! {
                use ::flatty::utils::mem::{offset_slice_ptr_start, cast_wide_ptr_with_offset};
                cast_wide_ptr_with_offset!(
                    Self,
                    <#last_ty as FlatUnsized>::ptr_from_bytes(offset_slice_ptr_start(bytes, Self::LAST_FIELD_OFFSET as isize)),
                    -(Self::LAST_FIELD_OFFSET as isize),
                )
            }
        }
        Data::Enum(..) => quote! {
            use ::flatty::utils::{floor_mul, mem::{set_slice_ptr_len, slice_ptr_len}};
            set_slice_ptr_len(bytes, floor_mul(slice_ptr_len(bytes) - Self::DATA_OFFSET, Self::ALIGN)) as *mut Self
        },
        Data::Union(..) => unimplemented!(),
    };
    quote! {
        unsafe fn ptr_from_bytes(bytes: *mut [u8]) -> *mut Self {
            use ::flatty::prelude::*;
            #body
        }
    }
}

fn ptr_to_bytes_method(_ctx: &Context, input: &DeriveInput) -> TokenStream {
    let body = match &input.data {
        Data::Struct(struct_data) => {
            assert!(!struct_data.fields.is_empty());
            let last_ty = &struct_data.fields.iter().enumerate().last().unwrap().1.ty;
            quote! {
                use ::flatty::utils::mem::{offset_slice_ptr_start, cast_wide_ptr_with_offset};
                offset_slice_ptr_start(
                    <#last_ty as FlatUnsized>::ptr_to_bytes(cast_wide_ptr_with_offset!(#last_ty, this, Self::LAST_FIELD_OFFSET as isize)),
                    -(Self::LAST_FIELD_OFFSET as isize),
                )
            }
        }
        Data::Enum(..) => quote! {
            use ::flatty::utils::mem::{set_slice_ptr_len, slice_ptr_len};
            let bytes = this as *mut [u8];
            set_slice_ptr_len(bytes, Self::DATA_OFFSET + slice_ptr_len(bytes))
        },
        Data::Union(..) => unimplemented!(),
    };
    quote! {
        unsafe fn ptr_to_bytes(this: *mut Self) -> *mut [u8] {
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
        quote! { ::flatty::traits::FlatBase + Sized },
        if ctx.info.sized {
            None
        } else {
            Some(quote! { ::flatty::traits::FlatUnsized })
        },
    );

    let align_as_ident = ctx.idents.align_as.as_ref().unwrap();
    let align_as_type = quote! { #align_as_ident<#generic_args> };
    let ptr_from_bytes_method = ptr_from_bytes_method(ctx, input);
    let ptr_to_bytes_method = ptr_to_bytes_method(ctx, input);

    quote! {
        unsafe impl<#generic_params> ::flatty::traits::FlatUnsized for #self_ident<#generic_args>
        #where_clause
        {
            type AlignAs = #align_as_type;

            #ptr_from_bytes_method
            #ptr_to_bytes_method
        }
    }
}
