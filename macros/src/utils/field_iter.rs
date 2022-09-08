use proc_macro2::TokenStream;
use quote::quote;
use std::iter::{self, ExactSizeIterator};
use syn::{self, Field, Fields, FieldsNamed, FieldsUnnamed};

pub type FieldIterator<'a> = Box<dyn ExactSizeIterator<Item = &'a Field> + 'a>;

pub trait FieldIter {
    fn iter(&self) -> FieldIterator<'_>;
}

impl FieldIter for Fields {
    fn iter(&self) -> FieldIterator<'_> {
        match self {
            Fields::Named(named_fields) => named_fields.iter(),
            Fields::Unnamed(unnamed_fields) => unnamed_fields.iter(),
            Fields::Unit => Box::new(iter::empty()),
        }
    }
}

impl FieldIter for FieldsNamed {
    fn iter(&self) -> FieldIterator<'_> {
        Box::new(self.named.iter())
    }
}

impl FieldIter for FieldsUnnamed {
    fn iter(&self) -> FieldIterator<'_> {
        Box::new(self.unnamed.iter())
    }
}

pub fn type_list<'a, I: Iterator<Item = &'a Field>>(field_iter: I) -> TokenStream {
    field_iter.fold(quote! {}, |a, f| {
        let ty = &f.ty;
        quote! { #a #ty, }
    })
}
