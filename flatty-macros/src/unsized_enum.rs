use crate::parts::{
    align_as, attrs,
    bounds::{self, where_},
    enum_, init, layout, validate,
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
    let (ref_ident, ref_contents) = enum_::make_ref(&input);
    let (mut_ident, mut_contents) = enum_::make_mut(&input);

    /*
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
    */

    let expanded = quote! {
        #[repr(#enum_ty)]
        #vis enum #state_ident {
            #state_contents
        }

        #[repr(C)]
        #vis struct #ident {
            state: #state_ident,
            //_align: [<Self as ::flatty::FlatUnsized>::AlignAs; 0],
            data: [u8],
        }

        #vis enum #ref_ident<'a> {
            #ref_contents
        }

        #vis enum #mut_ident<'a> {
            #mut_contents
        }
        /*
        impl UnsizedEnum {
            const DATA_OFFSET: usize = max(u8::SIZE, Self::ALIGN);

            pub fn as_ref(&self) -> UnsizedEnumRef<'_> {
                match self.state {
                    UnsizedEnumState::A => UnsizedEnumRef::A,
                    UnsizedEnumState::B => {
                        UnsizedEnumRef::B(unsafe { i32::interpret_unchecked(&self.data) })
                    }
                    UnsizedEnumState::C => {
                        UnsizedEnumRef::C(unsafe { FlatVec::<u8>::interpret_unchecked(&self.data) })
                    }
                }
            }

            pub fn as_mut(&mut self) -> UnsizedEnumMut<'_> {
                match self.state {
                    UnsizedEnumState::A => UnsizedEnumMut::A,
                    UnsizedEnumState::B => {
                        UnsizedEnumMut::B(unsafe { i32::interpret_mut_unchecked(&mut self.data) })
                    }
                    UnsizedEnumState::C => {
                        UnsizedEnumMut::C(unsafe { FlatVec::<u8>::interpret_mut_unchecked(&mut self.data) })
                    }
                }
            }
        }

        #[allow(dead_code)]
        #[repr(u8)]
        pub enum UnsizedEnumAlignAs {
            A,
            B(<i32 as FlatUnsized>::AlignAs),
            C(<FlatVec<u8> as FlatUnsized>::AlignAs),
        }

        impl FlatBase for UnsizedEnum {
            const ALIGN: usize = max(u8::ALIGN, max(i32::ALIGN, <FlatVec<u8>>::ALIGN));

            const MIN_SIZE: usize = Self::DATA_OFFSET + min(0, min(i32::MIN_SIZE, FlatVec::<u8>::MIN_SIZE));
            fn size(&self) -> usize {
                Self::DATA_OFFSET
                    + match self.as_ref() {
                        UnsizedEnumRef::A => 0,
                        UnsizedEnumRef::B(r) => r.size(),
                        UnsizedEnumRef::C(r) => r.size(),
                    }
            }
        }

        impl FlatUnsized for UnsizedEnum {
            type AlignAs = UnsizedEnumAlignAs;

            fn ptr_metadata(mem: &[u8]) -> usize {
                mem.len() - Self::DATA_OFFSET
            }
        }

        pub enum UnsizedEnumInit {
            A,
            B(<i32 as FlatInit>::Init),
            C(<FlatVec<u8> as FlatInit>::Init),
        }

        impl FlatInit for UnsizedEnum {
            type Init = UnsizedEnumInit;
            unsafe fn init_unchecked(mem: &mut [u8], init: Self::Init) -> &mut Self {
                let self_ = Self::interpret_mut_unchecked(mem);
                match init {
                    UnsizedEnumInit::A => {
                        self_.state = UnsizedEnumState::A;
                    }
                    UnsizedEnumInit::B(inner_init) => {
                        self_.state = UnsizedEnumState::B;
                        i32::init_unchecked(&mut self_.data, inner_init);
                    }
                    UnsizedEnumInit::C(inner_init) => {
                        self_.state = UnsizedEnumState::C;
                        <FlatVec<u8>>::init_unchecked(&mut self_.data, inner_init);
                    }
                }
                self_
            }

            fn pre_validate(mem: &[u8]) -> Result<(), InterpretError> {
                if *u8::interpret(mem).unwrap() >= 3 {
                    Err(InterpretError::InvalidState)
                } else {
                    Ok(())
                }
            }
            fn post_validate(&self) -> Result<(), InterpretError> {
                match &self.state {
                    UnsizedEnumState::A => Ok(()),
                    UnsizedEnumState::B => {
                        if self.data.len() < i32::MIN_SIZE {
                            return Err(InterpretError::InsufficientSize);
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
                            return Err(InterpretError::InsufficientSize);
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

            unsafe fn interpret_unchecked(mem: &[u8]) -> &Self {
                let slice = from_raw_parts(mem.as_ptr(), Self::ptr_metadata(mem));
                &*(slice as *const [_] as *const Self)
            }
            unsafe fn interpret_mut_unchecked(mem: &mut [u8]) -> &mut Self {
                let slice = from_raw_parts_mut(mem.as_mut_ptr(), Self::ptr_metadata(mem));
                &mut *(slice as *mut [_] as *mut Self)
            }
        }

        */

        /*
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

            unsafe fn init_unchecked(mem: &mut [u8], init: Self::Init) -> &mut Self {
                #init_fn
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
        */
    };

    TokenStream::from(expanded)
}
