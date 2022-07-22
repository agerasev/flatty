use crate::parts::{
    bounds::{self, where_},
    validate,
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, DeriveInput};

pub fn derive(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);

    let ident = &input.ident;
    let where_clause = where_(bounds::make(&input, quote! { ::flatty::FlatSized }, None));
    let pre_validate = validate::make_pre(&input);
    let post_validate = validate::make_post(&input);

    let expanded = quote! {
        unsafe impl ::flatty::Flat for #ident #where_clause {}

        impl ::flatty::FlatInit for #ident #where_clause {
            type Init = Self;
            unsafe fn init_unchecked(mem: &mut [u8], init: Self::Init) -> &mut Self {
                let self_ = Self::interpret_mut_unchecked(mem);
                // Dirty hack because the compiler cannot prove that `Self::Init` is the same as `Self`.
                *self_ = core::ptr::read(&init as *const _ as *const Self);
                self_
            }
            fn pre_validate(mem: &[u8]) -> Result<(), ::flatty::InterpretError> {
                #pre_validate
            }
            fn post_validate(&self) -> Result<(), ::flatty::InterpretError> {
                #post_validate
            }
            unsafe fn interpret_unchecked(mem: &[u8]) -> &Self {
                &*(mem.as_ptr() as *const Self)
            }
            unsafe fn interpret_mut_unchecked(mem: &mut [u8]) -> &mut Self {
                &mut *(mem.as_mut_ptr() as *mut Self)
            }
        }
    };

    TokenStream::from(expanded)
}
