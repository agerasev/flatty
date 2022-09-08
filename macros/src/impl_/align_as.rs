use crate::{utils::type_list, Context};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn impl_(ctx: &Context, input: &DeriveInput) -> TokenStream {
    let type_list = match &input.data {
        Data::Struct(data) => type_list(data.fields.iter()),
        Data::Enum(data) => {
            let enum_type = ctx.info.enum_type.as_ref().unwrap();
            data.variants
                .iter()
                .fold(quote! { #enum_type }, |accum, variant| {
                    let var_type_list = type_list(variant.fields.iter());
                    quote! { #accum #var_type_list }
                })
        }
        Data::Union(..) => unimplemented!(),
    };

    let vis = &input.vis;
    let align_as_type = ctx.idents.align_as.as_ref().unwrap();

    quote! {
        #[allow(dead_code)]
        #[repr(C)]
        #vis struct #align_as_type(
            #type_list
        );
    }
}
