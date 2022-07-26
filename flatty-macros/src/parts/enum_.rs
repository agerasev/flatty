use crate::{
    parts::{init, layout, match_, validate},
    utils::fields_iter::FieldsIter,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{self, parse2, Data, DeriveInput, Fields, Ident, Index, Type};

fn state_ident(input: &DeriveInput) -> Ident {
    Ident::new(&format!("{}State", input.ident), input.ident.span())
}

pub fn make_state(input: &DeriveInput) -> (Ident, TokenStream2) {
    let ident = state_ident(input);
    let contents = match &input.data {
        Data::Struct(_) | Data::Union(_) => unimplemented!(),
        Data::Enum(enum_data) => enum_data.variants.iter().fold(quote! {}, |accum, variant| {
            let var_ident = &variant.ident;
            quote! {
                #accum
                #var_ident,
            }
        }),
    };
    (ident, contents)
}

fn make_mapped<F: Fn(&Type) -> Type>(input: &DeriveInput, map_ty: F) -> TokenStream2 {
    let contents = match &input.data {
        Data::Struct(_) | Data::Union(_) => unimplemented!(),
        Data::Enum(enum_data) => enum_data.variants.iter().fold(quote! {}, |accum, variant| {
            let var_ident = &variant.ident;
            let var_body = match &variant.fields {
                Fields::Named(fields) => {
                    let items = fields.named.iter().fold(quote! {}, |accum, field| {
                        let ty = map_ty(&field.ty);
                        let ident = field.ident.as_ref().unwrap();
                        quote! { #accum #ident: #ty, }
                    });
                    quote! { { #items } }
                }
                Fields::Unnamed(fields) => {
                    let items = fields.unnamed.iter().fold(quote! {}, |accum, field| {
                        let ty = map_ty(&field.ty);
                        quote! { #accum #ty, }
                    });
                    quote! { (#items) }
                }
                Fields::Unit => {
                    quote! {}
                }
            };
            quote! {
                #accum
                #var_ident #var_body,
            }
        }),
    };
    contents
}

fn ref_ident(input: &DeriveInput) -> Ident {
    Ident::new(&format!("{}Ref", input.ident), input.ident.span())
}

fn mut_ident(input: &DeriveInput) -> Ident {
    Ident::new(&format!("{}Mut", input.ident), input.ident.span())
}

pub fn make_ref(input: &DeriveInput) -> (Ident, TokenStream2) {
    let ident = ref_ident(input);
    let contents = make_mapped(input, |ty| {
        let stream = quote! { &'a #ty };
        parse2::<Type>(stream).unwrap()
    });
    (ident, contents)
}

pub fn make_mut(input: &DeriveInput) -> (Ident, TokenStream2) {
    let ident = mut_ident(input);
    let contents = make_mapped(input, |ty| {
        let stream = quote! { &'a mut #ty };
        parse2::<Type>(stream).unwrap()
    });
    (ident, contents)
}

pub fn make_as_gen<F: Fn(TokenStream2) -> TokenStream2>(
    input: &DeriveInput,
    ref_ident: &Ident,
    read_fn: &TokenStream2,
    map_data: F,
    split_fn: &TokenStream2,
) -> TokenStream2 {
    let state_ident = state_ident(input);
    let contents = match &input.data {
        Data::Struct(_) | Data::Union(_) => unimplemented!(),
        Data::Enum(enum_data) => enum_data.variants.iter().fold(quote! {}, |accum, variant| {
            let var_ident = &variant.ident;
            let iter = variant.fields.fields_iter();
            let len = iter.len();
            let items =
                iter.enumerate()
                    .fold(quote! {}, |accum, (i, field)| {
                        let ty = &field.ty;
                        let init = match &field.ident {
                            Some(fi) => quote!{ #fi: },
                            None => quote!{},
                        };
                        let data = map_data(quote!{ data[(offset - last_offset)..] });
                        let (split, add_size) = if i + 1 < len {
                            let size = quote!{ <#ty as ::flatty::FlatSized>::SIZE };
                            (
                                quote!{
                                    let (last_data, next_data) = (#data).#split_fn(#size);
                                    data = next_data;
                                },
                                quote! {
                                    offset += #size;
                                    last_offset = offset;
                                },
                            )
                        } else {
                            (quote! { let last_data = #data; }, quote!{})
                        };
                        quote! {
                            #accum
                            #init {
                                offset = ::flatty::utils::upper_multiple(offset, <#ty as ::flatty::FlatBase>::ALIGN);
                                #split
                                let tmp = unsafe { <#ty as ::flatty::FlatInit>::#read_fn(last_data) };
                                #add_size
                                tmp
                            },
                        }
                    });
            let var_body = match &variant.fields {
                Fields::Named(_) => {
                    quote! { { #items } }
                }
                Fields::Unnamed(_) => {
                    quote! { (#items) }
                }
                Fields::Unit => {
                    assert!(items.is_empty());
                    quote! {}
                }
            };
            quote! {
                #accum
                #state_ident::#var_ident => #ref_ident::#var_ident #var_body,
            }
        }),
    };
    let data = map_data(quote! { self.data });
    quote! {
        let mut data = #data;
        let mut offset: usize = 0;
        let mut last_offset = offset;
        match self.state {
            #contents
        }
    }
}

pub fn make_as_ref(input: &DeriveInput) -> TokenStream2 {
    make_as_gen(
        input,
        &ref_ident(input),
        &quote! { reinterpret_unchecked },
        |slice| {
            quote! { &#slice }
        },
        &quote! { split_at },
    )
}

pub fn make_as_mut(input: &DeriveInput) -> TokenStream2 {
    make_as_gen(
        input,
        &mut_ident(input),
        &quote! { reinterpret_mut_unchecked },
        |slice| {
            quote! { &mut #slice }
        },
        &quote! { split_at_mut },
    )
}

pub fn make_size(input: &DeriveInput) -> TokenStream2 {
    layout::make_size_gen(input, &ref_ident(input), quote! { self.as_ref() })
}

pub fn make_var_min_sizes(input: &DeriveInput) -> (usize, TokenStream2) {
    match &input.data {
        Data::Enum(enum_data) => {
            let count = enum_data.variants.len();
            let items = enum_data.variants.iter().fold(quote! {}, |accum, variant| {
                let var_min_size = layout::make_min_size_fields(&variant.fields);
                quote! { #accum #var_min_size, }
            });
            (count, quote! { [ #items ] })
        }
        Data::Struct(_) | Data::Union(_) => unimplemented!(),
    }
}

pub fn make_min_size(input: &DeriveInput) -> TokenStream2 {
    match &input.data {
        Data::Struct(_) | Data::Union(_) => unimplemented!(),
        Data::Enum(enum_data) => {
            let items = enum_data.variants.iter().enumerate().fold(
                quote! { usize::MAX },
                |accum, (i, _)| {
                    let index = Index::from(i);
                    let var_min_size = quote! { Self::VAR_MIN_SIZES[#index] };
                    quote! { ::flatty::utils::min(#accum, #var_min_size) }
                },
            );
            quote! {
                ::flatty::utils::upper_multiple(
                    Self::DATA_OFFSET + #items,
                    Self::ALIGN,
                )
            }
        }
    }
}

pub fn make_init_checked(input: &DeriveInput) -> TokenStream2 {
    let ident = init::type_ident(input);
    let body = match &input.data {
        Data::Struct(_) | Data::Union(_) => unimplemented!(),
        Data::Enum(enum_data) => {
            let contents =
                enum_data
                    .variants
                    .iter()
                    .enumerate()
                    .fold(quote! {}, |accum, (i, variant)| {
                        let var_ident = &variant.ident;
                        let index = Index::from(i);
                        let bs = match_::make_bindings(&variant.fields);
                        let bindings = bs.bindings;
                        quote! {
                            #accum
                            #ident::#var_ident #bindings => {
                                if mem.len() < Self::DATA_OFFSET + Self::VAR_MIN_SIZES[#index] {
                                    return Err(::flatty::Error::InsufficientSize)
                                }
                            }
                        }
                    });
            quote! {
                match &init {
                    #contents
                }
            }
        }
    };
    quote! {
        <Self as ::flatty::FlatBase>::check_size_and_align(mem)?;
        #body
        Ok(unsafe { Self::placement_new_unchecked(mem, init) })
    }
}

pub fn make_pre_validate(input: &DeriveInput) -> TokenStream2 {
    validate::make_pre_gen(
        input,
        quote! {
            if mem.len() < Self::DATA_OFFSET + Self::VAR_MIN_SIZES[*state as usize] {
                return Err(::flatty::Error::InsufficientSize)
            }
        },
    )
}

pub fn make_post_validate(input: &DeriveInput) -> TokenStream2 {
    validate::make_post_gen(input, &ref_ident(input), quote! { self.as_ref() })
}
