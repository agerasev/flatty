mod float;
mod impl_;
mod int;
#[cfg(test)]
mod tests;

use flatty_base::Flat;
use num_traits::{FromPrimitive, Num, ToPrimitive};

/// Type that can be safely transefered between different machines.
///
/// # Safety
///
/// Implementing this trait must guarantee that `Self` has the same binary representation on any target platform this crate could be built for.
pub unsafe trait Portable: Flat {}

/// Trait for casting portable type to/from native counterparts.
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

pub mod prelude {
    pub use super::{NativeCast, Portable};
}

macro_rules! derive_display {
    ($self:ty, $native:ty) => {
        impl core::fmt::Debug for $self {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                <$native as core::fmt::Debug>::fmt(&self.to_native(), f)
            }
        }
        impl core::fmt::Display for $self {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                <$native as core::fmt::Display>::fmt(&self.to_native(), f)
            }
        }
    };
}

pub(crate) use derive_display;
