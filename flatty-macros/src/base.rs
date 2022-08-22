use crate::parts::attrs;
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, Data, DeriveInput};

pub fn make(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let info = parse_macro_input!(attr as attrs::make_flat::MakeFlatInfo);
    let input = parse_macro_input!(stream as DeriveInput);

    let enum_type = match input.data {
        Data::Struct(_) | Data::Union(_) => {
            assert!(info.enum_type.is_none(), "`enum_type` is not allowed here");
            quote! {}
        }
        Data::Enum(_) => match info.enum_type {
            Some(ty) => quote! { , #ty },
            None => quote! { , u8 },
        },
    };

    let derive = match info.sized {
        true => quote! { #[derive(::flatty::macros::FlatSized)] },
        false => match &input.data {
            Data::Enum(_) => quote! { #[::flatty::macros::make_flat_unsized_enum] },
            Data::Struct(_) => quote! { #[::flatty::macros::make_flat_unsized_struct] },
            _ => panic!(),
        },
    };

    let expanded = quote! {
        #derive
        #[repr(C #enum_type)]
        #input
    };

    TokenStream::from(expanded)
}
