use crate::{utils::generic, Context};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Index};

pub fn validate_code(ctx: &Context, input: &DeriveInput, bytes_ident: &TokenStream) -> TokenStream {
    if let Data::Enum(data) = &input.data {
        let tag_type = ctx.info.tag_type.as_ref().unwrap();
        let var_count = Index::from(data.variants.len());
        quote! {
            use ::flatty::{traits::*, error::{Error, ErrorKind}};
            <#tag_type>::validate_unchecked(#bytes_ident)?;
            let tag = <#tag_type>::from_bytes_unchecked(#bytes_ident);
            if *tag < #var_count {
                Ok(())
            } else {
                Err(Error {
                    kind: ErrorKind::InvalidEnumTag,
                    pos: 0,
                })
            }
        }
    } else {
        panic!("Only enum has tag");
    }
}

pub fn struct_(ctx: &Context, input: &DeriveInput, local: bool) -> TokenStream {
    if let Data::Enum(data) = &input.data {
        let tag_type = ctx.info.tag_type.as_ref().unwrap();
        let vis = if !local { Some(&input.vis) } else { None };
        let tag = ctx.idents.tag.as_ref().unwrap();
        let variants = data.variants.iter().fold(quote! {}, |accum, var| {
            let ident = &var.ident;
            quote! {
                #accum
                #ident,
            }
        });
        let bytes_ident = quote! {__flatty_bytes};
        let code = validate_code(ctx, input, &bytes_ident);
        quote! {
            #[allow(dead_code)]
            #[derive(Clone, Copy, PartialEq, Eq, Debug)]
            #[repr(#tag_type)]
            #vis enum #tag {
                #variants
            }

            unsafe impl ::flatty::traits::FlatValidate for #tag {
                unsafe fn validate_unchecked(#bytes_ident: &[u8]) -> Result<(), ::flatty::Error> {
                    #code
                }
            }

            unsafe impl ::flatty::Flat for #tag {}
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
