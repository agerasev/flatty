mod parts;
mod utils;

mod base;
mod portable;
mod sized;
mod unsized_enum;
mod unsized_struct;

use proc_macro::TokenStream;

/// Derive type from `FlatSized`.
///
/// *It is recommended to use [`make_flat()`] macro instead.*
#[proc_macro_derive(FlatSized)]
pub fn derive_flat_sized(stream: TokenStream) -> TokenStream {
    sized::derive(stream)
}

/// Create an unsized struct.
///
/// *It is recommended to use [`make_flat()`] macro instead.*
#[proc_macro_attribute]
pub fn make_flat_unsized_struct(attr: TokenStream, stream: TokenStream) -> TokenStream {
    unsized_struct::make(attr, stream)
}

/// Create an unsized enum.
///
/// *It is recommended to use [`make_flat()`] macro instead.*
#[proc_macro_attribute]
pub fn make_flat_unsized_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    unsized_enum::make(attr, item)
}

/// Derive type from `Portable`.
///
/// *It is recommended to use [`make_flat()`] macro instead.*
#[proc_macro_derive(Portable)]
pub fn derive_portable(stream: TokenStream) -> TokenStream {
    portable::derive(stream)
}

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
    base::make(attr, item)
}
