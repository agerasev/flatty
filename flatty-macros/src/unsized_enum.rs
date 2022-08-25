use crate::parts::{
    align_as, attrs, enum_,
    generic::{self, where_},
    init, layout,
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, DeriveInput};

pub fn make(attr: TokenStream, stream: TokenStream) -> TokenStream {
    assert!(attr.is_empty());
    let input = parse_macro_input!(stream as DeriveInput);
    attrs::repr::validate(&input);

    let vis = &input.vis;
    let ident = &input.ident;

    let enum_ty = attrs::repr::get_enum_type(&input);
    let (state_ident, state_contents) = enum_::make_state(&input);

    let derive_default = if attrs::default::has(&input) {
        quote! { #[derive(Default)] }
    } else {
        quote! {}
    };

    let (params, bindings) = generic::make_params(&input);
    let where_clause = where_(generic::make_bounds(
        &input,
        quote! { ::flatty::FlatSized },
        Some(quote! { ::flatty::Flat }),
    ));
    let align = layout::make_align(&input);
    let (vars_count, var_min_sizes) = enum_::make_var_min_sizes(&input);
    let min_size = enum_::make_min_size(&input);
    let size = enum_::make_size(&input);
    let size_of = enum_::make_size_of(&input);

    let (ref_ident, ref_contents) = enum_::make_ref(&input);
    let (mut_ident, mut_contents) = enum_::make_mut(&input);
    let as_ref_ident = enum_::make_as_ref(&input);
    let as_mut_ident = enum_::make_as_mut(&input);

    let (align_as_ident, align_as_contents) = align_as::make(&input);

    let (init_ident, init_body) = init::make_type(&input);

    let init_fn = init::make(&input);
    let init_fn_checked = enum_::make_init_checked(&input);
    let pre_validate = enum_::make_pre_validate(&input);
    let post_validate = enum_::make_post_validate(&input);

    let expanded = quote! {
        #[allow(dead_code)]
        #[repr(#enum_ty)]
        #vis enum #state_ident {
            #state_contents
        }

        #[repr(C)]
        #vis struct #ident<#bindings> {
            state: #state_ident,
            _align: [<Self as ::flatty::FlatUnsized>::AlignAs; 0],
            _phantom: ::core::marker::PhantomData<#init_ident<#params>>,
            data: [u8],
        }

        #vis enum #ref_ident<'__flatty_a, #bindings> {
            #ref_contents
        }

        #vis enum #mut_ident<'__flatty_a, #bindings> {
            #mut_contents
        }

        #[allow(dead_code)]
        #[repr(C)]
        #vis struct #align_as_ident<#bindings> ( #align_as_contents );

        impl<#bindings> #ident<#params> #where_clause {
            const DATA_OFFSET: usize = ::flatty::utils::max(<#enum_ty as ::flatty::FlatSized>::SIZE, <Self as ::flatty::FlatBase>::ALIGN);

            const VAR_MIN_SIZES: [usize; #vars_count] = #var_min_sizes;

            #[allow(clippy::eval_order_dependence)]
            pub fn as_ref(&self) -> #ref_ident<'_, #params> {
                #as_ref_ident
            }

            #[allow(clippy::eval_order_dependence)]
            pub fn as_mut(&mut self) -> #mut_ident<'_, #params> {
                #as_mut_ident
            }
        }

        impl<#bindings> ::flatty::FlatBase for #ident<#params> #where_clause {
            const ALIGN: usize = #align;

            const MIN_SIZE: usize = #min_size;

            #[allow(unused_variables)]
            fn size(&self) -> usize {
                #size
            }
        }

        impl<#bindings> ::flatty::FlatUnsized for #ident<#params> #where_clause {
            type AlignAs = #align_as_ident<#params>;

            fn ptr_metadata(mem: &[u8]) -> usize {
                ::flatty::utils::lower_multiple(mem.len() - Self::DATA_OFFSET, <Self as ::flatty::FlatBase>::ALIGN)
            }
        }

        #derive_default
        #vis enum #init_ident<#bindings> #init_body

        impl<#bindings> ::flatty::FlatInit for #ident<#params> #where_clause {
            type Dyn = #init_ident<#params>;
            #[allow(unused_variables)]
            fn size_of(value: &Self::Dyn) -> usize {
                #size_of
            }

            unsafe fn placement_new_unchecked<'__flatty_a, '__flatty_b>(mem: &'__flatty_a mut [u8], init: &'__flatty_b Self::Dyn) -> &'__flatty_a mut Self {
                #init_fn
            }

            #[allow(unused_variables)]
            fn placement_new<'__flatty_a, '__flatty_b>(mem: &'__flatty_a mut [u8], init: &'__flatty_b Self::Dyn) -> Result<&'__flatty_a mut Self, ::flatty::Error> {
                #init_fn_checked
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
