//! Flat message buffers with direct mapping to Rust types without packing/unpacking.
//!
//! # Main traits
//!
//! + [`Flat`] - type that occupies a single contiguous memory area. Guaranteed to have same binary representation on the same platform.
//! + [`Portable`] - flat type that has stable platform-independent binary representation and therefore it can be safely transfered between different platforms (of different address width or even different endianness).
//!
//! # Basic types
//!
//! ## Sized
//!
//! + Unit type (`()`).
//! + Signed and unsigned integers ([`u8`], [`i8`], [`u16`], [`i16`], [`u32`], [`i32`], [`u64`], [`i64`], [`u128`], [`i128`]).
//! + Floating-point numbers ([`f32`], [`f64`]).
//! + Array of some sized flat type ([`[T; N]`](`array`)` where T: `[`FlatSized`]).
//!
//! ## Unsized
//!
//! + Flat vector ([`FlatVec<T, L = u32>`](`FlatVec`)).
//!
//! # User-defined types
//!
//! User can create new composite types by using [`flat`] macro.
//!
//! ## Struct
//!
//! ```rust
//! #[flatty::flat]
//! struct SizedStruct {
//!     a: u8,
//!     b: u16,
//!     c: u32,
//!     d: [u64; 4],
//! }
//! ```
//!
//! ## Enum
//!
//! For enum you may explicitly set the type of tag (default value is [`u8`]).
//!
//! ```rust
//! #[flatty::flat(enum_type = "u32")]
//! enum SizedEnum {
//!     A,
//!     B(u16, u8),
//!     C { a: u8, b: u16 },
//!     D(u32),
//! }
//! ```
//!
//! ## Unsized struct
//!
//! Unsized struct is [DST](https://doc.rust-lang.org/reference/dynamically-sized-types.html). The reference to that structure contains its size.
//!
//! ```rust
//! #[flatty::flat(sized = false)]
//! struct UnsizedStruct {
//!     a: u8,
//!     b: u16,
//!     c: flatty::FlatVec<u64>,
//! }
//! ```
//!
//! ## Unsized enum
//!
//! Rust doesn't support [DST](https://doc.rust-lang.org/reference/dynamically-sized-types.html) enums yet so for now enum declaration is translated to unsized structure.
//!
//! But it has `as_ref`/`as_mut` methods that returns a native enum that contains references to original enum fields.
//!
//! ```rust
//! #[flatty::flat(sized = false)]
//! enum UnsizedEnum {
//!     A,
//!     B(u8, u16),
//!     C { a: u8, b: flatty::FlatVec<u8, u16> },
//! }
//! ```
//!
#![no_std]

pub use flatty_base::{
    emplacer::{self, Emplacer},
    error::{self, Error},
    traits::{self, Flat, FlatDefault, FlatSized},
    utils,
    vec::{self, flat_vec, FlatVec},
};
pub use flatty_macros::flat;
pub use flatty_portable as portable;

pub use portable::Portable;

pub mod prelude {
    pub use flatty_base::traits::*;
    pub use flatty_portable::prelude::*;
}
