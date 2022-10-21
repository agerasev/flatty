#![allow(dead_code)]

use flatty::{flat, Flat, FlatVec};
use std::marker::PhantomData;

#[flat]
struct GenericSizedStruct<'a, S: Flat, T: Flat, U, const N: usize>
where
    U: 'a,
    [T; N]: Default,
{
    a: S,
    b: [T; N],
    c: PhantomData<&'a U>,
}

#[flat]
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
        _p: PhantomData<&'a U>,
    },
    D(GenericSizedStruct<'a, S, T, U, N>),
    #[default]
    E,
}

#[flat(sized = false)]
struct GenericUnsizedStruct<'a, T: Flat, U, const N: usize>
where
    U: 'a,
    [T; N]: Default,
{
    a: [T; N],
    b: PhantomData<&'a U>,
    c: FlatVec<T>,
}

#[flat(sized = false)]
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
        _p: PhantomData<&'a U>,
    },
    D(GenericUnsizedStruct<'a, T, U, N>),
    #[default]
    E,
}
