use crate::{utils::generic, Context};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn impl_(ctx: &Context, input: &DeriveInput) -> TokenStream {
    let self_ident = &input.ident;

    let generics = &input.generics;
    let generic_params = generic::make_params(generics);
    let where_clause = generic::make_bounds(
        input,
        quote! { ::flatty::Flat + Sized },
        if ctx.info.sized {
            None
        } else {
            Some(quote! { ::flatty::Flat })
        },
    );

    quote! {
        unsafe impl<#generics> ::flatty::Flat for #self_ident<#generic_params>
        where
            #where_clause
        {
        }
    }
}
