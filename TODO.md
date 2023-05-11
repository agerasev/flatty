# TODO

+ In `flat` macro rename `enum_type` to `tag_type`.
+ In channel rename `emplace` method.
+ Support for `Box`-ed flat types.
+ Consider making `Emplacer` unsafe.
+ Don't return referencs to `Self` from methods (because we cannot prove that they actually point to `Self`).
+ Remove all traits from the crate root except `Flat`, `Portable` and `Emplacer`.
+ Make `MaybeUninitUnsized` to store really uninit bytes inside or rename and edit docs.
+ Add to README brief comparison to [Postcard](https://github.com/jamesmunns/postcard), [rkiv](https://github.com/rkyv/rkyv), [FlatBuffers](https://github.com/google/flatbuffers), [Cap'n Proto](https://github.com/capnproto/capnproto), etc.
