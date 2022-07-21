mod parts;
mod utils;

mod sized;
//mod unsized_enum;
//mod unsized_struct;

use proc_macro::TokenStream;

#[proc_macro_derive(FlatSized)]
pub fn derive_flat_sized(stream: TokenStream) -> TokenStream {
    sized::derive(stream)
}
