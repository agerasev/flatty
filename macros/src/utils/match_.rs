use proc_macro2::TokenStream;
use quote::quote;
use syn::{self, spanned::Spanned, Fields, Ident};

pub struct Bindings {
    pub pattern: TokenStream,
    pub wrapper: TokenStream,
    pub prefix: TokenStream,
}

pub fn bindings(fields: &Fields) -> Bindings {
    match fields {
        Fields::Named(named_fields) => {
            let pattern = named_fields.named.iter().fold(quote! {}, |accum, field| {
                let ident = field.ident.as_ref().unwrap();
                quote! { #accum #ident, }
            });
            Bindings {
                pattern: quote! { { #pattern } },
                wrapper: quote! {},
                prefix: quote! {},
            }
        }
        Fields::Unnamed(unnamed_fields) => {
            let pattern = unnamed_fields.unnamed.iter().enumerate().fold(
                quote! {},
                |accum, (index, field)| {
                    let ident = Ident::new(&format!("b{}", index), field.span());
                    quote! { #accum #ident, }
                },
            );
            Bindings {
                pattern: quote! { (#pattern) },
                wrapper: quote! { let wrapper = (#pattern); },
                prefix: quote! { wrapper. },
            }
        }
        Fields::Unit => Bindings {
            pattern: quote! {},
            wrapper: quote! {},
            prefix: quote! {},
        },
    }
}
