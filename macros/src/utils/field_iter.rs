use std::iter::{self, ExactSizeIterator};
use syn::{self, Field, Fields, FieldsNamed, FieldsUnnamed};

pub type FieldIterator<'a> = Box<dyn ExactSizeIterator<Item = &'a Field> + 'a>;

pub trait FieldIter {
    fn field_iter(&self) -> FieldIterator<'_>;
}

impl FieldIter for Fields {
    fn field_iter(&self) -> FieldIterator<'_> {
        match self {
            Fields::Named(named_fields) => named_fields.field_iter(),
            Fields::Unnamed(unnamed_fields) => unnamed_fields.field_iter(),
            Fields::Unit => Box::new(iter::empty()),
        }
    }
}

impl FieldIter for FieldsNamed {
    fn field_iter(&self) -> FieldIterator<'_> {
        Box::new(self.named.iter())
    }
}

impl FieldIter for FieldsUnnamed {
    fn field_iter(&self) -> FieldIterator<'_> {
        Box::new(self.unnamed.iter())
    }
}
