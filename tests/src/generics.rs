#![allow(dead_code)]

use flatty::{flat, traits::FlatValidate, Flat, FlatVec};
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug)]
struct Unused<T>(PhantomData<T>);
unsafe impl<T> Send for Unused<T> {}
unsafe impl<T> Sync for Unused<T> {}
unsafe impl<T> Flat for Unused<T> {}
unsafe impl<T> FlatValidate for Unused<T> {
    unsafe fn validate_unchecked(_: &[u8]) -> Result<(), flatty::Error> {
        Ok(())
    }
}
impl<T> Default for Unused<T> {
    fn default() -> Self {
        Unused(PhantomData)
    }
}

#[flat]
#[derive(Default)]
struct GenericSizedStruct<'a, S: Flat, T: Flat, U, const N: usize>
where
    U: 'a,
    [T; N]: Default,
{
    a: S,
    b: [T; N],
    c: Unused<&'a U>,
}

#[flat]
#[derive(Default)]
enum GenericSizedEnum<'a, S: Flat, T: Flat, U, const N: usize>
where
    U: 'a,
    [T; N]: Default,
    S: Default,
    T: Default,
{
    A(S, T),
    B([T; N]),
    C {
        x: T,
        _p: Unused<&'a U>,
    },
    D(GenericSizedStruct<'a, S, T, U, N>),
    #[default]
    E,
}

#[flat(sized = false, default = true)]
struct GenericUnsizedStruct<'a, T: Flat, U, const N: usize>
where
    U: 'a,
    [T; N]: Default,
{
    a: [T; N],
    b: Unused<&'a U>,
    c: FlatVec<T>,
}

#[flat(sized = false, default = true)]
enum GenericUnsizedEnum<'a, S: Flat, T: Flat, U, const N: usize>
where
    U: 'a,
    [T; N]: Default,
    S: Default,
    T: Default,
{
    A(S, T),
    B([T; N], FlatVec<T>),
    C {
        x: T,
        _p: Unused<&'a U>,
    },
    D(GenericUnsizedStruct<'a, T, U, N>),
    #[default]
    E,
}
