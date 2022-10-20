use crate::{
    utils::{generic, type_list, FieldIter, FieldIterator},
    Context,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, Data, DeriveInput, Field, Fields, GenericParam, Ident, Index, Type, Variant};

struct LastParam<'a> {
    ident: Ident,
    ty: &'a Type,
    emplacer_ty: TokenStream,
}

fn last_ident(postfix: &str) -> Ident {
    Ident::new(&format!("__flatty_Last{}", postfix), Span::call_site())
}

fn last_param<'a>(fields: &'a Fields, postfix: &'_ str) -> Option<LastParam<'a>> {
    fields.iter().last().map(|f| {
        let ty = &f.ty;
        LastParam {
            ident: last_ident(postfix),
            ty,
            emplacer_ty: quote! { ::flatty::Emplacer<#ty> },
        }
    })
}

fn var_last_params(variants: &Punctuated<Variant, Comma>) -> Vec<Option<LastParam<'_>>> {
    variants
        .iter()
        .map(|var| last_param(&var.fields, &format!("_{}", &var.ident)))
        .collect()
}

fn extend_args<'a, 'b: 'a, I: Iterator<Item = &'a LastParam<'b>>>(orig: &TokenStream, add: I) -> TokenStream {
    let mut args = quote! { #orig };
    for param in add {
        let ident = &param.ident;
        args = quote! { #args #ident, }
    }
    args
}

fn extend_params<'a, 'b: 'a, I: Iterator<Item = &'a LastParam<'b>>>(
    orig: &Punctuated<GenericParam, Comma>,
    add: I,
) -> TokenStream {
    let mut params = quote! { #orig };
    if !orig.empty_or_trailing() {
        params = quote! { #params, }
    }
    for param in add {
        let (ident, emplacer_ty) = (&param.ident, &param.emplacer_ty);
        params = quote! { #params #ident: #emplacer_ty, }
    }
    params
}

pub fn struct_(ctx: &Context, input: &DeriveInput) -> TokenStream {
    fn collect_fields(fields: &Fields, last_param: Option<&Ident>) -> (TokenStream, TokenStream) {
        let (iter, last) = {
            let len = fields.iter().len();
            (fields.iter().take(if len > 0 { len - 1 } else { 0 }), fields.iter().last())
        };
        match fields {
            Fields::Unit => (quote! {}, quote! {;}),
            Fields::Unnamed(..) => {
                let mut items = iter.fold(quote! {}, |accum, f| {
                    let ty = &f.ty;
                    quote! { #accum #ty, }
                });
                if let Some(..) = last {
                    let param = last_param.unwrap();
                    items = quote! { #items #param, }
                }
                (quote! { ( #items ) }, quote! {;})
            }
            Fields::Named(..) => {
                let mut items = iter.fold(quote! {}, |accum, f| {
                    let ty = &f.ty;
                    let ident = f.ident.as_ref().unwrap();
                    quote! {
                        #accum
                        #ident: #ty,
                    }
                });
                if let Some(last) = last {
                    let param = last_param.unwrap();
                    let ident = last.ident.as_ref().unwrap();
                    items = quote! { #items #ident: #param, }
                }
                (
                    quote! {
                        {
                            #items
                        }
                    },
                    quote! {},
                )
            }
        }
    }

    let init_ident = ctx.idents.init.as_ref().unwrap();
    let vis = &input.vis;

    let generic_params = &input.generics.params;
    let where_clause = &input.generics.where_clause;

    match &input.data {
        Data::Struct(data) => {
            let last_param = last_param(&data.fields, "");
            let (body, semi) = collect_fields(&data.fields, last_param.as_ref().map(|p| &p.ident));
            let all_params = extend_params(generic_params, [last_param].iter().flatten());
            quote! {
                #vis struct #init_ident<#all_params> #where_clause #body #semi
            }
        }
        Data::Enum(data) => {
            let mut items = quote! {};
            let last_params = var_last_params(&data.variants);
            for (var, param) in data.variants.iter().zip(last_params.iter()) {
                let var_ident = &var.ident;
                let (var_body, _) = collect_fields(&var.fields, param.as_ref().map(|p| &p.ident));
                items = quote! {
                    #items
                    #var_ident #var_body,
                };
            }
            let all_params = extend_params(generic_params, last_params.iter().flatten());
            quote! { #vis enum #init_ident<#all_params> #where_clause { #items } }
        }
        _ => unimplemented!(),
    }
}

pub fn impl_(ctx: &Context, input: &DeriveInput) -> TokenStream {
    fn field_postfix(i: usize, f: &Field) -> TokenStream {
        f.ident.as_ref().map(|x| quote! { #x }).unwrap_or_else(|| {
            let index = Index::from(i);
            quote! { #index }
        })
    }

    fn collect_fields<'a, F: Fn(usize, &'a Field) -> TokenStream>(fields: &'a Fields, get_item: F) -> TokenStream {
        let iter = fields.iter();
        let len = iter.len();
        let mut items = if len > 0 {
            let type_list = type_list(fields.iter());
            quote! {
                let iter = unsafe { iter::MutIter::new_unchecked(uninit.as_mut_bytes(), iter::type_list!(#type_list)) };
            }
        } else {
            quote! {}
        };
        for (i, f) in fields.iter().enumerate() {
            let item = get_item(i, f);

            let uninit_ident = Ident::new(&format!("__u_{}", i), Span::call_site());
            let step = if i + 1 < len {
                quote! { let (iter, #uninit_ident) = iter.next(); }
            } else {
                quote! { let #uninit_ident = iter.finalize(); }
            };

            items = quote! {
                #items
                #step
                #item.emplace(#uninit_ident)?;
            }
        }
        items
    }

    let self_ident = &input.ident;
    let init_ident = ctx.idents.init.as_ref().unwrap();

    let generic_params = &input.generics.params;
    let generic_args = generic::args(&input.generics);
    let where_clause = &input.generics.where_clause;

    let all_params;
    let all_args;

    let body = match &input.data {
        Data::Struct(data) => {
            let last_param = last_param(&data.fields, "");
            all_params = extend_params(generic_params, [last_param.as_ref()].into_iter().flatten());
            all_args = extend_args(&generic_args, [last_param.as_ref()].into_iter().flatten());

            collect_fields(&data.fields, |i, f| {
                let item = field_postfix(i, f);
                quote! { self.#item }
            })
        }
        Data::Enum(data) => {
            let last_params = var_last_params(&data.variants);
            all_params = extend_params(generic_params, last_params.iter().flatten());
            all_args = extend_args(&generic_args, last_params.iter().flatten());

            let items = data.variants.iter().fold(quote! {}, |accum, var| {
                let ident = &var.ident;
                let get_item = |i, f| {
                    let item = field_postfix(i, f);
                    if f.ident.is_some() {
                        item
                    } else {
                        let ident = Ident::new(&format!("b{}", item), Span::call_site());
                        quote! { #ident }
                    }
                };
                let items = collect_fields(&var.fields, get_item);
                let pat_body = var
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| get_item(i, f))
                    .fold(quote! {}, |accum, ident| quote! { #accum #ident, });
                let pat = match var.fields {
                    Fields::Unit => quote! {},
                    Fields::Unnamed(..) => quote! { (#pat_body) },
                    Fields::Named(..) => quote! { { #pat_body } },
                };
                quote! {
                    #accum
                    #init_ident::#ident #pat => {
                        #items
                    }
                }
            });

            quote! {
                match self {
                    #items
                }
            }
        }
        _ => unimplemented!(),
    };

    quote! {
        impl<#all_params> ::flatty::Emplacer<#self_ident<#generic_args>> for #init_ident<#all_args>
        #where_clause
        {
            fn emplace<'__flatty_a>(
                self,
                uninit: &'__flatty_a mut ::flatty::mem::MaybeUninitUnsized<#self_ident<#generic_args>>,
            ) -> Result<&'__flatty_a mut #self_ident<#generic_args>, ::flatty::Error> {
                use ::flatty::{prelude::*, utils::iter::{prelude::*, self}};
                #body
                Ok(unsafe { uninit.assume_init_mut() })
            }
        }
    }
}

pub fn impl_default(ctx: &Context, input: &DeriveInput) -> TokenStream {
    quote! {}
}
