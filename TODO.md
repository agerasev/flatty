# TODO

+ In `flat` macro rename `enum_type` to `tag_type`.
+ In channel rename `emplace` method.
+ Support for `Box`-ed flat types.
+ Consider making `Emplacer` unsafe.
+ Don't return referencs to `Self` from methods (because we cannot prove that they actually point to `Self`).
+ Remove all traits from the crate root except `Flat`, `Portable` and `Emplacer`.
