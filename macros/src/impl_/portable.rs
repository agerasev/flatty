use crate::{utils::generic, Context};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn impl_(_ctx: &Context, input: &DeriveInput) -> TokenStream {
    let self_ident = &input.ident;

    let generic_params = &input.generics.params;
    let generic_args = generic::args(&input.generics);
    let where_clause = generic::where_clause(input, quote! { ::flatty::Portable }, None);

    quote! {
        unsafe impl<#generic_params> ::flatty::Portable for #self_ident<#generic_args>
        #where_clause
        {
        }
    }
}
