use crate::parts::{
    align_as, attrs,
    generic::{self, where_},
    init, layout, validate,
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, DeriveInput};

pub fn make(_attr: TokenStream, stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    attrs::repr::validate(&input);

    let vis = &input.vis;
    let ident = &input.ident;

    let derive_default = if attrs::default::has(&input) {
        quote! { #[derive(Default)] }
    } else {
        quote! {}
    };
    let input_without_default = attrs::default::filter_out(&input);

    let (params, bindings) = generic::make_params(&input);
    let where_clause = where_(generic::make_bounds(
        &input,
        quote! { ::flatty::FlatSized },
        Some(quote! { ::flatty::Flat }),
    ));

    let align = layout::make_align(&input);
    let min_size = layout::make_min_size(&input);
    let size = layout::make_size(&input);
    let size_of = layout::make_size_of(&input);

    let (align_as_ident, align_as_contents) = align_as::make(&input);
    let ptr_metadata = layout::make_ptr_metadata(&input);

    let (init_ident, init_body) = init::make_type(&input);
    let init_fn = init::make(&input);
    let pre_validate = validate::make_pre(&input);
    let post_validate = validate::make_post(&input);

    let expanded = quote! {
        #input_without_default

        #[allow(dead_code)]
        #[repr(C)]
        #vis struct #align_as_ident<#bindings> ( #align_as_contents );

        impl<#bindings> ::flatty::FlatBase for #ident<#params> #where_clause {
            const ALIGN: usize = #align;

            const MIN_SIZE: usize = #min_size;
            fn size(&self) -> usize {
                #size
            }
        }

        impl<#bindings> ::flatty::FlatUnsized for #ident<#params> #where_clause {
            type AlignAs = #align_as_ident<#params>;

            fn ptr_metadata(mem: &[u8]) -> usize {
                #ptr_metadata
            }
        }

        #derive_default
        #vis struct #init_ident<#bindings> #init_body

        impl<#bindings> ::flatty::FlatInit for #ident<#params> #where_clause {
            type Dyn = #init_ident<#params>;
            fn size_of(value: &Self::Dyn) -> usize {
                #size_of
            }

            unsafe fn placement_new_unchecked<'__flatty_a, '__flatty_b>(mem: &'__flatty_a mut [u8], init: &'__flatty_b Self::Dyn) -> &'__flatty_a mut Self {
                #init_fn
            }

            fn pre_validate(mem: &[u8]) -> Result<(), ::flatty::Error> {
                #pre_validate
            }
            fn post_validate(&self) -> Result<(), ::flatty::Error> {
                #post_validate
            }

            unsafe fn reinterpret_unchecked(mem: &[u8]) -> &Self {
                let slice = ::core::slice::from_raw_parts(mem.as_ptr(), <Self as ::flatty::FlatUnsized>::ptr_metadata(mem));
                &*(slice as *const [_] as *const Self)
            }
            unsafe fn reinterpret_mut_unchecked(mem: &mut [u8]) -> &mut Self {
                let slice = ::core::slice::from_raw_parts_mut(mem.as_mut_ptr(), <Self as ::flatty::FlatUnsized>::ptr_metadata(mem));
                &mut *(slice as *mut [_] as *mut Self)
            }
        }

        unsafe impl<#bindings> ::flatty::Flat for #ident<#params> #where_clause {}
    };

    TokenStream::from(expanded)
}
