use crate::parts::attrs;
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, Data, DeriveInput};

pub fn apply(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let info = parse_macro_input!(attr as attrs::MakeFlatInfo);
    let input = parse_macro_input!(stream as DeriveInput);

    match input.data {
        Data::Struct(_) => assert!(
            info.enum_type.is_none(),
            "`enum_type` is not allowed for struct"
        ),
        Data::Enum(_) => assert!(info.enum_type.is_some(), "`enum_type` must be set for enum"),
        Data::Union(_) => assert!(
            info.enum_type.is_none(),
            "`enum_type` is not allowed for union"
        ),
    };

    let trait_ = if info.sized {
        quote! { FlatSized }
    } else {
        quote! { FlatUnsized }
    };
    let enum_type = match info.enum_type {
        Some(ty) => quote! { , #ty },
        None => quote! {},
    };

    let expanded = quote! {
        #[derive(#trait_)]
        #[repr(C #enum_type)]
        #input
    };

    TokenStream::from(expanded)
}
