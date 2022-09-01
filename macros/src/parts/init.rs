use crate::{
    parts::{attrs, dyn_, match_},
    utils::fields_iter::FieldsIter,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{self, Data, DeriveInput, Index};

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
            <#ty>::placement_new_unchecked(&mut mem[offset..], #prefix #ident);
            #add_size
        }
    })
}

pub fn make(input: &DeriveInput) -> TokenStream2 {
    let type_ident = dyn_::ident(input);
    let body = match &input.data {
        Data::Struct(struct_data) => make_fields(&struct_data.fields, quote! { &init. }),
        Data::Enum(enum_data) => {
            let enum_ty = attrs::repr::get_enum_type(input);
            let contents =
                enum_data
                    .variants
                    .iter()
                    .enumerate()
                    .fold(quote! {}, |accum, (i, variant)| {
                        let var_ident = &variant.ident;
                        let index = Index::from(i);
                        let bs = match_::make_bindings(&variant.fields);
                        let (bindings, wrapper, prefix) = (bs.bindings, bs.wrapper, bs.prefix);
                        let items = make_fields(&variant.fields, quote!{ &#prefix });
                        quote! {
                            #accum
                            #type_ident::#var_ident #bindings => {
                                #wrapper
                                let state = <#enum_ty as ::flatty::FlatInit>::placement_new_unchecked(mem, &#index);
                                offset += Self::DATA_OFFSET;
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
        Self::reinterpret_mut_unchecked(mem)
    }
}
