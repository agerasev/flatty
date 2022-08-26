//! Flat message buffers.
//!
//! # Overview
//!
//! There are two main traits:
//!
//! + [`Flat`] - type that occupies a single contiguous memory area and could be accessed without packing/unpacking.
//! + [`Portable`] - type that has stable memory representation and therefore it could be safely transfered between different machines (of different address width or even different endianness).  
//!
//! The crate provides basic flat types and a macro to create new flat types.
//!
//! Message is represented as native Rust `struct` or `enum`.
//!
//! ## Basic types
//!
//! ### Sized
//!
//! + Unit type (`()`).
//! + Signed and unsigned integers ([`u8`], [`i8`], [`u16`], [`i16`], [`u32`], [`i32`], [`u64`], [`i64`], [`u128`], [`i128`]).
//! + Floating-point numbers ([`f32`], [`f64`]).
//! + Array of some sized flat type ([`[T; N]`](`array`)` where T: `[`FlatSized`](`FlatSized`)).
//!
//! ### Unsized
//!
//! + Flat vector ([`FlatVec<T, L = u32>`](`FlatVec`)).
//!
//! ## User-defined types
//!
//! ### Sized struct
//!
//! ```rust
//! #[flatty::make_flat]
//! struct SizedStruct {
//!     a: u8,
//!     b: u16,
//!     c: u32,
//!     d: [u64; 4],
//! }
//! ```
//!
//! ### Sized enum
//!
//! For enum you may explicitly set the type of variant index (default value is [`u8`]).
//!
//! ```rust
//! #[flatty::make_flat(enum_type = "u32")]
//! enum SizedEnum {
//!     A,
//!     B(u16, u8),
//!     C { a: u8, b: u16 },
//!     D(u32),
//! }
//! ```
//!
//! ### Unsized struct
//!
//! ```rust
//! #[flatty::make_flat(sized = false)]
//! struct UnsizedStruct {
//!     a: u8,
//!     b: u16,
//!     c: flatty::FlatVec<u64>,
//! }
//! ```
//!
//! ### Unsized enum
//!
//! ```rust
//! #[flatty::make_flat(sized = false)]
//! enum UnsizedEnum {
//!     A,
//!     B(u8, u16),
//!     C { a: u8, b: flatty::FlatVec<u8, u16> },
//! }
//! ```
//!

pub use base::*;
pub use macros::{self, make_flat};
