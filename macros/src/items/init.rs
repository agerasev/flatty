use crate::{
    utils::{generic, type_list, FieldIter},
    Context,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, Data, DeriveInput, Field, Fields, GenericParam, Ident, Index};

struct Param<'a> {
    ident: Ident,
    field: &'a Field,
    bound: TokenStream,
}

impl<'a> Param<'a> {
    fn new(ident: Ident, field: &'a Field) -> Self {
        let ty = &field.ty;
        Param {
            ident,
            field,
            bound: quote! { ::flatty::Emplacer<#ty> },
        }
    }
}

fn global_prefix(span: Span) -> Ident {
    Ident::new("__flatty", span)
}

fn param<'a: 'b, 'b>(i: usize, f: &'a Field, prefix: &'b Ident) -> Param<'a> {
    let name = match &f.ident {
        Some(ident) => format!("{}__{}", prefix, ident.to_string().to_uppercase()),
        None => format!("{}__{}", prefix, i),
    };
    Param::new(Ident::new(&name, prefix.span()), f)
}

fn params<'a: 'b, 'b>(fields: &'a Fields, prefix: &'b Ident) -> Vec<Param<'a>> {
    fields.iter().enumerate().map(|(i, f)| param(i, f, prefix)).collect()
}

fn data_params<'a: 'b, 'b>(data: &'a Data, prefix: &'b Ident) -> Vec<Vec<Param<'a>>> {
    match data {
        Data::Struct(struct_) => vec![params(&struct_.fields, prefix)],
        Data::Enum(enum_) => enum_
            .variants
            .iter()
            .map(|var| {
                let prefix = Ident::new(&format!("{}__{}", prefix, var.ident), var.ident.span());
                params(&var.fields, &prefix)
            })
            .collect(),
        Data::Union(..) => unimplemented!(),
    }
}

fn make_args<'a, 'b: 'a, I: Iterator<Item = &'a Param<'b>>>(params: I) -> TokenStream {
    let mut args = quote! {};
    for param in params {
        let ident = &param.ident;
        args = quote! { #args #ident, }
    }
    args
}

fn extend_params<'a, 'b: 'a, I: Iterator<Item = &'a Param<'b>>>(
    orig: &Punctuated<GenericParam, Comma>,
    add: I,
) -> TokenStream {
    let mut params = quote! { #orig };
    if !orig.empty_or_trailing() {
        params = quote! { #params, }
    }
    for param in add {
        let (ident, bound) = (&param.ident, &param.bound);
        params = quote! { #params #ident: #bound, }
    }
    params
}

pub fn struct_(ctx: &Context, input: &DeriveInput) -> TokenStream {
    fn collect_fields(fields: &Fields, prefix: &Ident) -> (TokenStream, TokenStream) {
        let params = params(fields, prefix);
        match fields {
            Fields::Unit => (quote! {}, quote! {;}),
            Fields::Unnamed(..) => {
                let items = params.iter().fold(quote! {}, |accum, param| {
                    let ty = &param.ident;
                    let vis = &param.field.vis;
                    quote! { #accum #vis #ty, }
                });
                (quote! { ( #items ) }, quote! {;})
            }
            Fields::Named(..) => {
                let items = params.iter().fold(quote! {}, |accum, param| {
                    let ty = &param.ident;
                    let fi = param.field.ident.as_ref().unwrap();
                    let vis = &param.field.vis;
                    quote! { #accum #vis #fi: #ty, }
                });
                (quote! { { #items } }, quote! {})
            }
        }
    }

    let init_ident = ctx.idents.init.as_ref().unwrap();
    let vis = &input.vis;

    let prefix = global_prefix(init_ident.span());
    let params = data_params(&input.data, &prefix);
    let item = match &input.data {
        Data::Struct(data) => {
            let args = make_args(params.iter().flatten());
            let (body, semi) = collect_fields(&data.fields, &prefix);
            quote! {
                #vis struct #init_ident<#args> #body #semi
            }
        }
        Data::Enum(data) => {
            let mut items = quote! {};
            let args = make_args(params.iter().flatten());
            for var in data.variants.iter() {
                let var_ident = &var.ident;
                let prefix = Ident::new(&format!("{}__{}", prefix, var_ident), var_ident.span());
                let (var_body, _) = collect_fields(&var.fields, &prefix);
                items = quote! {
                    #items
                    #var_ident #var_body,
                };
            }
            quote! { #vis enum #init_ident<#args> { #items } }
        }
        _ => unimplemented!(),
    };
    quote! {
        #[allow(non_camel_case_types)]
        #item
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

    let self_params = &input.generics.params;
    let self_args = generic::args(&input.generics);
    let where_clause = &input.generics.where_clause;

    let prefix = global_prefix(init_ident.span());
    let init_params = data_params(&input.data, &prefix);

    let init_args = make_args(init_params.iter().flatten());
    let all_params = extend_params(self_params, init_params.iter().flatten());

    let body = match &input.data {
        Data::Struct(data) => collect_fields(&data.fields, |i, f| {
            let item = field_postfix(i, f);
            quote! { self.#item }
        }),
        Data::Enum(data) => {
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
        #[allow(non_camel_case_types)]
        impl<#all_params> ::flatty::Emplacer<#self_ident<#self_args>> for #init_ident<#init_args>
        #where_clause
        {
            fn emplace<'__flatty_a>(
                self,
                uninit: &'__flatty_a mut ::flatty::mem::MaybeUninitUnsized<#self_ident<#self_args>>,
            ) -> Result<&'__flatty_a mut #self_ident<#self_args>, ::flatty::Error> {
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
