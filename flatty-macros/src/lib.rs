mod generic;
mod sized;
//mod unsized_enum;
//mod unsized_struct;
mod utils;

use proc_macro::TokenStream;

#[proc_macro_derive(Flat)]
pub fn derive_flat_sized(stream: TokenStream) -> TokenStream {
    sized::derive(stream)
}
