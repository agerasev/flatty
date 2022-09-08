use crate::{utils::FieldIter, Context};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn impl_(ctx: &Context, input: &DeriveInput) -> TokenStream {
    pub fn collect_fields<I: FieldIter>(fields: &I) -> TokenStream {
        fields.iter().fold(quote! {}, |a, f| {
            let ty = &f.ty;
            quote! { #a <#ty as ::flatty::FlatMaybeUnsized>::AlignAs, }
        })
    }

    let type_list = match &input.data {
        Data::Struct(data) => collect_fields(&data.fields),
        Data::Enum(data) => {
            let enum_type = ctx.info.enum_type.as_ref().unwrap();
            data.variants
                .iter()
                .fold(quote! { #enum_type }, |accum, variant| {
                    let var_type_list = collect_fields(&variant.fields);
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
