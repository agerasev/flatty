mod flat_attribute;

use proc_macro::TokenStream;
use quote::ToTokens;

/// Attribute macro that creates a flat type from `struct` or `enum` declaration.
///
/// # Usage examples
///
/// ```rust_no_check
/// #[flatty::flat(sized = false)]
/// struct ... { ... }
/// ```
///
/// or
///
/// ```rust_no_check
/// #[flatty::flat(sized = false, tag_type = "u32")]
/// enum ... { ... }
/// ```
///
/// # Arguments
///
/// + `sized: bool`, optional, `true` by default. Whether structure is sized or not.
/// + `tag_type: str`, for `enum` declaration only, optional, `"u8"` by default.
///   The type used for enum variant index. Possible values: `"u8"`, `"u16"`, `"u32"`.
/// + `portable: bool`, optional, `false` by default. Whether structure should implement `Portable`.
/// + `default: bool`, optional, `false` by default. Whether to create default constructors (see `FlatDefault`).
#[proc_macro_attribute]
pub fn flat(attr: TokenStream, item: TokenStream) -> TokenStream {
    match flat_attribute::apply(attr.into(), item.into()) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}

#[proc_macro_derive(Flat)]
pub fn derive_flat(_input: TokenStream) -> TokenStream {
    unimplemented!()
}

#[proc_macro_derive(Portable)]
pub fn derive_portable(_input: TokenStream) -> TokenStream {
    unimplemented!()
}
