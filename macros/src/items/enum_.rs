use syn::{Data, DeriveInput, Fields};

pub fn is_c_like(input: &DeriveInput) -> bool {
    if let Data::Enum(data) = &input.data {
        data.variants.iter().all(|var| matches!(var.fields, Fields::Unit))
    } else {
        panic!("Enum is expected")
    }
}
