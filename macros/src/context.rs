use crate::Info;
use proc_macro2::Ident;

#[derive(Clone, Default, Debug)]
pub struct AssocIdents {
    pub align_as: Option<Ident>,
    pub tag: Option<Ident>,
    pub ref_: Option<Ident>,
    pub mut_: Option<Ident>,
    pub init: Option<Ident>,
}

#[derive(Clone, Debug)]
pub struct Context {
    pub info: Info,
    pub idents: AssocIdents,
}
