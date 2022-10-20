use crate::{
    utils::{generic, type_list, FieldIter},
    Context,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

fn init_default_method(ctx: &Context, input: &DeriveInput) -> TokenStream {
    fn collect_fields<I: FieldIter>(fields: &I, bytes: TokenStream) -> TokenStream {
        let iter = fields.iter();
        if iter.len() == 0 {
            return quote! { Ok(()) };
        }
        let type_list = type_list(iter);
        quote! {
            unsafe { iter::MutIter::new_unchecked(#bytes, iter::type_list!(#type_list)) }.init_default_all()?;
            Ok(unsafe { this.assume_init_mut() })
        }
    }

    let body = match &input.data {
        Data::Struct(struct_data) => collect_fields(&struct_data.fields, quote! { this.as_mut_bytes() }),
        Data::Enum(_enum_data) => {
            let tag_type = ctx.idents.tag.as_ref().unwrap();
            quote! {
                Self::set_tag(this, <#tag_type>::default())
            }
        }
        Data::Union(_union_data) => unimplemented!(),
    };
    quote! {
        fn init_default(this: &mut ::flatty::mem::MaybeUninitUnsized<Self>) -> Result<&mut Self, ::flatty::Error> {
            use ::flatty::{prelude::*, mem::MaybeUninitUnsized, utils::iter::{prelude::*, self}};
            #body
        }
    }
}

fn enum_set_tag_method(ctx: &Context, _input: &DeriveInput) -> TokenStream {
    let tag_type = ctx.idents.tag.as_ref().unwrap();
    quote! {
        pub fn set_tag(this: &mut ::flatty::mem::MaybeUninitUnsized<Self>, tag: #tag_type) -> Result<&mut Self, ::flatty::Error> {
            use ::flatty::mem::MaybeUninitUnsized;
            let bytes = this.as_mut_bytes();
            unsafe { MaybeUninitUnsized::<#tag_type>::from_mut_bytes_unchecked(bytes) }
                .as_mut_sized().write(tag);
            unsafe { Self::init_default_data_by_tag(tag, bytes.get_unchecked_mut(Self::DATA_OFFSET..)) }
                .map_err(|e| e.offset(Self::DATA_OFFSET))?;
            Ok(unsafe { this.assume_init_mut() })
        }
    }
}

fn enum_reset_tag_method(ctx: &Context, _input: &DeriveInput) -> TokenStream {
    let tag_type = ctx.idents.tag.as_ref().unwrap();
    quote! {
        pub fn reset_tag(&mut self, tag: #tag_type) -> Result<(), ::flatty::Error> {
            self.tag = tag;
            unsafe { Self::init_default_data_by_tag(tag, &mut self.data) }.map_err(|e| e.offset(Self::DATA_OFFSET))
        }
    }
}

fn enum_init_default_data_by_tag_method(ctx: &Context, input: &DeriveInput) -> TokenStream {
    let enum_type = ctx.info.enum_type.as_ref().unwrap();
    let tag_type = ctx.idents.tag.as_ref().unwrap();
    let match_body = if let Data::Enum(data) = &input.data {
        data.variants.iter().fold(quote! {}, |accum, var| {
            let var_name = &var.ident;
            let type_list = type_list(var.fields.iter());
            let body = if !type_list.is_empty() {
                quote! { iter::MutIter::new_unchecked(bytes, iter::type_list!(#type_list)).init_default_all() }
            } else {
                quote! { Ok(()) }
            };
            quote! {
                #accum
                #tag_type::#var_name => { #body }
            }
        })
    } else {
        unreachable!();
    };
    quote! {
        unsafe fn init_default_data_by_tag(tag: #tag_type, bytes: &mut [u8]) -> Result<(), ::flatty::Error> {
            use ::flatty::{utils::iter::{prelude::*, self}, Error, ErrorKind};
            if bytes.len() < Self::DATA_MIN_SIZES[tag as #enum_type as usize] {
                return Err(Error { kind: ErrorKind::InsufficientSize, pos: 0 });
            }
            match tag {
                #match_body
            }
        }
    }
}

pub fn impl_(ctx: &Context, input: &DeriveInput) -> TokenStream {
    let self_ident = &input.ident;

    let generic_params = &input.generics.params;
    let generic_args = generic::args(&input.generics);
    let where_clause = generic::where_clause(
        input,
        quote! { ::flatty::FlatDefault + Sized },
        if ctx.info.sized {
            None
        } else {
            Some(quote! { ::flatty::FlatDefault })
        },
    );

    let init_default_method = init_default_method(ctx, input);

    let main = quote! {
        unsafe impl<#generic_params> ::flatty::FlatDefault for #self_ident<#generic_args>
        #where_clause
        {
            #init_default_method
        }
    };

    let extras = if let Data::Enum(..) = &input.data {
        let set_tag_method = enum_set_tag_method(ctx, input);
        let reset_tag_method = enum_reset_tag_method(ctx, input);
        let init_default_data_by_tag_method = enum_init_default_data_by_tag_method(ctx, input);

        quote! {
            impl<#generic_params> #self_ident<#generic_args>
            #where_clause
            {
                #set_tag_method
                #reset_tag_method
                #init_default_data_by_tag_method
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #main
        #extras
    }
}
