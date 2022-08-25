mod float;
mod int;

use crate::Flat;
use num_traits::{FromPrimitive, Num, ToPrimitive};

/// Type that can be safely transefered between different machines.
pub unsafe trait Portable: Flat {}

pub trait NativeCast: Num + FromPrimitive + ToPrimitive + Copy {
    type Native: Num + FromPrimitive + ToPrimitive + Copy;
    fn from_native(n: Self::Native) -> Self;
    fn to_native(&self) -> Self::Native;
}

pub use int::{be, le, Integer};
