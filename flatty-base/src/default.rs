/// Create a new instance of `Self` onto raw memory from dynamic representation of the type.
fn placement_new(bytes: &'a mut [u8]) -> Result<&'a mut Self, Error> {
    Self::check_size_and_align(mem)?;
    Ok(unsafe { Self::placement_new_unchecked(mem, init) })
}
/// Create a new default instance of `Self` onto raw memory.
fn placement_default(mem: &mut [u8]) -> Result<&mut Self, Error>
where
    Self::Dyn: Default,
{
    Self::placement_new(mem, &Self::Dyn::default())
}
/// Initialize without checks.
///
/// # Safety
///
/// Size, alignment and specific type requirements must be ok.
unsafe fn placement_new_unchecked<'a, 'b>(mem: &'a mut [u8], init: &'b Self::Dyn) -> &'a mut Self;
