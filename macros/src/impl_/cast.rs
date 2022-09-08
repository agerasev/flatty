use crate::{
    utils::{generic, type_list, FieldIter},
    Context,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

fn validate_method(ctx: &Context, input: &DeriveInput) -> TokenStream {
    fn collect_fields<I: FieldIter>(fields: &I, bytes: TokenStream) -> TokenStream {
        let iter = fields.iter();
        if iter.len() == 0 {
            return quote! {
                Ok(())
            };
        }
        let type_list = type_list(iter);
        quote! {
            unsafe { RefIter::new_unchecked(#bytes, type_list!(#type_list)) }
                .validate_all()
        }
    }

    let body = match &input.data {
        Data::Struct(struct_data) => {
            collect_fields(&struct_data.fields, quote! { this.as_bytes() })
        }
        Data::Enum(enum_data) => {
            let tag_type = ctx.idents.tag.as_ref().unwrap();
            let validate_tag = quote! {
                let tag = unsafe { MaybeUninitUnsized::<#tag_type>::from_bytes_unchecked(this.as_bytes()) };
                <#tag_type as ::flatty::FlatCast>::validate(tag)?;
                *unsafe{ tag.assume_init_ref() }
            };
            let varaints = enum_data.variants.iter().fold(quote! {}, |accum, variant| {
                let items = collect_fields(&variant.fields, quote! { data });
                let var_name = &variant.ident;
                quote! {
                    #accum
                    #tag_type::#var_name => { #items }
                }
            });

            quote! {
                use ::flatty::{Error, ErrorKind};

                let tag = { #validate_tag };
                let data = unsafe { this.as_bytes().get_unchecked(Self::DATA_OFFSET..) };

                match tag {
                    #varaints
                }.map_err(|e| e.offset(Self::DATA_OFFSET))
            }
        }
        Data::Union(_union_data) => unimplemented!(),
    };
    quote! {
        fn validate(this: &::flatty::mem::MaybeUninitUnsized<Self>) -> Result<(), ::flatty::Error> {
            use ::flatty::{prelude::*, mem::MaybeUninitUnsized, iter::{prelude::*, RefIter, type_list}};
            #body
        }
    }
}

pub fn impl_(ctx: &Context, input: &DeriveInput) -> TokenStream {
    let self_ident = &input.ident;

    let generic_params = &input.generics.params;
    let generic_args = generic::args(&input.generics);
    let where_clause = generic::where_clause(
        input,
        quote! { ::flatty::FlatCast + Sized },
        if ctx.info.sized {
            None
        } else {
            Some(quote! { ::flatty::FlatCast })
        },
    );

    let validate_method = validate_method(ctx, input);

    quote! {
        impl<#generic_params> ::flatty::FlatCast for #self_ident<#generic_args>
        where
            #where_clause
        {
            #validate_method
        }
    }
}
