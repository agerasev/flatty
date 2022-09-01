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

    /// Make a pointer to `Self` from bytes without any checks.
    fn ptr_from_bytes(bytes: &[u8]) -> *const Self;

    /// Make a mutable pointer to `Self` from bytes without any checks.
    fn ptr_from_mut_bytes(bytes: &mut [u8]) -> *mut Self;
}
