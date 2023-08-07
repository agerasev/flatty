use flatty::{self, prelude::*, Emplacer};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub trait CommonWriter<M: Flat + ?Sized>: Sized {
    fn buffer(&self) -> &[u8];
    fn buffer_mut(&mut self) -> &mut [u8];
}

pub struct CommonUninitWriteGuard<'a, M: Flat + ?Sized, O: CommonWriter<M>> {
    owner: &'a mut O,
    _phantom: PhantomData<M>,
}

impl<'a, M: Flat + ?Sized, O: CommonWriter<M>> CommonUninitWriteGuard<'a, M, O> {
    pub fn new(owner: &'a mut O) -> Self {
        Self {
            owner,
            _phantom: PhantomData,
        }
    }

    pub fn buffer(&self) -> &[u8] {
        self.owner.buffer()
    }
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        self.owner.buffer_mut()
    }

    /// # Safety
    ///
    /// Underlying message data must be initialized.
    pub unsafe fn assume_valid(self) -> CommonWriteGuard<'a, M, O> {
        CommonWriteGuard {
            owner: self.owner,
            _phantom: PhantomData,
        }
    }

    pub fn new_in_place(self, emplacer: impl Emplacer<M>) -> Result<CommonWriteGuard<'a, M, O>, flatty::Error> {
        M::new_in_place(self.owner.buffer_mut(), emplacer)?;
        Ok(unsafe { self.assume_valid() })
    }
}

impl<'a, M: Flat + FlatDefault + ?Sized, O: CommonWriter<M>> CommonUninitWriteGuard<'a, M, O> {
    pub fn default_in_place(self) -> Result<CommonWriteGuard<'a, M, O>, flatty::Error> {
        M::default_in_place(self.owner.buffer_mut())?;
        Ok(unsafe { self.assume_valid() })
    }
}

impl<'a, M: Flat + ?Sized, O: CommonWriter<M> + Unpin> Unpin for CommonUninitWriteGuard<'a, M, O> {}

pub struct CommonWriteGuard<'a, M: Flat + ?Sized, O: CommonWriter<M>> {
    pub(crate) owner: &'a mut O,
    _phantom: PhantomData<M>,
}

impl<'a, M: Flat + ?Sized, O: CommonWriter<M> + Unpin> Unpin for CommonWriteGuard<'a, M, O> {}

impl<'a, M: Flat + ?Sized, O: CommonWriter<M>> Deref for CommonWriteGuard<'a, M, O> {
    type Target = M;
    fn deref(&self) -> &M {
        unsafe { M::from_bytes_unchecked(self.owner.buffer()) }
    }
}

impl<'a, M: Flat + ?Sized, O: CommonWriter<M>> DerefMut for CommonWriteGuard<'a, M, O> {
    fn deref_mut(&mut self) -> &mut M {
        unsafe { M::from_mut_bytes_unchecked(self.owner.buffer_mut()) }
    }
}
