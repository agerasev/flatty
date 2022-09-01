use super::derive;
use syn::{Data, DeriveInput};

pub fn has(input: &DeriveInput) -> bool {
    derive::has(input, "Default")
}

pub fn filter_out(input: &DeriveInput) -> DeriveInput {
    let mut output = input.clone();
    derive::remove(&mut output.attrs, "Default");
    match &mut output.data {
        Data::Enum(enum_data) => {
            for variant in &mut enum_data.variants {
                variant.attrs.retain(|attr| !attr.path.is_ident("default"))
            }
        }
        _ => (),
    }
    output
}
