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
