use std::iter::{self, ExactSizeIterator};
use syn::{self, Field, Fields, FieldsNamed, FieldsUnnamed};

pub type FieldsIterator<'a> = Box<dyn ExactSizeIterator<Item = &'a Field> + 'a>;

pub trait FieldsIter {
    fn fields_iter(&self) -> FieldsIterator<'_>;
}

impl FieldsIter for Fields {
    fn fields_iter(&self) -> FieldsIterator<'_> {
        match self {
            Fields::Named(named_fields) => named_fields.fields_iter(),
            Fields::Unnamed(unnamed_fields) => unnamed_fields.fields_iter(),
            Fields::Unit => Box::new(iter::empty()),
        }
    }
}
impl FieldsIter for FieldsNamed {
    fn fields_iter(&self) -> FieldsIterator<'_> {
        Box::new(self.named.iter())
    }
}
impl FieldsIter for FieldsUnnamed {
    fn fields_iter(&self) -> FieldsIterator<'_> {
        Box::new(self.unnamed.iter())
    }
}
