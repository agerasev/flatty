use flatty::{self, mem::MaybeUninitUnsized, prelude::*, Emplacer};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub trait CommonWriter<M: Portable + ?Sized>: Sized {
    fn buffer(&self) -> &[u8];
    fn buffer_mut(&mut self) -> &mut [u8];
}

pub struct CommonUninitWriteGuard<'a, M: Portable + ?Sized, O: CommonWriter<M>> {
    owner: &'a mut O,
    _phantom: PhantomData<M>,
}

impl<'a, M: Portable + ?Sized, O: CommonWriter<M>> CommonUninitWriteGuard<'a, M, O> {
    pub fn new(owner: &'a mut O) -> Self {
        Self {
            owner,
            _phantom: PhantomData,
        }
    }

    /// # Safety
    ///
    /// Underlying message data must be initialized.
    pub unsafe fn assume_init(self) -> CommonWriteGuard<'a, M, O> {
        CommonWriteGuard {
            owner: self.owner,
            _phantom: PhantomData,
        }
    }

    pub fn emplace(mut self, emplacer: impl Emplacer<M>) -> Result<CommonWriteGuard<'a, M, O>, flatty::Error> {
        self.deref_mut().new_in_place(emplacer)?;
        Ok(unsafe { self.assume_init() })
    }
}

impl<'a, M: Portable + FlatDefault + ?Sized, O: CommonWriter<M>> CommonUninitWriteGuard<'a, M, O> {
    pub fn default(self) -> Result<CommonWriteGuard<'a, M, O>, flatty::Error> {
        M::uninit_from_mut_bytes(self.owner.buffer_mut())?.default_in_place()?;
        Ok(unsafe { self.assume_init() })
    }
}

impl<'a, M: Portable + ?Sized, O: CommonWriter<M> + Unpin> Unpin for CommonUninitWriteGuard<'a, M, O> {}

impl<'a, M: Portable + ?Sized, O: CommonWriter<M>> Deref for CommonUninitWriteGuard<'a, M, O> {
    type Target = MaybeUninitUnsized<M>;
    fn deref(&self) -> &Self::Target {
        unsafe { MaybeUninitUnsized::from_bytes_unchecked(self.owner.buffer()) }
    }
}

impl<'a, M: Portable + ?Sized, O: CommonWriter<M>> DerefMut for CommonUninitWriteGuard<'a, M, O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { MaybeUninitUnsized::from_mut_bytes_unchecked(self.owner.buffer_mut()) }
    }
}

pub struct CommonWriteGuard<'a, M: Portable + ?Sized, O: CommonWriter<M>> {
    pub(crate) owner: &'a mut O,
    _phantom: PhantomData<M>,
}

impl<'a, M: Portable + ?Sized, O: CommonWriter<M> + Unpin> Unpin for CommonWriteGuard<'a, M, O> {}

impl<'a, M: Portable + ?Sized, O: CommonWriter<M>> Deref for CommonWriteGuard<'a, M, O> {
    type Target = M;
    fn deref(&self) -> &M {
        unsafe { MaybeUninitUnsized::from_bytes_unchecked(self.owner.buffer()).assume_init() }
    }
}

impl<'a, M: Portable + ?Sized, O: CommonWriter<M>> DerefMut for CommonWriteGuard<'a, M, O> {
    fn deref_mut(&mut self) -> &mut M {
        unsafe { MaybeUninitUnsized::from_mut_bytes_unchecked(self.owner.buffer_mut()).assume_init_mut() }
    }
}
