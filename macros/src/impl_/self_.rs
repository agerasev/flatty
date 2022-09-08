use super::{align_as, tag};
use crate::{utils::generic, utils::type_list, Context};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn impl_(ctx: &Context, input: &DeriveInput) -> TokenStream {
    let self_ident = &input.ident;

    let generic_params = &input.generics.params;
    let generic_args = generic::args(&input.generics);
    let where_clause = &input.generics.where_clause;

    let mut items = quote! {};
    let mut extras = quote! {};

    match &input.data {
        Data::Enum(..) => {
            let enum_type = ctx.info.enum_type.as_ref().unwrap();
            items = quote! {
                #items
                const DATA_OFFSET: usize = ::flatty::utils::ceil_mul(<#enum_type as ::flatty::FlatSized>::SIZE, <Self as ::flatty::FlatBase>::ALIGN);
            };

            let tag_impl = tag::impl_(ctx, input, ctx.info.sized);
            extras = quote! {
                #extras
                #tag_impl
            }
        }
        Data::Struct(data) => {
            if !data.fields.is_empty() && !ctx.info.sized {
                let value = if data.fields.len() > 1 {
                    let len = data.fields.len();
                    let type_list = type_list(data.fields.iter().take(len - 1));
                    let last_ty = &data.fields.iter().last().unwrap().ty;
                    quote! {
                        ::flatty::utils::ceil_mul(
                            ::flatty::iter::fold_size!(0; #type_list),
                            <#last_ty as ::flatty::FlatBase>::ALIGN,
                        )
                    }
                } else {
                    quote! { 0 }
                };
                items = quote! {
                    #items
                    const LAST_FIELD_OFFSET: usize = #value;
                }
            }
        }
        Data::Union(..) => unimplemented!(),
    }
    if !ctx.info.sized {
        let align_as_impl = align_as::impl_(ctx, input);
        extras = quote! {
            #extras
            #align_as_impl
        }
    }

    quote! {
        impl<#generic_params> #self_ident<#generic_args>
        #where_clause
        {
            #items
        }

        #extras
    }
}
