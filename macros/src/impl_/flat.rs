use crate::{utils::generic, Context};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn impl_(ctx: &Context, input: &DeriveInput) -> TokenStream {
    let self_ident = &input.ident;

    let generic_params = &input.generics.params;
    let generic_args = generic::make_args(&input.generics);
    let where_clause = generic::make_where_clause(
        input,
        quote! { ::flatty::Flat + Sized },
        if ctx.info.sized {
            None
        } else {
            Some(quote! { ::flatty::Flat })
        },
    );

    quote! {
        unsafe impl<#generic_params> ::flatty::Flat for #self_ident<#generic_args>
        where
            #where_clause
        {
        }
    }
}
