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

Flat message buffers with direct mapping to Rust types without packing/unpacking.

## Overview

Type called flat when it occupies a single contiguous memory area. Such types is useful when you need to store or send such object as a binary data while having convenient way to access its contents without serializing/deserializing.

This crate provides basic flat types and the way to create new user-defined composite flat types (using `#[flat]` attribute macro), which can be used almost like regular Rust `struct`s or `enum`s.

Also the crate can be used without `std` and even `alloc`.

## Concepts

### [DST](https://doc.rust-lang.org/reference/dynamically-sized-types.html)

Flat type can be dynamically sized (like `FlatVec`), in that case it exploits Rust's ability to operate `?Sized` types. User-defined flat struct can also be unsized (only last field is allowed to be unsized). Even flat enum can be unsized, but Rust doesn't natively support them yet so its contents could be accessed only via `as_ref`/`as_mut` methods returning a regular enum containing references to original enum contents.

### `Emplacer`

Rust cannot construct DST in usual manner on stack because its size isn't known at compile time. Instead we may initialize such types onto given memory area. To do this we can use so-called emplacer - something that can initialize object onto given memory. For sized types its instance is also emplacer.

### `MaybeUninitUnsized`

It's like `MaybeUninit` but for unsized flat types. It checks that underlying memory is properly aligned and large enough to contain minimally-sized instance of the type, but its contents is uninitialized.

### `Portable`

Flat type guarantee that it has the same binary representation on the platforms with same byte order, alignment and address width. If you need stronger guarantees you may use `Portable` types - they have the same binary representation on *any* platform and always aligned to byte. To make own flat type portable use `#[flat(portable = true)]`. Also this can be used to created packed flat types without alignment issues. 

## Examples

You can find some examples on how to create and use flat types in [`tests`](tests/src/) directory.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
