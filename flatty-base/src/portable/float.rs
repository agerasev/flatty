use super::NativeCast;
use crate::{Error, Flat, FlatCast, Portable};
use core::{
    cmp::{Ordering, PartialOrd},
    ops::{Add, Div, Mul, Neg, Rem, Sub},
};
use num_traits::{FromPrimitive, Num, NumCast, One, ToPrimitive, Zero};

/// Generic portable floating-point number. Has alignment == 1.
///
/// Parameters:
/// + `BE`: Endianness. `false` => little-endian, `true` => big-endian.
/// + `N`: Width in bytes.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Float<const BE: bool, const N: usize> {
    bytes: [u8; N],
}

impl<const BE: bool, const N: usize> Default for Float<BE, N> {
    fn default() -> Self {
        Self { bytes: [0; N] }
    }
}

impl<const BE: bool, const N: usize> Float<BE, N> {
    pub fn from_bytes(bytes: [u8; N]) -> Self {
        Self { bytes }
    }
    pub fn to_bytes(self) -> [u8; N] {
        self.bytes
    }
}

impl<const BE: bool, const N: usize> FlatCast for Float<BE, N> {
    unsafe fn validate_contents(_: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}

unsafe impl<const BE: bool, const N: usize> Portable for Float<BE, N> {}

unsafe impl<const BE: bool, const N: usize> Flat for Float<BE, N> {}

macro_rules! derive_float {
    ($self:ty, $native:ty, $from_bytes:ident, $to_bytes:ident$(,)?) => {
        impl NativeCast for $self {
            type Native = $native;
            fn from_native(n: $native) -> Self {
                Float::from_bytes(n.$to_bytes())
            }
            fn to_native(&self) -> Self::Native {
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
                Some(Float::from_native(<$native>::from_u64(n)?))
            }
            fn from_i64(n: i64) -> Option<Self> {
                Some(Float::from_native(<$native>::from_i64(n)?))
            }
        }

        impl Num for $self {
            type FromStrRadixErr = <$native as Num>::FromStrRadixErr;
            fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
                Ok(Self::from_native(<$native as Num>::from_str_radix(
                    str, radix,
                )?))
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
        impl Neg for $self {
            type Output = Self;
            fn neg(self) -> Self::Output {
                Self::from_native(-self.to_native())
            }
        }

        impl PartialOrd for $self {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.to_native().partial_cmp(&other.to_native())
            }
        }
    };
}

macro_rules! derive_le_float {
    ($self:ty, $native:ty $(,)?) => {
        derive_float!($self, $native, from_le_bytes, to_le_bytes);
    };
}

macro_rules! derive_be_float {
    ($self:ty, $native:ty $(,)?) => {
        derive_float!($self, $native, from_be_bytes, to_be_bytes);
    };
}

derive_le_float!(Float<false, 4>, f32);
derive_le_float!(Float<false, 8>, f64);

derive_be_float!(Float<true, 4>, f32);
derive_be_float!(Float<true, 8>, f64);

pub mod le {
    use super::Float;

    pub type F32 = Float<false, 4>;
    pub type F64 = Float<false, 8>;
}

pub mod be {
    use super::Float;

    pub type F32 = Float<true, 4>;
    pub type F64 = Float<true, 8>;
}
