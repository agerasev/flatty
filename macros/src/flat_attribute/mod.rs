mod context;
mod info;
mod items;
mod utils;

use context::{AssocIdents, Context};
use info::Info;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse2, Data, DeriveInput, Ident, Result};

pub fn apply(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let mut ctx = Context {
        info: parse2(attr)?,
        idents: AssocIdents::default(),
        c_like_enum: None,
    };
    let input: DeriveInput = parse2(item)?;

    match &input.data {
        Data::Struct(_) => {
            assert!(ctx.info.tag_type.is_none(), "`tag_type` is not allowed for `struct`",);
        }
        Data::Enum(_) => {
            if ctx.info.tag_type.is_none() {
                ctx.info.tag_type = Some(Ident::new("u8", Span::call_site()));
            }
            if items::enum_::is_c_like(&input) {
                ctx.c_like_enum = Some(true);
                ctx.idents.tag = Some(input.ident.clone());
            } else {
                ctx.c_like_enum = Some(false);
                ctx.idents.tag = Some(Ident::new(&format!("{}Tag", input.ident), input.ident.span()));
            };
            if !ctx.info.sized {
                ctx.idents.ref_ = Some(Ident::new(&format!("{}Ref", input.ident), input.ident.span()));
                ctx.idents.mut_ = Some(Ident::new(&format!("{}Mut", input.ident), input.ident.span()));
            }
        }
        Data::Union(_) => unimplemented!(),
    };
    if !ctx.info.sized {
        ctx.idents.align_as = Some(Ident::new(&format!("{}AlignAs", input.ident), input.ident.span()));
        ctx.idents.init = Some(Ident::new(&format!("{}Init", input.ident), input.ident.span()));
    }

    let specific = match &input.data {
        Data::Struct(_) => {
            if ctx.info.sized {
                let derive_default = if ctx.info.default {
                    quote! { #[derive(Default)] }
                } else {
                    quote! {}
                };

                quote! {
                    #derive_default
                    #[repr(C)]
                    #input
                }
            } else {
                let align_as_struct = items::align_as::struct_(&ctx, &input);
                let init_struct = items::init::struct_(&ctx, &input);

                let init_impl = items::init::impl_(&ctx, &input);

                let base_impl = items::base::impl_(&ctx, &input);
                let unsized_impl = items::unsized_::impl_(&ctx, &input);
                let default_impl = if ctx.info.default {
                    items::init::impl_default(&ctx, &input)
                } else {
                    quote! {}
                };

                quote! {
                    #[repr(C)]
                    #input

                    #align_as_struct
                    #init_struct

                    #init_impl

                    #base_impl
                    #unsized_impl
                    #default_impl
                }
            }
        }
        Data::Enum(_) => {
            if ctx.info.sized {
                let tag_type = ctx.info.tag_type.as_ref().unwrap();
                let derive_default = if ctx.info.default {
                    quote! { #[derive(Default)] }
                } else {
                    quote! {}
                };
                let (repr, tag_struct) = if !ctx.c_like_enum.unwrap() {
                    (quote! { #[repr(C, #tag_type)] }, items::tag::struct_(&ctx, &input, true))
                } else {
                    (quote! { #[repr(#tag_type)] }, quote! {})
                };

                quote! {
                    #derive_default
                    #repr
                    #input

                    #tag_struct
                }
            } else {
                assert!(!ctx.c_like_enum.unwrap(), "C-like enums cannot be unsized");

                let struct_ = items::unsized_enum::struct_(&ctx, &input);
                let align_as_struct = items::align_as::struct_(&ctx, &input);
                let tag_struct = items::tag::struct_(&ctx, &input, false);
                let ref_struct = items::unsized_enum::ref_struct(&ctx, &input);
                let mut_struct = items::unsized_enum::mut_struct(&ctx, &input);
                let init_struct = items::init::struct_(&ctx, &input);

                let tag_impl = items::tag::impl_(&ctx, &input);
                let ref_impl = items::unsized_enum::ref_impl(&ctx, &input);
                let mut_impl = items::unsized_enum::mut_impl(&ctx, &input);
                let init_impl = items::init::impl_(&ctx, &input);

                let base_impl = items::base::impl_(&ctx, &input);
                let unsized_impl = items::unsized_::impl_(&ctx, &input);
                let default_impl = if ctx.info.default {
                    items::init::impl_default(&ctx, &input)
                } else {
                    quote! {}
                };

                quote! {
                    #struct_

                    #align_as_struct
                    #tag_struct
                    #ref_struct
                    #mut_struct
                    #init_struct

                    #tag_impl
                    #ref_impl
                    #mut_impl
                    #init_impl

                    #base_impl
                    #unsized_impl
                    #default_impl
                }
            }
        }
        Data::Union(_) => unimplemented!(),
    };

    let base_self_impl = items::base::self_impl(&ctx, &input);
    let cast_impl = items::cast::impl_(&ctx, &input);
    let flat_impl = items::flat::impl_(&ctx, &input);
    let portable_impl = if ctx.info.portable {
        items::portable::impl_(&ctx, &input)
    } else {
        quote! {}
    };

    Ok(quote! {
        #specific

        #base_self_impl
        #cast_impl
        #flat_impl
        #portable_impl
    })
}
