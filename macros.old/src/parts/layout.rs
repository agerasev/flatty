use crate::{
    parts::{attrs, match_},
    utils::fields_iter::FieldsIter,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::iter::Iterator;
use syn::{self, Data, DeriveInput, Ident, Index, Type};

pub fn make_ptr_metadata(input: &DeriveInput) -> TokenStream2 {
    match &input.data {
        Data::Struct(struct_data) => match struct_data.fields.fields_iter().last() {
            Some(field) => {
                let field_ty = &field.ty;
                quote! {
                    <#field_ty as ::flatty::FlatUnsized>::ptr_metadata(
                        &mem[(<Self as ::flatty::FlatBase>::MIN_SIZE - <#field_ty as ::flatty::FlatBase>::MIN_SIZE)..],
                    )
                }
            }
            None => quote! { 0 },
        },
        Data::Enum(_enum_data) => unimplemented!(),
        Data::Union(_union_data) => unimplemented!(),
    }
}
