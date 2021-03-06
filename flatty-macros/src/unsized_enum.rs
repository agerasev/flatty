use crate::parts::{
    align_as, attrs,
    bounds::{self, where_},
    enum_, init, layout,
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, DeriveInput};

pub fn make(attr: TokenStream, stream: TokenStream) -> TokenStream {
    assert!(attr.is_empty());
    let input = parse_macro_input!(stream as DeriveInput);
    attrs::validate_repr(&input);

    let vis = &input.vis;
    let ident = &input.ident;

    let enum_ty = attrs::get_enum_type(&input);
    let (state_ident, state_contents) = enum_::make_state(&input);

    let where_clause = where_(bounds::make(
        &input,
        quote! { ::flatty::FlatSized },
        Some(quote! { ::flatty::Flat }),
    ));
    let align = layout::make_align(&input);
    let (vars_count, var_min_sizes) = enum_::make_var_min_sizes(&input);
    let min_size = enum_::make_min_size(&input);
    let size = enum_::make_size(&input);

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
        #vis struct #ident {
            state: #state_ident,
            _align: [<Self as ::flatty::FlatUnsized>::AlignAs; 0],
            data: [u8],
        }

        #vis enum #ref_ident<'a> {
            #ref_contents
        }

        #vis enum #mut_ident<'a> {
            #mut_contents
        }

        #[allow(dead_code)]
        #[repr(C)]
        #vis struct #align_as_ident ( #align_as_contents );

        impl #ident #where_clause {
            const DATA_OFFSET: usize = ::flatty::utils::max(<#enum_ty as ::flatty::FlatSized>::SIZE, <Self as ::flatty::FlatBase>::ALIGN);

            const VAR_MIN_SIZES: [usize; #vars_count] = #var_min_sizes;

            #[allow(clippy::eval_order_dependence)]
            pub fn as_ref(&self) -> UnsizedEnumRef<'_> {
                #as_ref_ident
            }

            #[allow(clippy::eval_order_dependence)]
            pub fn as_mut(&mut self) -> UnsizedEnumMut<'_> {
                #as_mut_ident
            }
        }

        impl ::flatty::FlatBase for #ident #where_clause {
            const ALIGN: usize = #align;

            const MIN_SIZE: usize = #min_size;

            #[allow(unused_variables)]
            fn size(&self) -> usize {
                #size
            }
        }

        impl ::flatty::FlatUnsized for #ident #where_clause {
            type AlignAs = #align_as_ident;

            fn ptr_metadata(mem: &[u8]) -> usize {
                mem.len() - Self::DATA_OFFSET
            }
        }

        #vis enum #init_ident #init_body

        impl ::flatty::FlatInit for #ident #where_clause {
            type Init = #init_ident;

            unsafe fn placement_new_unchecked(mem: &mut [u8], init: Self::Init) -> &mut Self {
                #init_fn
            }

            #[allow(unused_variables)]
            fn placement_new(mem: &mut [u8], init: Self::Init) -> Result<&mut Self, ::flatty::Error> {
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

        unsafe impl ::flatty::Flat for #ident #where_clause {}

        /*
        impl FlatInit for UnsizedEnum {
            type Init = UnsizedEnumInit;
            unsafe fn placement_new_unchecked(mem: &mut [u8], init: Self::Init) -> &mut Self {
                let self_ = Self::reinterpret_mut_unchecked(mem);
                match init {
                    UnsizedEnumInit::A => {
                        self_.state = UnsizedEnumState::A;
                    }
                    UnsizedEnumInit::B(inner_init) => {
                        self_.state = UnsizedEnumState::B;
                        i32::placement_new_unchecked(&mut self_.data, inner_init);
                    }
                    UnsizedEnumInit::C(inner_init) => {
                        self_.state = UnsizedEnumState::C;
                        <FlatVec<u8>>::placement_new_unchecked(&mut self_.data, inner_init);
                    }
                }
                self_
            }

            fn pre_validate(mem: &[u8]) -> Result<(), Error> {
                if *u8::reinterpret(mem).unwrap() >= 3 {
                    Err(Error::InvalidState)
                } else {
                    Ok(())
                }
            }
            fn post_validate(&self) -> Result<(), Error> {
                match &self.state {
                    UnsizedEnumState::A => Ok(()),
                    UnsizedEnumState::B => {
                        if self.data.len() < i32::MIN_SIZE {
                            return Err(Error::InsufficientSize);
                        }
                        i32::pre_validate(&self.data)?;
                        if let UnsizedEnumRef::B(inner) = self.as_ref() {
                            inner.post_validate()
                        } else {
                            unreachable!();
                        }
                    }
                    UnsizedEnumState::C => {
                        if self.data.len() < FlatVec::<u8>::MIN_SIZE {
                            return Err(Error::InsufficientSize);
                        }
                        <FlatVec<u8>>::pre_validate(&self.data)?;
                        if let UnsizedEnumRef::C(inner) = self.as_ref() {
                            inner.post_validate()
                        } else {
                            unreachable!();
                        }
                    }
                }
            }

            unsafe fn reinterpret_unchecked(mem: &[u8]) -> &Self {
                let slice = from_raw_parts(mem.as_ptr(), Self::ptr_metadata(mem));
                &*(slice as *const [_] as *const Self)
            }
            unsafe fn reinterpret_mut_unchecked(mem: &mut [u8]) -> &mut Self {
                let slice = from_raw_parts_mut(mem.as_mut_ptr(), Self::ptr_metadata(mem));
                &mut *(slice as *mut [_] as *mut Self)
            }
        }
        */
    };

    TokenStream::from(expanded)
}
