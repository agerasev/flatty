use crate::parts::generic::{self, where_};
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, DeriveInput};

pub fn derive(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);

    let ident = &input.ident;
    let (params, bindings) = generic::make_params(&input);
    let where_clause = where_(generic::make_bounds(
        &input,
        quote! { ::flatty::Portable },
        None,
    ));

    let expanded = quote! {
        unsafe impl<#bindings> ::flatty::Portable for #ident<#params> #where_clause {}
    };

    TokenStream::from(expanded)
}
