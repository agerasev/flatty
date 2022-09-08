use crate::Context;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Index};

pub fn impl_(ctx: &Context, input: &DeriveInput, local: bool) -> TokenStream {
    if let Data::Enum(data) = &input.data {
        let enum_type = ctx.info.enum_type.as_ref().unwrap();
        let derive_default = if ctx.info.default {
            quote! { #[derive(Default)] }
        } else {
            quote! {}
        };
        let vis = if !local { Some(&input.vis) } else { None };
        let tag_type = ctx.idents.tag.as_ref().unwrap();
        let var_count = Index::from(data.variants.len());
        let variants = data.variants.iter().fold(quote! {}, |accum, var| {
            let ident = &var.ident;
            let default = var.attrs.iter().find(|attr| {
                attr.path
                    .get_ident()
                    .map(|ident| ident == "default")
                    .unwrap_or(false)
            });
            quote! {
                #accum
                #default
                #ident,
            }
        });
        quote! {
            #[allow(dead_code)]
            #[derive(Clone, Copy, PartialEq, Eq, Debug)]
            #derive_default
            #[repr(#enum_type)]
            #vis enum #tag_type {
                #variants
            }

            impl ::flatty::FlatCast for #tag_type {
                fn validate(this: &::flatty::mem::MaybeUninitUnsized<Self>) -> Result<(), ::flatty::Error> {
                    use ::flatty::{prelude::*, mem::MaybeUninitUnsized, Error, ErrorKind};
                    let tag = unsafe { MaybeUninitUnsized::<#enum_type>::from_bytes_unchecked(this.as_bytes()) };
                    <#enum_type as FlatCast>::validate(tag)?;
                    if *unsafe { tag.assume_init_ref() } < #var_count {
                        Ok(())
                    } else {
                        Err(Error {
                            kind: ErrorKind::InvalidEnumState,
                            pos: 0,
                        })
                    }
                }
            }

            unsafe impl ::flatty::Flat for #tag_type {}
        }
    } else {
        panic!("Only enum has tag");
    }
}
