use crate::mem::MaybeUninitUnsized;

/// Basic flat type properties.
///
/// # Safety
///
/// `ALIGN` and `MIN_SIZE` and `size` must match the ones of the `Self`.
pub unsafe trait FlatBase {
    /// Align of the type.
    const ALIGN: usize;
    /// Minimal size of an instance of the type.
    const MIN_SIZE: usize;

    /// Size of an instance of the type.
    fn size(&self) -> usize;
}

/// Dynamically-sized flat type. Like `?Sized` but for `Flat`.
///
/// For now it must be implemented for all [`Flat`](`crate::Flat`) types because there is no mutually exclusive traits in Rust yet.
///
/// # Safety
///
/// `AlignAs` type must have the same align as `Self`.
///
/// `ptr_metadata` must provide such value, that will give [`size()`](`FlatBase::size`)` <= `[`size_of_val()`](`core::mem::size_of_val`)` <= bytes.len()`.
pub unsafe trait FlatMaybeUnsized: FlatBase {
    /// Sized type that has the same alignment as `Self`.
    type AlignAs: Sized;

    /// Metadata to store in a wide pointer to `Self`.
    ///
    /// Panics if type is `Sized`.
    fn ptr_metadata(this: &MaybeUninitUnsized<Self>) -> usize;
    /// Length of underlying memory occupied by `this`.
    fn bytes_len(this: &Self) -> usize;

    /// # Safety
    ///
    /// `this` must be initialized.
    unsafe fn from_uninit_unchecked(this: &MaybeUninitUnsized<Self>) -> &Self;
    /// # Safety
    ///
    /// `this` must be initialized.
    unsafe fn from_mut_uninit_unchecked(this: &mut MaybeUninitUnsized<Self>) -> &mut Self;

    fn to_uninit(&self) -> &MaybeUninitUnsized<Self>;
    /// # Safety
    ///
    /// Modification of return value must not make `self` invalid.
    unsafe fn to_mut_uninit(&mut self) -> &mut MaybeUninitUnsized<Self>;
}

#[macro_export]
macro_rules! impl_unsized_uninit_cast {
    () => {
        unsafe fn from_uninit_unchecked(this: &MaybeUninitUnsized<Self>) -> &Self {
            let slice = ::core::ptr::slice_from_raw_parts(
                this.as_bytes().as_ptr(),
                Self::ptr_metadata(this),
            );
            &*(slice as *const [_] as *const Self)
        }
        unsafe fn from_mut_uninit_unchecked(this: &mut MaybeUninitUnsized<Self>) -> &mut Self {
            let slice = ::core::ptr::slice_from_raw_parts_mut(
                this.as_mut_bytes().as_mut_ptr(),
                Self::ptr_metadata(this),
            );
            &mut *(slice as *mut [_] as *mut Self)
        }

        fn to_uninit(&self) -> &MaybeUninitUnsized<Self> {
            unsafe {
                MaybeUninitUnsized::from_bytes_unchecked(::core::slice::from_raw_parts(
                    self as *const _ as *const u8,
                    Self::bytes_len(self),
                ))
            }
        }
        unsafe fn to_mut_uninit(&mut self) -> &mut MaybeUninitUnsized<Self> {
            MaybeUninitUnsized::from_mut_bytes_unchecked(::core::slice::from_raw_parts_mut(
                self as *mut _ as *mut u8,
                Self::bytes_len(self),
            ))
        }
    };
}

pub use impl_unsized_uninit_cast;
