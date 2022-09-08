use crate::Context;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn struct_(ctx: &Context, input: &DeriveInput) -> TokenStream {
    let vis = &input.vis;
    let self_ident = &input.ident;
    let tag_type = ctx.idents.tag.as_ref().unwrap();
    let align_as_type = ctx.idents.align_as.as_ref().unwrap();

    quote! {
        #[repr(C)]
        #vis struct #self_ident (
            tag: #tag_type,
            _align: [#align_as_type; 0],
            data: [u8],
        );
    }
}
