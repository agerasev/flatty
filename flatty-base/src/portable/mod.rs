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

pub use float::Float;
pub use int::Int;

/// Little-endian types.
pub mod le {
    use super::*;
    pub use float::le::*;
    pub use int::le::*;
}

/// Big-endian types.
pub mod be {
    use super::*;
    pub use float::be::*;
    pub use int::be::*;
}

unsafe impl Portable for u8 {}
unsafe impl Portable for i8 {}
