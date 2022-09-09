use crate::{
    utils::{generic, type_list},
    Context,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput, Field, Fields, Ident};

pub fn struct_(ctx: &Context, input: &DeriveInput) -> TokenStream {
    let vis = &input.vis;
    let self_ident = &input.ident;

    let generic_params = &input.generics.params;
    let generic_args = generic::args(&input.generics);
    let where_clause = &input.generics.where_clause;

    let tag_type = ctx.idents.tag.as_ref().unwrap();
    let align_as_ident = ctx.idents.align_as.as_ref().unwrap();
    let align_as_type = quote! { #align_as_ident<#generic_args> };

    quote! {
        #[repr(C)]
        #vis struct #self_ident<#generic_params>
        #where_clause
        {
            tag: #tag_type,
            _align: [#align_as_type; 0],
            data: [u8],
        }
    }
}

fn gen_ref_struct(_ctx: &Context, input: &DeriveInput, mutable: bool, ref_ident: &Ident) -> TokenStream {
    let mut_ = if mutable { Some(quote! { mut }) } else { None };
    let map_field = |field: &Field| {
        let ty = &field.ty;
        let prefix = match &field.ident {
            Some(ident) => quote! { #ident: },
            None => quote! {},
        };
        quote! { #prefix &'__flatty_a #mut_ #ty }
    };

    let vis = &input.vis;

    let generic_params = &input.generics.params;
    let where_clause = &input.generics.where_clause;

    let variants = if let Data::Enum(data) = &input.data {
        data.variants.iter().fold(quote! {}, |accum, var| {
            let var_ident = &var.ident;
            let contents = var.fields.iter().fold(quote! {}, |a, f| {
                let item = map_field(f);
                quote! { #a #item, }
            });
            let fields = match &var.fields {
                Fields::Unit => quote! {},
                Fields::Named(..) => quote! { { #contents } },
                Fields::Unnamed(..) => quote! { (#contents) },
            };
            quote! {
                #accum
                #var_ident #fields,
            }
        })
    } else {
        unreachable!();
    };

    quote! {
        #[allow(dead_code)]
        #vis enum #ref_ident<'__flatty_a, #generic_params>
        #where_clause
        {
            #variants
        }
    }
}

fn gen_ref_impl(
    ctx: &Context,
    input: &DeriveInput,
    mutable: bool,
    ref_ident: &Ident,
    ref_method_name: TokenStream,
    assume_init: TokenStream,
    ref_iter_type: TokenStream,
) -> TokenStream {
    let mut_ = if mutable { Some(quote! { mut }) } else { None };
    let self_ident = &input.ident;
    let tag_type = ctx.idents.tag.as_ref().unwrap();

    let generic_params = &input.generics.params;
    let generic_args = generic::args(&input.generics);
    let where_clause = &input.generics.where_clause;

    let match_body = if let Data::Enum(data) = &input.data {
        let bind = |i: usize, f: &Field| -> Ident {
            match &f.ident {
                Some(ident) => ident.clone(),
                None => Ident::new(&format!("b{}", i), f.span()),
            }
        };
        data.variants.iter().fold(quote! {}, |accum, var| {
            let var_ident = &var.ident;
            let bindings = if !var.fields.is_empty() {
                let preface = {
                    let type_list = type_list(var.fields.iter());
                    quote! { let iter = unsafe { #ref_iter_type::new_unchecked(& #mut_ self.data, type_list!(#type_list)) }; }
                };
                let bindings = {
                    let iter = var.fields.iter();
                    let len = iter.len();
                    iter.enumerate().fold(quote! {}, |a, (i, f)| {
                        let value = if i + 1 < len {
                            quote! { let (iter, value) = iter.next(); }
                        } else {
                            quote! { let value = iter.finalize(); }
                        };
                        let binding = bind(i, f);
                        quote! {
                            #a
                            #value
                            let #binding = unsafe { value.#assume_init() };
                        }
                    })
                };
                quote! {
                    #preface
                    #bindings
                }
            } else {
                quote! {}
            };
            let result = {
                let pattern = {
                    let contents = var.fields.iter().enumerate().fold(quote! {}, |a, (i, f)| {
                        let binding = bind(i, f);
                        quote! { #a #binding, }
                    });
                    match &var.fields {
                        Fields::Unit => quote! {},
                        Fields::Named(..) => quote! { { #contents } },
                        Fields::Unnamed(..) => quote! { (#contents) },
                    }
                };
                quote! { #ref_ident::#var_ident #pattern }
            };
            quote! {
                #accum
                #tag_type::#var_ident => {
                    #bindings
                    #result
                }
            }
        })
    } else {
        unreachable!();
    };

    quote! {
        impl<#generic_params> #self_ident<#generic_args>
        #where_clause
        {
            pub fn #ref_method_name(& #mut_ self) -> #ref_ident<'_, #generic_args> {
                use ::flatty::{prelude::*, iter::{prelude::*, type_list, #ref_iter_type}};
                match self.tag {
                    #match_body
                }
            }
        }
    }
}

pub fn ref_struct(ctx: &Context, input: &DeriveInput) -> TokenStream {
    gen_ref_struct(ctx, input, false, ctx.idents.ref_.as_ref().unwrap())
}

pub fn ref_impl(ctx: &Context, input: &DeriveInput) -> TokenStream {
    gen_ref_impl(
        ctx,
        input,
        false,
        ctx.idents.ref_.as_ref().unwrap(),
        quote! { as_ref },
        quote! { assume_init_ref },
        quote! { RefIter },
    )
}

pub fn mut_struct(ctx: &Context, input: &DeriveInput) -> TokenStream {
    gen_ref_struct(ctx, input, true, ctx.idents.mut_.as_ref().unwrap())
}

pub fn mut_impl(ctx: &Context, input: &DeriveInput) -> TokenStream {
    gen_ref_impl(
        ctx,
        input,
        true,
        ctx.idents.mut_.as_ref().unwrap(),
        quote! { as_mut },
        quote! { assume_init_mut },
        quote! { MutIter },
    )
}
