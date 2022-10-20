use crate::{utils::generic, Context};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Index};

pub fn struct_(ctx: &Context, input: &DeriveInput, local: bool) -> TokenStream {
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
            let default = var
                .attrs
                .iter()
                .find(|attr| attr.path.get_ident().map(|ident| ident == "default").unwrap_or(false));
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

            impl ::flatty::FlatCheck for #tag_type {
                fn validate(this: &::flatty::mem::MaybeUninitUnsized<Self>) -> Result<&Self, ::flatty::Error> {
                    use ::flatty::{prelude::*, mem::MaybeUninitUnsized, Error, ErrorKind};
                    let tag = unsafe { MaybeUninitUnsized::<#enum_type>::from_bytes_unchecked(this.as_bytes()) };
                    if *(<#enum_type as FlatCheck>::validate(tag)?) < #var_count {
                        Ok(unsafe { this.assume_init() })
                    } else {
                        Err(Error {
                            kind: ErrorKind::InvalidEnumTag,
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

pub fn impl_(ctx: &Context, input: &DeriveInput) -> TokenStream {
    if let Data::Enum(..) = &input.data {
        let self_ident = &input.ident;

        let generic_params = &input.generics.params;
        let generic_args = generic::args(&input.generics);
        let where_clause = &input.generics.where_clause;

        let tag_type = ctx.idents.tag.as_ref().unwrap();

        assert!(!ctx.info.sized);
        quote! {
            impl<#generic_params> #self_ident<#generic_args>
            #where_clause
            {
                pub fn tag(&self) -> #tag_type {
                    self.tag
                }
            }
        }
    } else {
        panic!("Only enum has tag");
    }
}
