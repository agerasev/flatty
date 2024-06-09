use core::{
    ops::{Deref, DerefMut},
    ptr,
    str::from_utf8,
};
use flatty_base::{
    emplacer::Emplacer,
    error::{Error, ErrorKind},
    traits::{Flat, FlatBase, FlatDefault, FlatSized, FlatUnsized, FlatValidate},
    utils::{floor_mul, mem::slice_ptr_len},
};
use stavec::GenericString;

pub use stavec::traits::{Length, Slot};

/// Growable flat vector of sized items.
///
/// It doesn't allocate memory on the heap but instead stores its contents in the same memory behind itself.
///
/// Obviously, this type is DST.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct FlatString<L: Flat + Length = usize>(GenericString<[u8], L>);

impl<L: Flat + Length> Deref for FlatString<L> {
    type Target = GenericString<[u8], L>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<L: Flat + Length> DerefMut for FlatString<L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

trait DataOffset<L: Flat + Length> {
    const DATA_OFFSET: usize = L::SIZE;
}

impl<L: Flat + Length> DataOffset<L> for FlatString<L> {}

unsafe impl<L: Flat + Length> FlatBase for FlatString<L> {
    const ALIGN: usize = L::ALIGN;
    const MIN_SIZE: usize = Self::DATA_OFFSET;

    fn size(&self) -> usize {
        Self::DATA_OFFSET + self.len()
    }
}

unsafe impl<L: Flat + Length> FlatUnsized for FlatString<L> {
    type AlignAs = L;

    unsafe fn ptr_from_bytes(bytes: *mut [u8]) -> *mut Self {
        let meta = floor_mul(slice_ptr_len(bytes) - Self::DATA_OFFSET, Self::ALIGN);
        ptr::slice_from_raw_parts_mut(bytes as *mut u8, meta) as *mut Self
    }
    unsafe fn ptr_to_bytes(this: *mut Self) -> *mut [u8] {
        let len = Self::DATA_OFFSET + slice_ptr_len(this as *mut [u8]);
        ptr::slice_from_raw_parts_mut(this as *mut u8, len)
    }
}

pub struct Empty;
pub struct FromStr<S: AsRef<str>>(pub S);

unsafe impl<L: Flat + Length> Emplacer<FlatString<L>> for Empty {
    unsafe fn emplace_unchecked(self, bytes: &mut [u8]) -> Result<&mut FlatString<L>, Error> {
        unsafe { (bytes.as_mut_ptr() as *mut L).write(L::zero()) };
        Ok(unsafe { FlatString::from_mut_bytes_unchecked(bytes) })
    }
}

unsafe impl<L: Flat + Length, S: AsRef<str>> Emplacer<FlatString<L>> for FromStr<S> {
    unsafe fn emplace_unchecked(self, bytes: &mut [u8]) -> Result<&mut FlatString<L>, Error> {
        unsafe { <Empty as Emplacer<FlatString<L>>>::emplace_unchecked(Empty, bytes) }?;
        let vec = unsafe { FlatString::<L>::from_mut_bytes_unchecked(bytes) };
        vec.push_str(self.0.as_ref()).map_err(|_| Error {
            kind: ErrorKind::InsufficientSize,
            pos: 0,
        })?;
        Ok(vec)
    }
}

impl<L: Flat + Length> FlatDefault for FlatString<L> {
    type DefaultEmplacer = Empty;

    fn default_emplacer() -> Empty {
        Empty
    }
}

unsafe impl<L: Flat + Length> FlatValidate for FlatString<L> {
    unsafe fn validate_unchecked(bytes: &[u8]) -> Result<(), Error> {
        unsafe { L::validate_unchecked(bytes) }?;
        let this = unsafe { Self::from_bytes_unchecked(bytes) };
        if this.len() > this.capacity() {
            return Err(Error {
                kind: ErrorKind::InsufficientSize,
                pos: Self::DATA_OFFSET,
            });
        }
        match from_utf8(unsafe { this.0.as_vec().data().as_ref().get_unchecked(..this.len()) }) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error {
                kind: ErrorKind::InvalidData,
                pos: Self::DATA_OFFSET + e.valid_up_to(),
            }),
        }
    }
}

unsafe impl<L: Flat + Length> Flat for FlatString<L> {}

/// Creates [`FlatString`] emplacer from given array.
#[macro_export]
macro_rules! flat_string {
    () => {
        $crate::string::FromStr("")
    };
    ($s:literal) => {
        $crate::string::FromStr($s)
    };
}
pub use flat_string;

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::bytes::AlignedBytes;

    #[test]
    fn push_str() {
        let mut bytes = AlignedBytes::new(4 + 8, 4);
        let string = FlatString::<u32>::default_in_place(&mut bytes).unwrap();
        assert_eq!(string.capacity(), 8);
        assert_eq!(string.len(), 0);
        assert_eq!(string.remaining(), 8);

        string.push_str("abc").unwrap();
        assert_eq!(string.len(), 3);
        assert_eq!(string.remaining(), 5);
        assert_eq!(string.as_str(), "abc");

        string.push_str("defgh").unwrap();
        assert_eq!(string.len(), 8);
        assert_eq!(string.remaining(), 0);
        assert_eq!(string.as_str(), "abcdefgh");

        assert!(string.push_str("i").is_err());
    }

    #[test]
    fn eq() {
        let mut mem_a = AlignedBytes::new(2 + 4, 2);
        let string_a = FlatString::<u16>::new_in_place(&mut mem_a, flat_string!("abcd")).unwrap();

        let mut mem_b = AlignedBytes::new(2 + 4, 2);
        let string_b = FlatString::<u16>::new_in_place(&mut mem_b, flat_string!("abcd")).unwrap();

        let mut mem_c = AlignedBytes::new(2 + 2, 2);
        let string_c = FlatString::<u16>::new_in_place(&mut mem_c, flat_string!("ab")).unwrap();

        assert_eq!(string_a, string_b);
        assert_ne!(string_a, string_c);
        assert_ne!(string_b, string_c);

        string_b.clear();
        assert_ne!(string_a, string_b);
    }
}
