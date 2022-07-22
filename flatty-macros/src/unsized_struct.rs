use crate::parts::{
    align_as,
    bounds::{self, where_},
    init, layout, validate,
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, DeriveInput};

pub fn derive(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);

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
    let pre_validate = validate::make_pre(&input);
    let post_validate = validate::make_post(&input);

    let expanded = quote! {
        /*
        unsafe impl ::flatty::Flat for #ident #where_clause {}
        */

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

            unsafe fn init_unchecked(mem: &mut [u8], init: Self::Init) -> &mut Self {
                /*
                let mut offset = 0;
                <u8 as FlatInit>::init_unchecked(&mut mem[offset..], init.a);
                offset = upper_multiple(offset + <u8>::SIZE, u16::ALIGN);
                <u16 as FlatInit>::init_unchecked(&mut mem[offset..], init.b);
                offset = upper_multiple(offset + <u16>::SIZE, <FlatVec<u64>>::ALIGN);
                <FlatVec<u64> as FlatInit>::init_unchecked(&mut mem[offset..], init.c);
                */
                // TODO: Implement
                Self::interpret_mut_unchecked(mem)
            }

            fn pre_validate(mem: &[u8]) -> Result<(), ::flatty::InterpretError> {
                #pre_validate
            }
            fn post_validate(&self) -> Result<(), ::flatty::InterpretError> {
                #post_validate
            }

            unsafe fn interpret_unchecked(mem: &[u8]) -> &Self {
                let slice = ::core::slice::from_raw_parts(mem.as_ptr(), Self::ptr_metadata(mem));
                &*(slice as *const [_] as *const Self)
            }
            unsafe fn interpret_mut_unchecked(mem: &mut [u8]) -> &mut Self {
                let slice = ::core::slice::from_raw_parts_mut(mem.as_mut_ptr(), Self::ptr_metadata(mem));
                &mut *(slice as *mut [_] as *mut Self)
            }
        }
    };

    TokenStream::from(expanded)
}
