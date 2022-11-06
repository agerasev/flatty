use flatty::{self, mem::MaybeUninitUnsized, prelude::*, Emplacer};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub trait AbstractWriter<M: Portable + ?Sized>: Sized {
    fn buffer(&self) -> &[u8];
    fn buffer_mut(&mut self) -> &mut [u8];

    fn new_message(&mut self) -> UninitWriteGuard<M, Self> {
        UninitWriteGuard::new(self)
    }
}

pub struct UninitWriteGuard<'a, M: Portable + ?Sized, O: AbstractWriter<M>> {
    pub(crate) owner: &'a mut O,
    _phantom: PhantomData<M>,
}

pub struct WriteGuard<'a, M: Portable + ?Sized, O: AbstractWriter<M>> {
    pub(crate) owner: &'a mut O,
    _phantom: PhantomData<M>,
}

impl<'a, M: Portable + ?Sized, O: AbstractWriter<M>> UninitWriteGuard<'a, M, O> {
    pub fn new(owner: &'a mut O) -> Self {
        Self {
            owner,
            _phantom: PhantomData,
        }
    }

    /// # Safety
    ///
    /// Underlying message data must be initialized.
    pub unsafe fn assume_init(self) -> WriteGuard<'a, M, O> {
        WriteGuard {
            owner: self.owner,
            _phantom: PhantomData,
        }
    }

    pub fn emplace(mut self, emplacer: impl Emplacer<M>) -> Result<WriteGuard<'a, M, O>, flatty::Error> {
        self.deref_mut().new_in_place(emplacer)?;
        Ok(unsafe { self.assume_init() })
    }
}

impl<'a, M: Portable + FlatDefault + ?Sized, O: AbstractWriter<M>> UninitWriteGuard<'a, M, O> {
    pub fn default(self) -> Result<WriteGuard<'a, M, O>, flatty::Error> {
        M::from_mut_bytes(self.owner.buffer_mut())?.default_in_place()?;
        Ok(unsafe { self.assume_init() })
    }
}

impl<'a, M: Portable + ?Sized, O: AbstractWriter<M> + Unpin> Unpin for UninitWriteGuard<'a, M, O> {}
impl<'a, M: Portable + ?Sized, O: AbstractWriter<M> + Unpin> Unpin for WriteGuard<'a, M, O> {}

impl<'a, M: Portable + ?Sized, O: AbstractWriter<M>> Deref for UninitWriteGuard<'a, M, O> {
    type Target = MaybeUninitUnsized<M>;
    fn deref(&self) -> &Self::Target {
        unsafe { MaybeUninitUnsized::from_bytes_unchecked(self.owner.buffer()) }
    }
}

impl<'a, M: Portable + ?Sized, O: AbstractWriter<M>> DerefMut for UninitWriteGuard<'a, M, O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { MaybeUninitUnsized::from_mut_bytes_unchecked(self.owner.buffer_mut()) }
    }
}

impl<'a, M: Portable + ?Sized, O: AbstractWriter<M>> Deref for WriteGuard<'a, M, O> {
    type Target = M;
    fn deref(&self) -> &M {
        unsafe { MaybeUninitUnsized::from_bytes_unchecked(self.owner.buffer()).assume_init() }
    }
}

impl<'a, M: Portable + ?Sized, O: AbstractWriter<M>> DerefMut for WriteGuard<'a, M, O> {
    fn deref_mut(&mut self) -> &mut M {
        unsafe { MaybeUninitUnsized::from_mut_bytes_unchecked(self.owner.buffer_mut()).assume_init_mut() }
    }
}
