mod parts;
mod utils;

mod base;
mod sized;
mod unsized_enum;
mod unsized_struct;

use proc_macro::TokenStream;

#[proc_macro_derive(FlatSized)]
pub fn derive_flat_sized(stream: TokenStream) -> TokenStream {
    sized::derive(stream)
}

#[proc_macro_derive(FlatUnsized)]
pub fn derive_flat_unsized(stream: TokenStream) -> TokenStream {
    unsized_struct::derive(stream)
}

#[proc_macro_attribute]
pub fn make_flat(attr: TokenStream, item: TokenStream) -> TokenStream {
    base::make(attr, item)
}

#[proc_macro_attribute]
pub fn make_flat_unsized_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    unsized_enum::make(attr, item)
}
