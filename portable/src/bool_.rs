use crate::{NativeCast, Portable};
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};
use flatty_base::{
    error::{Error, ErrorKind},
    traits::{Flat, FlatValidate},
};

/// Boolean type that has portable binary representation.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Bool {
    #[default]
    False = 0,
    True = 1,
}

impl From<bool> for Bool {
    fn from(value: bool) -> Self {
        match value {
            true => Bool::True,
            false => Bool::False,
        }
    }
}

impl From<Bool> for bool {
    fn from(value: Bool) -> Self {
        match value {
            Bool::True => true,
            Bool::False => false,
        }
    }
}

unsafe impl FlatValidate for Bool {
    unsafe fn validate_unchecked(bytes: &[u8]) -> Result<(), Error> {
        match bytes.get_unchecked(0) {
            0..=1 => Ok(()),
            _ => Err(Error {
                kind: ErrorKind::InvalidData,
                pos: 0,
            }),
        }
    }
}

unsafe impl Flat for Bool {}

unsafe impl Portable for Bool {}

impl NativeCast for Bool {
    type Native = bool;

    fn from_native(n: bool) -> Self {
        n.into()
    }
    fn to_native(&self) -> bool {
        (*self).into()
    }
}

impl Not for Bool {
    type Output = Self;
    fn not(self) -> Self::Output {
        bool::from(self).not().into()
    }
}

impl BitAnd for Bool {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        bool::from(self).bitand(bool::from(rhs)).into()
    }
}

impl BitOr for Bool {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        bool::from(self).bitor(bool::from(rhs)).into()
    }
}

impl BitXor for Bool {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        bool::from(self).bitxor(bool::from(rhs)).into()
    }
}

impl BitAndAssign for Bool {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.bitand(rhs);
    }
}

impl BitOrAssign for Bool {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.bitor(rhs);
    }
}

impl BitXorAssign for Bool {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = self.bitxor(rhs);
    }
}
