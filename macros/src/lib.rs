mod context;
mod info;
mod items;
mod utils;

use context::{AssocIdents, Context};
use info::Info;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Ident};

/// Attribute macro that creates a flat type from `struct` or `enum` declaration.
///
/// # Usage examples
///
/// ```rust_no_check
/// #[flatty::make_flat(sized = false)]
/// struct ... { ... }
/// ```
///
/// or
///
/// ```rust_no_check
/// #[flatty::make_flat(sized = false, enum_type = "u32")]
/// enum ... { ... }
/// ```
///
/// # Arguments
///
/// + `sized: bool`, optional, `true` by default. Whether structure is sized or not.
/// + `enum_type: str`, for `enum` declaration only, optional, `"u8"` by default.
///   The type used for enum variant index. Possible valiues: `"u8"`, `"u16"`, `"u32"`.
/// + `portable: bool`, optional, `false` by default. Whether structure should implement `Portable`.
#[proc_macro_attribute]
pub fn make_flat(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut ctx = Context {
        info: parse_macro_input!(attr as Info),
        idents: AssocIdents::default(),
    };
    let input = parse_macro_input!(item as DeriveInput);

    match &input.data {
        Data::Struct(_) => {
            assert!(ctx.info.enum_type.is_none(), "`enum_type` is not allowed for `struct`",);
        }
        Data::Enum(_) => {
            if ctx.info.enum_type.is_none() {
                ctx.info.enum_type = Some(Ident::new("u8", Span::call_site()));
            }

            ctx.idents.tag = Some(Ident::new(&format!("{}Tag", input.ident), input.ident.span()));
            if !ctx.info.sized {
                ctx.idents.ref_ = Some(Ident::new(&format!("{}Ref", input.ident), input.ident.span()));
                ctx.idents.mut_ = Some(Ident::new(&format!("{}Mut", input.ident), input.ident.span()));
            }
        }
        Data::Union(_) => unimplemented!(),
    };
    if !ctx.info.sized {
        ctx.idents.align_as = Some(Ident::new(&format!("{}AlignAs", input.ident), input.ident.span()));
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

                let base_impl = items::base::impl_(&ctx, &input);
                let unsized_impl = items::unsized_::impl_(&ctx, &input);
                let emplacer_struct = items::emplacer::struct_(&ctx, &input);
                let default_impl = if ctx.info.default {
                    items::default::impl_(&ctx, &input)
                } else {
                    quote! {}
                };

                quote! {
                    #[repr(C)]
                    #input

                    #align_as_struct

                    #base_impl
                    #unsized_impl
                    #default_impl
                }
            }
        }
        Data::Enum(_) => {
            let tag_struct = items::tag::struct_(&ctx, &input, ctx.info.sized);
            if ctx.info.sized {
                let enum_type = ctx.info.enum_type.as_ref().unwrap();
                let derive_default = if ctx.info.default {
                    quote! { #[derive(Default)] }
                } else {
                    quote! {}
                };

                quote! {
                    #derive_default
                    #[repr(C, #enum_type)]
                    #input

                    #tag_struct
                }
            } else {
                let struct_ = items::unsized_enum::struct_(&ctx, &input);
                let align_as_struct = items::align_as::struct_(&ctx, &input);
                let ref_struct = items::unsized_enum::ref_struct(&ctx, &input);
                let mut_struct = items::unsized_enum::mut_struct(&ctx, &input);

                let tag_impl = items::tag::impl_(&ctx, &input);
                let ref_impl = items::unsized_enum::ref_impl(&ctx, &input);
                let mut_impl = items::unsized_enum::mut_impl(&ctx, &input);

                let base_impl = items::base::impl_(&ctx, &input);
                let unsized_impl = items::unsized_::impl_(&ctx, &input);
                let emplacer_struct = items::emplacer::struct_(&ctx, &input);
                let default_impl = if ctx.info.default {
                    items::default::impl_(&ctx, &input)
                } else {
                    quote! {}
                };

                quote! {
                    #struct_

                    #align_as_struct
                    #tag_struct
                    #ref_struct
                    #mut_struct

                    #tag_impl
                    #ref_impl
                    #mut_impl

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

    TokenStream::from(quote! {
        #specific

        #base_self_impl
        #cast_impl
        #flat_impl
        #portable_impl
    })
}
