use crate::{impl_traits_for_native, Portable};
use core::{
    cmp::{Ord, Ordering, PartialOrd},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
};
use flatty_base::{
    error::Error,
    traits::{Flat, FlatValidate},
};
use num_traits::{Bounded, FromPrimitive, Num, NumCast, One, Signed, ToPrimitive, Unsigned, Zero};

/// Generic portable integer. Has alignment == 1.
///
/// Parameters:
/// + `BE`: Endianness. `false` => little-endian, `true` => big-endian.
/// + `N`: Width in bytes.
/// + `S`: Whether this type is signed (`true`) or unsigned (`false`).
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Int<const BE: bool, const N: usize, const S: bool> {
    bytes: [u8; N],
}

impl<const BE: bool, const N: usize, const S: bool> Default for Int<BE, N, S> {
    fn default() -> Self {
        Self { bytes: [0; N] }
    }
}

impl<const BE: bool, const N: usize, const S: bool> Int<BE, N, S> {
    pub fn from_bytes(bytes: [u8; N]) -> Self {
        Self { bytes }
    }
    pub fn to_bytes(self) -> [u8; N] {
        self.bytes
    }
}

unsafe impl<const BE: bool, const N: usize, const S: bool> FlatValidate for Int<BE, N, S> {
    unsafe fn validate_unchecked(_: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}

unsafe impl<const BE: bool, const N: usize, const S: bool> Flat for Int<BE, N, S> {}

unsafe impl<const BE: bool, const N: usize, const S: bool> Portable for Int<BE, N, S> {}

macro_rules! derive_int {
    ($self:ty, $native:ty, $from_bytes:ident, $to_bytes:ident$(,)?) => {
        impl $self {
            fn from_native(n: $native) -> Self {
                Int::from_bytes(n.$to_bytes())
            }
            fn to_native(self) -> $native {
                <$native>::$from_bytes(self.to_bytes())
            }
        }

        impl From<$native> for $self {
            fn from(n: $native) -> Self {
                Self::from_native(n)
            }
        }
        impl From<$self> for $native {
            fn from(s: $self) -> Self {
                s.to_native()
            }
        }

        impl NumCast for $self {
            fn from<T: ToPrimitive>(n: T) -> Option<Self> {
                Some(Self::from_native(<$native as NumCast>::from::<T>(n)?))
            }
        }
        impl ToPrimitive for $self {
            fn to_u64(&self) -> Option<u64> {
                self.to_native().to_u64()
            }
            fn to_i64(&self) -> Option<i64> {
                self.to_native().to_i64()
            }
        }
        impl FromPrimitive for $self {
            fn from_u64(n: u64) -> Option<Self> {
                Some(Self::from_native(<$native>::from_u64(n)?))
            }
            fn from_i64(n: i64) -> Option<Self> {
                Some(Self::from_native(<$native>::from_i64(n)?))
            }
        }

        impl Num for $self {
            type FromStrRadixErr = <$native as Num>::FromStrRadixErr;
            fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
                Ok(Self::from_native(<$native as Num>::from_str_radix(str, radix)?))
            }
        }

        impl Bounded for $self {
            fn min_value() -> Self {
                Self::from_native(<$native>::MIN)
            }
            fn max_value() -> Self {
                Self::from_native(<$native>::MAX)
            }
        }

        impl One for $self {
            fn one() -> Self {
                Self::from_native(<$native>::one())
            }
        }
        impl Zero for $self {
            fn zero() -> Self {
                Self::from_native(<$native>::zero())
            }
            fn is_zero(&self) -> bool {
                self.to_native().is_zero()
            }
        }

        impl Add for $self {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                Self::from_native(self.to_native() + rhs.to_native())
            }
        }
        impl Sub for $self {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                Self::from_native(self.to_native() - rhs.to_native())
            }
        }
        impl Mul for $self {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self {
                Self::from_native(self.to_native() * rhs.to_native())
            }
        }
        impl Div for $self {
            type Output = Self;
            fn div(self, rhs: Self) -> Self {
                Self::from_native(self.to_native() / rhs.to_native())
            }
        }
        impl Rem for $self {
            type Output = Self;
            fn rem(self, rhs: Self) -> Self {
                Self::from_native(self.to_native() % rhs.to_native())
            }
        }

        impl Ord for $self {
            fn cmp(&self, other: &Self) -> Ordering {
                self.to_native().cmp(&other.to_native())
            }
        }
        impl PartialOrd for $self {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.to_native().cmp(&other.to_native()))
            }
        }

        impl AddAssign for $self {
            fn add_assign(&mut self, rhs: Self) {
                *self = self.add(rhs);
            }
        }
        impl SubAssign for $self {
            fn sub_assign(&mut self, rhs: Self) {
                *self = self.sub(rhs);
            }
        }
        impl MulAssign for $self {
            fn mul_assign(&mut self, rhs: Self) {
                *self = self.mul(rhs);
            }
        }
        impl DivAssign for $self {
            fn div_assign(&mut self, rhs: Self) {
                *self = self.div(rhs);
            }
        }
        impl RemAssign for $self {
            fn rem_assign(&mut self, rhs: Self) {
                *self = self.rem(rhs);
            }
        }

        impl_traits_for_native!($self, $native);
    };
}

macro_rules! derive_int_signed {
    ($self:ty, $native:ty, $from_bytes:ident, $to_bytes:ident$(,)?) => {
        derive_int!($self, $native, $from_bytes, $to_bytes);

        impl Signed for $self {
            fn abs(&self) -> Self {
                Self::from_native(self.to_native().abs())
            }
            fn abs_sub(&self, other: &Self) -> Self {
                Self::from_native(self.to_native().abs_sub(&other.to_native()))
            }
            fn signum(&self) -> Self {
                Self::from_native(self.to_native().signum())
            }
            fn is_positive(&self) -> bool {
                self.to_native().is_positive()
            }
            fn is_negative(&self) -> bool {
                self.to_native().is_negative()
            }
        }

        impl Neg for $self {
            type Output = Self;
            fn neg(self) -> Self::Output {
                Self::from_native(-self.to_native())
            }
        }
    };
}

macro_rules! derive_int_unsigned {
    ($self:ty, $native:ty, $from_bytes:ident, $to_bytes:ident$(,)?) => {
        derive_int!($self, $native, $from_bytes, $to_bytes);

        impl Unsigned for $self {}
    };
}

macro_rules! derive_le_int_signed {
    ($self:ty, $native:ty $(,)?) => {
        derive_int_signed!($self, $native, from_le_bytes, to_le_bytes);
    };
}

macro_rules! derive_le_int_unsigned {
    ($self:ty, $native:ty $(,)?) => {
        derive_int_unsigned!($self, $native, from_le_bytes, to_le_bytes);
    };
}

macro_rules! derive_be_int_signed {
    ($self:ty, $native:ty $(,)?) => {
        derive_int_signed!($self, $native, from_be_bytes, to_be_bytes);
    };
}

macro_rules! derive_be_int_unsigned {
    ($self:ty, $native:ty $(,)?) => {
        derive_int_unsigned!($self, $native, from_be_bytes, to_be_bytes);
    };
}

derive_le_int_unsigned!(Int<false, 2, false>, u16);
derive_le_int_unsigned!(Int<false, 4, false>, u32);
derive_le_int_unsigned!(Int<false, 8, false>, u64);
derive_le_int_signed!(Int<false, 2, true>, i16);
derive_le_int_signed!(Int<false, 4, true>, i32);
derive_le_int_signed!(Int<false, 8, true>, i64);

derive_be_int_unsigned!(Int<true, 2, false>, u16);
derive_be_int_unsigned!(Int<true, 4, false>, u32);
derive_be_int_unsigned!(Int<true, 8, false>, u64);
derive_be_int_signed!(Int<true, 2, true>, i16);
derive_be_int_signed!(Int<true, 4, true>, i32);
derive_be_int_signed!(Int<true, 8, true>, i64);

unsafe impl Portable for u8 {}
unsafe impl Portable for i8 {}

pub mod le {
    pub type Int<const N: usize, const S: bool> = super::Int<false, N, S>;

    pub type U16 = Int<2, false>;
    pub type U32 = Int<4, false>;
    pub type U64 = Int<8, false>;
    pub type I16 = Int<2, true>;
    pub type I32 = Int<4, true>;
    pub type I64 = Int<8, true>;
}

pub mod be {
    pub type Int<const N: usize, const S: bool> = super::Int<true, N, S>;

    pub type U16 = Int<2, false>;
    pub type U32 = Int<4, false>;
    pub type U64 = Int<8, false>;
    pub type I16 = Int<2, true>;
    pub type I32 = Int<4, true>;
    pub type I64 = Int<8, true>;
}
