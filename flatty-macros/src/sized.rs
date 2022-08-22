use crate::parts::{
    attrs,
    generic::{self, where_},
    validate,
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, DeriveInput};

pub fn derive(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    attrs::repr::validate(&input);

    let ident = &input.ident;
    let (params, bindings) = generic::make_params(&input);
    let where_clause = where_(generic::make_bounds(
        &input,
        quote! { ::flatty::FlatSized },
        None,
    ));
    let pre_validate = validate::make_pre(&input);
    let post_validate = validate::make_post(&input);

    let expanded = quote! {
        impl<#bindings> ::flatty::FlatInit for #ident<#params> #where_clause {
            type Init = Self;
            unsafe fn placement_new_unchecked(mem: &mut [u8], init: Self::Init) -> &mut Self {
                let self_ = Self::reinterpret_mut_unchecked(mem);
                // Dirty hack because the compiler cannot prove that `Self::Init` is the same as `Self`.
                *self_ = ::core::ptr::read(&init as *const _ as *const Self);
                self_
            }
            fn pre_validate(mem: &[u8]) -> Result<(), ::flatty::Error> {
                #pre_validate
            }
            fn post_validate(&self) -> Result<(), ::flatty::Error> {
                #post_validate
            }
            unsafe fn reinterpret_unchecked(mem: &[u8]) -> &Self {
                &*(mem.as_ptr() as *const Self)
            }
            unsafe fn reinterpret_mut_unchecked(mem: &mut [u8]) -> &mut Self {
                &mut *(mem.as_mut_ptr() as *mut Self)
            }
        }

        unsafe impl<#bindings> ::flatty::Flat for #ident<#params> #where_clause {}
    };

    TokenStream::from(expanded)
}
