use crate::{
    utils::{generic, type_list, FieldIter},
    Context,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Comma, Data, DeriveInput, Field, Fields, GenericParam, Ident, Index,
};

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

fn field_param<'a: 'b, 'b>(i: usize, f: &'a Field, prefix: &'b str) -> Param<'a> {
    let name = match &f.ident {
        Some(ident) => format!("{}__{}", prefix, ident.to_string().to_uppercase()),
        None => format!("{}__{}", prefix, i),
    };
    Param::new(Ident::new(&name, f.span()), f)
}

fn fields_params<'a: 'b, 'b>(fields: &'a Fields, prefix: &'b str) -> Vec<Param<'a>> {
    fields.iter().enumerate().map(|(i, f)| field_param(i, f, prefix)).collect()
}

fn data_params<'a: 'b, 'b>(data: &'a Data, prefix: &'b str) -> Vec<Vec<Param<'a>>> {
    match data {
        Data::Struct(struct_) => vec![fields_params(&struct_.fields, prefix)],
        Data::Enum(enum_) => enum_
            .variants
            .iter()
            .map(|var| {
                let prefix = format!("{}__{}", prefix, var.ident);
                fields_params(&var.fields, &prefix)
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

fn variant_ident(init_ident: &Ident, var_ident: &Ident) -> Ident {
    Ident::new(&format!("{}{}", init_ident, var_ident), var_ident.span())
}

pub fn struct_(ctx: &Context, input: &DeriveInput) -> TokenStream {
    fn collect_fields(fields: &Fields, prefix: &str, pub_: bool) -> (TokenStream, TokenStream) {
        let get_vis = |param: &Param| {
            if pub_ {
                quote! { pub }
            } else {
                let vis = &param.field.vis;
                quote! { #vis }
            }
        };
        let params = fields_params(fields, prefix);
        match fields {
            Fields::Unit => (quote! {}, quote! {;}),
            Fields::Unnamed(..) => {
                let items = params.iter().fold(quote! {}, |accum, param| {
                    let ty = &param.ident;
                    let vis = get_vis(param);
                    quote! { #accum #vis #ty, }
                });
                (quote! { ( #items ) }, quote! {;})
            }
            Fields::Named(..) => {
                let items = params.iter().fold(quote! {}, |accum, param| {
                    let ty = &param.ident;
                    let fi = param.field.ident.as_ref().unwrap();
                    let vis = get_vis(param);
                    quote! { #accum #vis #fi: #ty, }
                });
                (quote! { { #items } }, quote! {})
            }
        }
    }

    let init_ident = ctx.idents.init.as_ref().unwrap();
    let vis = &input.vis;

    let prefix = "";
    let params = data_params(&input.data, &prefix);
    match &input.data {
        Data::Struct(data) => {
            let args = make_args(params.iter().flatten());
            let (body, semi) = collect_fields(&data.fields, &prefix, false);
            quote! {
                #[allow(non_camel_case_types)]
                #vis struct #init_ident<#args> #body #semi
            }
        }
        Data::Enum(data) => {
            let mut items = quote! {};
            let mut variants = quote! {};
            for var in data.variants.iter() {
                let var_ident = &var.ident;
                let prefix = format!("{}__{}", prefix, var_ident);
                let (var_body, _) = collect_fields(&var.fields, &prefix, false);
                variants = quote! {
                    #variants
                    #var_ident #var_body,
                };

                let var_init_ident = variant_ident(init_ident, var_ident);
                let var_args = make_args(fields_params(&var.fields, "").iter());
                let (body, semi) = collect_fields(&var.fields, "", true);
                items = quote! {
                    #items

                    #[allow(non_camel_case_types)]
                    #vis struct #var_init_ident<#var_args> #body #semi
                }
            }

            let args = make_args(params.iter().flatten());
            quote! {
                #[allow(non_camel_case_types)]
                #vis enum #init_ident<#args> { #variants }

                #items
            }
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
                let iter = unsafe { iter::MutIter::new_unchecked(
                    bytes,
                    iter::type_list!(#type_list),
                ) };
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

    let init_params = data_params(&input.data, "");

    let body = match &input.data {
        Data::Struct(data) => {
            let body = collect_fields(&data.fields, |i, f| {
                let item = field_postfix(i, f);
                quote! { self.#item }
            });
            quote! {
                let bytes = uninit.as_mut_bytes();
                #body
            }
        }
        Data::Enum(data) => {
            let tag_ident = ctx.idents.tag.as_ref().unwrap();
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
                let tag = quote! {
                    unsafe { ::flatty::mem::MaybeUninitUnsized::<#tag_ident>::from_mut_bytes_unchecked(bytes) }
                        .as_mut_sized()
                        .write(#tag_ident::#ident);
                };
                let body = collect_fields(&var.fields, get_item);
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
                        let mut bytes = uninit.as_mut_bytes();
                        #tag
                        bytes = unsafe{ bytes.get_unchecked_mut(<#self_ident<#self_args>>::DATA_OFFSET..) };
                        #body
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

    let mut impls = quote! {};
    if let Data::Enum(data) = &input.data {
        for (i, var) in data.variants.iter().enumerate() {
            let var_ident = &var.ident;
            let var_init_ident = variant_ident(init_ident, var_ident);

            let params = fields_params(&var.fields, "");
            let args = make_args(params.iter());
            let all_params = extend_params(self_params, params.iter());

            let mut init_args = quote! {};
            for (j, ref_params) in init_params.iter().enumerate() {
                let ia = if i != j {
                    (0..(ref_params.len())).into_iter().fold(quote! {}, |a, _| {
                        quote! { #a ::flatty::NeverEmplacer, }
                    })
                } else {
                    make_args(params.iter())
                };
                init_args = quote! { #init_args #ia };
            }

            let body = match &var.fields {
                Fields::Unit => quote! {},
                Fields::Unnamed(fields) => {
                    let items = fields.unnamed.iter().enumerate().fold(quote! {}, |a, (i, _)| {
                        let index = Index::from(i);
                        quote! { #a this.#index, }
                    });
                    quote! { (#items) }
                }
                Fields::Named(fields) => {
                    let items = fields.named.iter().fold(quote! {}, |a, f| {
                        let ident = f.ident.as_ref().unwrap();
                        quote! { #a #ident: this.#ident, }
                    });
                    quote! { { #items } }
                }
            };

            impls = quote! {
                #impls

                #[allow(non_camel_case_types)]
                impl<#args> From<#var_init_ident<#args>> for #init_ident<#init_args> {
                    fn from(this: #var_init_ident<#args>) -> Self {
                        Self::#var_ident #body
                    }
                }

                #[allow(non_camel_case_types)]
                impl<#all_params> ::flatty::Emplacer<#self_ident<#self_args>> for #var_init_ident<#args>
                #where_clause
                {
                    fn emplace<'__flatty_a>(
                        self,
                        uninit: &'__flatty_a mut ::flatty::mem::MaybeUninitUnsized<#self_ident<#self_args>>,
                    ) -> Result<&'__flatty_a mut #self_ident<#self_args>, ::flatty::Error> {
                        <#init_ident<#init_args> as From<Self>>::from(self).emplace(uninit)
                    }
                }
            }
        }
    }

    let init_args = make_args(init_params.iter().flatten());
    let all_params = extend_params(self_params, init_params.iter().flatten());
    quote! {
        #impls

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
