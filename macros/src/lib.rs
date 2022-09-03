mod context;
mod impl_;
mod info;
mod utils;

use context::Context;
use info::Info;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

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
    let ctx = Context {
        info: parse_macro_input!(attr as Info),
    };
    let input = parse_macro_input!(item as DeriveInput);

    TokenStream::from(match (&input.data, ctx.info.sized) {
        (data @ (Data::Struct(_) | Data::Enum(_)), true) => {
            let repr = match data {
                Data::Struct(_) => {
                    assert!(
                        ctx.info.enum_type.is_none(),
                        "`enum_type` is not allowed for `struct`",
                    );
                    quote! { #[repr(C)] }
                }
                Data::Enum(_) => match &ctx.info.enum_type {
                    Some(ty) => quote! { #[repr(C, #ty)] },
                    None => quote! { #[repr(C, u8)] },
                },
                Data::Union(_) => unreachable!(),
            };
            let derive_default = if ctx.info.default {
                quote! { #[derive(Default)] }
            } else {
                quote! {}
            };

            let cast_impl = impl_::cast(&ctx, &input);
            let flat_impl = impl_::flat(&ctx, &input);

            quote! {
                #derive_default
                #repr
                #input

                #cast_impl
                #flat_impl
            }
        }
        (Data::Struct(_), false) => {
            quote! {
                #[repr(C)]
                #input
            }
        }
        (Data::Enum(_), false) => {
            quote! {}
        }
        (Data::Union(_), _) => unimplemented!(),
    })
}
