use crate::{util::const_max, Flat, FlatExt, FlatSized, FlatVec};

#[repr(u8)]
pub enum UnsizedEnumState {
    A,
    B,
    C,
}

#[repr(C)]
pub struct UnsizedEnum {
    id: UnsizedEnumState,
    _align: [<Self as Flat>::AlignAs; 0],
    data: [u8],
}

pub enum UnsizedEnumRef<'a> {
    A,
    B(&'a i32),
    C(&'a FlatVec<u8>),
}

pub enum UnsizedEnumMut<'a> {
    A,
    B(&'a mut i32),
    C(&'a mut FlatVec<u8>),
}

impl UnsizedEnum {
    const DATA_OFFSET: usize = const_max(u8::SIZE, Self::ALIGN);

    pub fn as_ref(&self) -> UnsizedEnumRef<'_> {
        match self.id {
            UnsizedEnumState::A => UnsizedEnumRef::A,
            UnsizedEnumState::B => UnsizedEnumRef::B(unsafe {
                i32::from_slice(&self.data[self.data.as_ptr().align_offset(i32::ALIGN)..])
            }),
            UnsizedEnumState::C => UnsizedEnumRef::C(unsafe {
                FlatVec::<u8>::from_slice(
                    &self.data[self.data.as_ptr().align_offset(FlatVec::<u8>::ALIGN)..],
                )
            }),
        }
    }
}

#[repr(u8)]
pub enum UnsizedEnumAlignAs {
    A,
    B(<i32 as Flat>::AlignAs),
    C(<FlatVec<u8> as Flat>::AlignAs),
}

unsafe impl Flat for UnsizedEnum {
    type AlignAs = UnsizedEnumAlignAs;

    fn size(&self) -> usize {
        Self::DATA_OFFSET
            + match self.as_ref() {
                UnsizedEnumRef::A => 0,
                UnsizedEnumRef::B(r) => r.size(),
                UnsizedEnumRef::C(r) => r.size(),
            }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
