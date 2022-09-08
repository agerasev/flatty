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
            assert!(
                ctx.info.enum_type.is_none(),
                "`enum_type` is not allowed for `struct`",
            );
        }
        Data::Enum(_) => {
            if ctx.info.enum_type.is_none() {
                ctx.info.enum_type = Some(Ident::new("u8", Span::call_site()));
            }

            ctx.idents.tag = Some(Ident::new(
                &format!("{}Tag", input.ident),
                input.ident.span(),
            ));
            if !ctx.info.sized {
                ctx.idents.ref_ = Some(Ident::new(
                    &format!("{}Ref", input.ident),
                    input.ident.span(),
                ));
                ctx.idents.mut_ = Some(Ident::new(
                    &format!("{}Mut", input.ident),
                    input.ident.span(),
                ));
            }
        }
        Data::Union(_) => unimplemented!(),
    };
    if !ctx.info.sized {
        ctx.idents.align_as = Some(Ident::new(
            &format!("{}AlignAs", input.ident),
            input.ident.span(),
        ));
    }

    let specific = match (&input.data, ctx.info.sized) {
        (Data::Struct(_) | Data::Enum(_), true) => {
            let repr = match &ctx.info.enum_type {
                Some(ty) => quote! { #[repr(C, #ty)] },
                None => quote! { #[repr(C)] },
            };
            let derive_default = if ctx.info.default {
                quote! { #[derive(Default)] }
            } else {
                quote! {}
            };

            quote! {
                #derive_default
                #repr
                    #input
            }
        }
        (Data::Struct(_), false) => {
            let base_impl = items::base::impl_(&ctx, &input);
            let maybe_unsized_impl = items::maybe_unsized::impl_(&ctx, &input);
            let default_impl = items::default::impl_(&ctx, &input);

            quote! {
                #[repr(C)]
                #input

                #base_impl
                #maybe_unsized_impl
                #default_impl
            }
        }
        (Data::Enum(_), false) => {
            let struct_ = items::unsized_enum::struct_(&ctx, &input);
            let ref_ = items::unsized_enum::ref_(&ctx, &input);
            let mut_ = items::unsized_enum::mut_(&ctx, &input);

            let base_impl = items::base::impl_(&ctx, &input);
            let maybe_unsized_impl = items::maybe_unsized::impl_(&ctx, &input);
            let default_impl = items::default::impl_(&ctx, &input);

            quote! {
                #struct_
                #ref_
                #mut_

                #base_impl
                #maybe_unsized_impl
                #default_impl
            }
        }
        (Data::Union(_), _) => unimplemented!(),
    };

    let self_impl = items::self_::impl_(&ctx, &input);
    let cast_impl = items::cast::impl_(&ctx, &input);
    let flat_impl = items::flat::impl_(&ctx, &input);
    let portable_impl = if ctx.info.portable {
        items::portable::impl_(&ctx, &input)
    } else {
        quote! {}
    };

    TokenStream::from(quote! {
        #specific
        #self_impl
        #cast_impl
        #flat_impl
        #portable_impl
    })
}
