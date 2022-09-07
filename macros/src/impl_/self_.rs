use super::tag::impl_ as tag_impl;
use crate::{utils::generic, Context};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn impl_(ctx: &Context, input: &DeriveInput) -> TokenStream {
    let self_ident = &input.ident;

    let generic_params = &input.generics.params;
    let generic_args = generic::args(&input.generics);
    let where_clause = &input.generics.where_clause;

    let mut items = quote! {};
    let mut extras = quote! {};

    if let Data::Enum(..) = &input.data {
        let enum_type = ctx.info.enum_type.as_ref().unwrap();
        items = quote! {
            #items
            const DATA_OFFSET: usize = ::flatty::utils::ceil_mul(<#enum_type as ::flatty::FlatSized>::SIZE, <Self as ::flatty::FlatBase>::ALIGN);
        };

        let tag_impl = tag_impl(ctx, input, ctx.info.sized);
        extras = quote! {
            #extras
            #tag_impl
        }
    }

    quote! {
        impl<#generic_params> #self_ident<#generic_args>
        where
            #where_clause
        {
            #items
        }

        #extras
    }
}
