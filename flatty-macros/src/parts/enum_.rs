use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{self, parse2, Data, DeriveInput, Fields, Ident, Type};

pub fn make_state(input: &DeriveInput) -> (Ident, TokenStream2) {
    let ident = Ident::new(&format!("{}State", input.ident), input.ident.span());
    let contents = match &input.data {
        Data::Struct(_) | Data::Union(_) => unimplemented!(),
        Data::Enum(enum_data) => enum_data.variants.iter().fold(quote! {}, |accum, variant| {
            let var_ident = &variant.ident;
            quote! {
                #accum
                #var_ident,
            }
        }),
    };
    (ident, contents)
}

fn make_mapped<F: Fn(&Type) -> Type>(input: &DeriveInput, map_ty: F) -> TokenStream2 {
    let contents = match &input.data {
        Data::Struct(_) | Data::Union(_) => unimplemented!(),
        Data::Enum(enum_data) => enum_data.variants.iter().fold(quote! {}, |accum, variant| {
            let var_ident = &variant.ident;
            let var_body = match &variant.fields {
                Fields::Named(fields) => {
                    let items = fields.named.iter().fold(quote! {}, |accum, field| {
                        let ty = map_ty(&field.ty);
                        let ident = field.ident.as_ref().unwrap();
                        quote! { #accum #ident: #ty, }
                    });
                    quote! { { #items } }
                }
                Fields::Unnamed(fields) => {
                    let items = fields.unnamed.iter().fold(quote! {}, |accum, field| {
                        let ty = map_ty(&field.ty);
                        quote! { #accum #ty, }
                    });
                    quote! { (#items) }
                }
                Fields::Unit => {
                    quote! {}
                }
            };
            quote! {
                #accum
                #var_ident #var_body,
            }
        }),
    };
    contents
}

pub fn make_ref(input: &DeriveInput) -> (Ident, TokenStream2) {
    let ident = Ident::new(&format!("{}Ref", input.ident), input.ident.span());
    let contents = make_mapped(input, |ty| {
        let stream = quote! { &'a #ty };
        parse2::<Type>(stream).unwrap()
    });
    (ident, contents)
}

pub fn make_mut(input: &DeriveInput) -> (Ident, TokenStream2) {
    let ident = Ident::new(&format!("{}Mut", input.ident), input.ident.span());
    let contents = make_mapped(input, |ty| {
        let stream = quote! { &'a mut #ty };
        parse2::<Type>(stream).unwrap()
    });
    (ident, contents)
}
