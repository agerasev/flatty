use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{self, Data, DeriveInput, Fields, Ident};

fn make_fields(fields: &Fields) -> TokenStream2 {
    match fields {
        Fields::Named(fields) => {
            let contents = fields.named.iter().fold(quote! {}, |accum, field| {
                let field_vis = &field.vis;
                let field_ty = &field.ty;
                let field_ident = field.ident.as_ref().unwrap();
                quote! {
                    #accum
                    #field_vis #field_ident: <#field_ty as ::flatty::FlatInit>::Dyn,
                }
            });
            quote! { { #contents } }
        }
        Fields::Unnamed(fields) => {
            let contents = fields.unnamed.iter().fold(quote! {}, |accum, field| {
                let field_ty = &field.ty;
                assert!(field.ident.is_none());
                quote! { #accum <#field_ty as ::flatty::FlatInit>::Dyn, }
            });
            quote! { (#contents) }
        }
        Fields::Unit => quote! {},
    }
}

pub fn ident(input: &DeriveInput) -> Ident {
    Ident::new(&format!("{}Dyn", input.ident), input.ident.span())
}

pub fn make(input: &DeriveInput) -> (Ident, TokenStream2) {
    let ident = ident(input);
    let body = match &input.data {
        Data::Struct(struct_data) => {
            let body = make_fields(&struct_data.fields);
            let semi = match &struct_data.fields {
                Fields::Unnamed(_) | Fields::Unit => quote! { ; },
                Fields::Named(_) => quote! {},
            };
            quote! { #body #semi }
        }
        Data::Enum(enum_data) => {
            let contents = enum_data.variants.iter().fold(quote! {}, |accum, variant| {
                let var_body = make_fields(&variant.fields);
                let var_ident = &variant.ident;
                let default = if variant
                    .attrs
                    .iter()
                    .any(|attr| attr.path.is_ident("default"))
                {
                    quote! { #[default] }
                } else {
                    quote! {}
                };
                quote! {
                    #accum
                    #default
                    #var_ident #var_body,
                }
            });
            quote! { { #contents }}
        }
        Data::Union(_union_data) => unimplemented!(),
    };
    (ident, body)
}
