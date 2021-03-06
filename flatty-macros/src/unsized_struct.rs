use crate::parts::{
    align_as, attrs,
    bounds::{self, where_},
    init, layout, validate,
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, DeriveInput};

pub fn derive(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    attrs::validate_repr(&input);

    let vis = &input.vis;
    let ident = &input.ident;

    let where_clause = where_(bounds::make(
        &input,
        quote! { ::flatty::FlatSized },
        Some(quote! { ::flatty::Flat }),
    ));

    let align = layout::make_align(&input);
    let min_size = layout::make_min_size(&input);
    let size = layout::make_size(&input);

    let (align_as_ident, align_as_contents) = align_as::make(&input);
    let ptr_metadata = layout::make_ptr_metadata(&input);

    let (init_ident, init_body) = init::make_type(&input);
    let init_fn = init::make(&input);
    let pre_validate = validate::make_pre(&input);
    let post_validate = validate::make_post(&input);

    let expanded = quote! {
        unsafe impl ::flatty::Flat for #ident #where_clause {}

        #[allow(dead_code)]
        #[repr(C)]
        #vis struct #align_as_ident ( #align_as_contents );

        impl ::flatty::FlatBase for #ident #where_clause {
            const ALIGN: usize = #align;

            const MIN_SIZE: usize = #min_size;
            fn size(&self) -> usize {
                #size
            }
        }

        impl ::flatty::FlatUnsized for #ident #where_clause {
            type AlignAs = #align_as_ident;

            fn ptr_metadata(mem: &[u8]) -> usize {
                #ptr_metadata
            }
        }

        //#[derive(Default)]
        #vis struct #init_ident #init_body

        impl ::flatty::FlatInit for #ident #where_clause {
            type Init = #init_ident;

            unsafe fn placement_new_unchecked(mem: &mut [u8], init: Self::Init) -> &mut Self {
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
    };

    TokenStream::from(expanded)
}
