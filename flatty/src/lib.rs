//! Flat message buffers.
//!
//! # Overview
//!
//! The crate provides basic flat types and a macro to create new flat types. Flat means that it occupies a single contiguous memory area.
//!
//! Flat types have stable binary representation can be safely transferred between machines (of the same endianness) as is without packing/unpacking.
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

pub use base_::*;
pub use macros::{self, make_flat};
