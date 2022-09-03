use crate::{
    parts::{attrs, dyn_, match_},
    utils::fields_iter::FieldsIter,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{self, Data, DeriveInput, Index};
