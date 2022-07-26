# flatty

[![Crates.io][crates_badge]][crates]
[![Docs.rs][docs_badge]][docs]
[![Github Actions][github_badge]][github]
[![License][license_badge]][license]

[crates_badge]: https://img.shields.io/crates/v/flatty.svg
[docs_badge]: https://docs.rs/flatty/badge.svg
[github_badge]: https://github.com/agerasev/flatty/actions/workflows/test.yml/badge.svg
[license_badge]: https://img.shields.io/crates/l/flatty.svg

[crates]: https://crates.io/crates/flatty
[docs]: https://docs.rs/flatty
[github]: https://github.com/agerasev/flatty/actions/workflows/test.yml
[license]: #license

Flat message buffers.

# Overview

The crate provides basic flat types and a macro to create new flat types. Flat means that it occupies a single contiguous memory area.

Flat types have stable binary representation can be safely transferred between machines (of the same endianness) as is without packing/unpacking.

Message is represented as native Rust `struct` or `enum`.

## Basic types

### Sized

+ Unit type (`()`).
+ Signed and unsigned integers (`u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `u128`, `i128`).
+ Floating-point numbers (`f32`, `f64`).
+ Array of some sized flat type (`[T; N] where T: FlatSized`).

### Unsized

+ Flat vector (`FlatVec<T, L = u32>`).

## User-defined types

### Sized struct

```rust
#[flatty::make_flat]
struct SizedStruct {
    a: u8,
    b: u16,
    c: u32,
    d: [u64; 4],
}
```

### Sized enum

For enum you need to explicitly set the type of variant index.

```rust
#[flatty::make_flat(enum_type = "u8")]
enum SizedEnum {
    A,
    B(u16, u8),
    C { a: u8, b: u16 },
    D(u32),
}
```

### Unsized struct

Unsized struct is Rust DST. The reference to that structure contains its size.

```rust
#[flatty::make_flat(sized = false)]
struct UnsizedStruct {
    a: u8,
    b: u16,
    c: flatty::FlatVec<u64>,
}

```

### Unsized enum

Rust doesn't support DST enums yet so for now enum declaration is translated to unsized structure.

But it has `as_ref`/`as_mut` methods that returns a native enum that contains references to original enum fields.

```rust
#[flatty::make_flat(sized = false, enum_type = "u8")]
enum UnsizedEnum {
    A,
    B(u8, u16),
    C { a: u8, b: flatty::FlatVec<u8, u16> },
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
