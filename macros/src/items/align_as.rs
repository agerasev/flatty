use crate::{utils::FieldIter, Context};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn struct_(ctx: &Context, input: &DeriveInput) -> TokenStream {
    pub fn collect_fields<I: FieldIter>(fields: &I) -> TokenStream {
        fields.iter().fold(quote! {}, |a, f| {
            let ty = &f.ty;
            quote! { #a <#ty as ::flatty::traits::FlatUnsized>::AlignAs, }
        })
    }

    let type_list = match &input.data {
        Data::Struct(data) => collect_fields(&data.fields),
        Data::Enum(data) => {
            let tag_type = ctx.info.tag_type.as_ref().unwrap();
            data.variants.iter().fold(quote! { #tag_type, }, |accum, variant| {
                let var_type_list = collect_fields(&variant.fields);
                quote! { #accum #var_type_list }
            })
        }
        Data::Union(..) => unimplemented!(),
    };

    let vis = &input.vis;
    let align_as_type = ctx.idents.align_as.as_ref().unwrap();

    let generic_params = &input.generics.params;
    let where_clause = &input.generics.where_clause;

    quote! {
        #[allow(dead_code)]
        #[repr(C)]
        #vis struct #align_as_type<#generic_params>(
            #type_list
        ) #where_clause;
    }
}
