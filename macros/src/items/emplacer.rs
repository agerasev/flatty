use crate::Context;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn struct_(_ctx: &Context, _input: &DeriveInput) -> TokenStream {
    quote! {}
}
