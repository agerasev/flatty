use crate::{
    parts::{attrs, match_},
    utils::fields_iter::FieldsIter,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{self, Data, DeriveInput, Fields, Ident, Index};

fn make_type_fields(fields: &Fields) -> TokenStream2 {
    match fields {
        Fields::Named(fields) => {
            let contents = fields.named.iter().fold(quote! {}, |accum, field| {
                let field_ty = &field.ty;
                let field_ident = field.ident.as_ref().unwrap();
                quote! { #accum #field_ident: <#field_ty as ::flatty::FlatInit>::Init, }
            });
            quote! { { #contents } }
        }
        Fields::Unnamed(fields) => {
            let contents = fields.unnamed.iter().fold(quote! {}, |accum, field| {
                let field_ty = &field.ty;
                assert!(field.ident.is_none());
                quote! { #accum <#field_ty as ::flatty::FlatInit>::Init, }
            });
            quote! { (#contents) }
        }
        Fields::Unit => quote! {},
    }
}

pub fn type_ident(input: &DeriveInput) -> Ident {
    Ident::new(&format!("{}Init", input.ident), input.ident.span())
}

pub fn make_type(input: &DeriveInput) -> (Ident, TokenStream2) {
    let ident = type_ident(input);
    let body = match &input.data {
        Data::Struct(struct_data) => {
            let body = make_type_fields(&struct_data.fields);
            let semi = match &struct_data.fields {
                Fields::Unnamed(_) | Fields::Unit => quote! { ; },
                Fields::Named(_) => quote! {},
            };
            quote! { #body #semi }
        }
        Data::Enum(enum_data) => {
            let contents = enum_data.variants.iter().fold(quote! {}, |accum, variant| {
                let var_body = make_type_fields(&variant.fields);
                let var_ident = &variant.ident;
                quote! {
                    #accum
                    #var_ident #var_body,
                }
            });
            quote! { { #contents }}
        }
        Data::Union(_union_data) => unimplemented!(),
    };
    (ident, body)
}

fn make_fields<FI: FieldsIter>(fields: &FI, prefix: TokenStream2) -> TokenStream2 {
    let iter = fields.fields_iter();
    let len = iter.len();
    iter.enumerate().fold(quote! {}, |accum, (i, field)| {
        let ty = &field.ty;
        let ident = match &field.ident {
            Some(x) => quote! { #x },
            None => {
                let index = Index::from(i);
                quote! { #index }
            }
        };
        let add_size = if i + 1 < len {
            quote! { offset += <#ty as ::flatty::FlatSized>::SIZE; }
        } else {
            quote! {}
        };
        quote! {
            #accum
            offset = ::flatty::utils::upper_multiple(offset, <#ty as ::flatty::FlatBase>::ALIGN);
            <#ty>::init_unchecked(&mut mem[offset..], #prefix #ident);
            #add_size
        }
    })
}

pub fn make(input: &DeriveInput) -> TokenStream2 {
    let type_ident = type_ident(input);
    let body = match &input.data {
        Data::Struct(struct_data) => make_fields(&struct_data.fields, quote! { init. }),
        Data::Enum(enum_data) => {
            let enum_ty = attrs::get_enum_type(input);
            let contents =
                enum_data
                    .variants
                    .iter()
                    .enumerate()
                    .fold(quote! {}, |accum, (i, variant)| {
                        let var_ident = &variant.ident;
                        let index = Index::from(i);
                        let bs = match_::make_bindings(&variant.fields);
                        let (bindings, wrapper) = (bs.bindings, bs.wrapper);
                        let items = make_fields(&variant.fields, bs.prefix);
                        quote! {
                            #accum
                            #type_ident::#var_ident #bindings => {
                                #wrapper
                                <#enum_ty as ::flatty::FlatInit>::init_unchecked(mem, #index);
                                offset += Self::DATA_OFFSET;
                                // FIXME: Check variant size
                                #items
                            }
                        }
                    });
            quote! {
                match init {
                    #contents
                }
            }
        }
        Data::Union(_union_data) => unimplemented!(),
    };
    quote! {
        let mut offset: usize = 0;
        #body
        Self::interpret_mut_unchecked(mem)
    }
}
