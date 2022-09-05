use crate::{
    utils::{generic, FieldIter},
    Context,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Index};

fn validate_method(ctx: &Context, input: &DeriveInput) -> TokenStream {
    fn collect_fields<I: FieldIter>(fields: &I) -> TokenStream {
        let iter = fields.field_iter();
        let len = iter.len();
        iter.enumerate().fold(quote! {}, |accum, (i, field)| {
            let ty = &field.ty;

            let next_pos = if i + 1 < len {
                quote! { pos += <#ty as FlatSized>::SIZE; }
            } else {
                quote! {}
            };

            quote! {
                #accum

                pos = ceil_mul(pos, <#ty as FlatBase>::ALIGN);

                <#ty as FlatCast>::validate(unsafe {
                    Muu::<#ty>::from_bytes_unchecked(bytes.get_unchecked(pos..))
                }).map_err(|e| e.offset(pos))?;

                #next_pos
            }
        })
    }

    let body = match &input.data {
        Data::Struct(struct_data) => collect_fields(&struct_data.fields),
        Data::Enum(enum_data) => {
            let enum_ty = ctx.info.enum_type.as_ref().unwrap();
            let varaints =
                enum_data
                    .variants
                    .iter()
                    .enumerate()
                    .fold(quote! {}, |accum, (i, variant)| {
                        let index = Index::from(i);
                        let items = collect_fields(&variant.fields);
                        quote! {
                            #accum
                            #index => { #items }
                        }
                    });

            quote! {
                use ::flatty::{Error, ErrorKind};

                let tag = unsafe { Muu::<#enum_ty>::from_bytes_unchecked(bytes) };
                <#enum_ty as FlatCast>::validate(tag)?;
                pos += ceil_mul(pos + <#enum_ty as FlatSized>::SIZE, <Self as FlatBase>::ALIGN);

                match unsafe { *tag.as_ptr() } {
                    #varaints

                    _ => return Err(Error {
                        kind: ErrorKind::InvalidEnumState,
                        pos: 0,
                    }),
                };
            }
        }
        Data::Union(_union_data) => unimplemented!(),
    };
    quote! {
        fn validate(this: &::flatty::mem::Muu<Self>) -> Result<(), ::flatty::Error> {
            use ::flatty::{mem::Muu, prelude::*, utils::ceil_mul};

            let mut pos: usize = 0;
            let bytes = this.as_bytes();

            #body

            Ok(())
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
