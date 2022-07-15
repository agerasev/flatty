mod prim;
mod size;
mod vec;

pub use prim::*;
pub use size::*;
pub use vec::*;

/// # Safety
pub unsafe trait Flat {}

pub trait FlatExt {
    fn align_offset(ptr: *const u8) -> usize;

    fn from_slice(mem: &[u8]) -> &Self;
    fn from_slice_mut(mem: &mut [u8]) -> &mut Self;
}
