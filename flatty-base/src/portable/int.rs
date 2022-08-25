use super::NativeCast;
use crate::{Error, Flat, FlatInit, Portable};
use core::{
    mem,
    ops::{Add, Div, Mul, Neg, Rem, Sub},
};
use num_traits::{FromPrimitive, Num, One, Signed, ToPrimitive, Unsigned, Zero};

/// Generic portable integer. Has alignment == 1.
///
/// Parameters:
/// + `BE`: Endianness. `false` => little-endian, `true` => big-endian.
/// + `N`: Width in bytes.
/// + `S`: Signedness. `false` => unsigned, `true` => signed.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Integer<const BE: bool, const N: usize, const S: bool> {
    bytes: [u8; N],
}

impl<const BE: bool, const N: usize, const S: bool> Default for Integer<BE, N, S> {
    fn default() -> Self {
        Self { bytes: [0; N] }
    }
}

impl<const BE: bool, const N: usize, const S: bool> Integer<BE, N, S> {
    pub fn from_bytes(bytes: [u8; N]) -> Self {
        Self { bytes }
    }
    pub fn to_bytes(&self) -> [u8; N] {
        self.bytes
    }
}
impl<const BE: bool, const N: usize, const S: bool> From<[u8; N]> for Integer<BE, N, S> {
    fn from(bytes: [u8; N]) -> Self {
        Self::from_bytes(bytes)
    }
}
impl<const BE: bool, const N: usize, const S: bool> Into<[u8; N]> for Integer<BE, N, S> {
    fn into(self) -> [u8; N] {
        self.to_bytes()
    }
}

macro_rules! derive_int {
    ($self:ty, $native:ty, $from_bytes:ident, $to_bytes:ident$(,)?) => {
        impl NativeCast for $self {
            type Native = $native;
            fn from_native(n: $native) -> Self {
                Integer::from_bytes(n.$to_bytes())
            }
            fn to_native(&self) -> Self::Native {
                <$native>::$from_bytes(self.to_bytes())
            }
        }

        impl FlatInit for $self {
            type Dyn = $native;
            fn size_of(_: &$native) -> usize {
                mem::size_of::<Self>()
            }

            unsafe fn placement_new_unchecked<'a, 'b>(
                mem: &'a mut [u8],
                init: &'b Self::Dyn,
            ) -> &'a mut Self {
                let self_ = Self::reinterpret_mut_unchecked(mem);
                *self_ = Self::from_native(*init);
                self_
            }

            fn pre_validate(_: &[u8]) -> Result<(), Error> {
                Ok(())
            }
            fn post_validate(&self) -> Result<(), Error> {
                Ok(())
            }

            unsafe fn reinterpret_unchecked(mem: &[u8]) -> &Self {
                &*(mem.as_ptr() as *const Self)
            }
            unsafe fn reinterpret_mut_unchecked(mem: &mut [u8]) -> &mut Self {
                &mut *(mem.as_mut_ptr() as *mut Self)
            }
        }

        unsafe impl Portable for $self {}

        unsafe impl Flat for $self {}

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
                Some(Integer::from_native(<$native>::from_u64(n)?))
            }
            fn from_i64(n: i64) -> Option<Self> {
                Some(Integer::from_native(<$native>::from_i64(n)?))
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

derive_le_int_unsigned!(Integer<false, 2, false>, u16);
derive_le_int_unsigned!(Integer<false, 4, false>, u32);
derive_le_int_unsigned!(Integer<false, 8, false>, u64);
derive_le_int_signed!(Integer<false, 2, true>, i16);
derive_le_int_signed!(Integer<false, 4, true>, i32);
derive_le_int_signed!(Integer<false, 8, true>, i64);

derive_be_int_unsigned!(Integer<true, 2, false>, u16);
derive_be_int_unsigned!(Integer<true, 4, false>, u32);
derive_be_int_unsigned!(Integer<true, 8, false>, u64);
derive_be_int_signed!(Integer<true, 2, true>, i16);
derive_be_int_signed!(Integer<true, 4, true>, i32);
derive_be_int_signed!(Integer<true, 8, true>, i64);

/// Little-endian types.
#[allow(non_camel_case_types)]
pub mod le {
    use super::Integer;

    pub type u16 = Integer<false, 2, false>;
    pub type u32 = Integer<false, 4, false>;
    pub type u64 = Integer<false, 8, false>;
    pub type i16 = Integer<false, 2, true>;
    pub type i32 = Integer<false, 4, true>;
    pub type i64 = Integer<false, 8, true>;
}

/// Big-endian types.
#[allow(non_camel_case_types)]
pub mod be {
    use super::Integer;

    pub type u16 = Integer<true, 2, false>;
    pub type u32 = Integer<true, 4, false>;
    pub type u64 = Integer<true, 8, false>;
    pub type i16 = Integer<true, 2, true>;
    pub type i32 = Integer<true, 4, true>;
    pub type i64 = Integer<true, 8, true>;
}
