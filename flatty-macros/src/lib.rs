mod parts;
mod utils;

mod sized;
mod unsized_struct;
//mod unsized_enum;

use proc_macro::TokenStream;

#[proc_macro_derive(FlatSized)]
pub fn derive_flat_sized(stream: TokenStream) -> TokenStream {
    sized::derive(stream)
}

#[proc_macro_derive(FlatUnsized)]
pub fn derive_flat_unsized(stream: TokenStream) -> TokenStream {
    unsized_struct::derive(stream)
}
