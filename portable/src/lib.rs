#![no_std]

mod bool_;
mod float;
mod impl_;
mod int;
#[cfg(test)]
mod tests;

use flatty_base::traits::Flat;

/// Type that can be safely transfered between different machines.
///
/// # Safety
///
/// Implementing this trait must guarantee that `Self` has the same binary representation on any target platform this crate could be built for.
pub unsafe trait Portable: Flat {}

unsafe impl Portable for () {}

pub use bool_::Bool;
pub use float::Float;
pub use int::Int;

/// Little-endian types.
pub mod le {
    pub use super::float::le::*;
    pub use super::int::le::*;
}

/// Big-endian types.
pub mod be {
    pub use super::float::be::*;
    pub use super::int::be::*;
}

pub mod traits {
    pub use super::Portable;
}

macro_rules! impl_traits_for_native {
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

        #[cfg(feature = "serde")]
        impl serde::Serialize for $self {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                <$native as serde::Serialize>::serialize(&self.to_native(), serializer)
            }
        }
        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for $self {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                <$native as serde::Deserialize<'de>>::deserialize(deserializer).map(<$self>::from_native)
            }
        }
    };
}

pub(crate) use impl_traits_for_native;
