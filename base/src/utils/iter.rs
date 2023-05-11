use crate::{
    error::{Error, ErrorKind},
    traits::*,
    utils::{ceil_mul, max},
};
use core::marker::PhantomData;

pub trait TypeIter {
    type Item: Flat + ?Sized;
    fn align(&self) -> usize;
    fn min_size(&self, pos: usize) -> usize;

    fn check_align_and_min_size(&self, data: &[u8]) -> Result<(), Error> {
        if data.as_ptr().align_offset(self.align()) != 0 {
            Err(Error {
                kind: ErrorKind::BadAlign,
                pos: 0,
            })
        } else if data.len() < self.min_size(0) {
            Err(Error {
                kind: ErrorKind::InsufficientSize,
                pos: 0,
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug)]
pub struct SingleType<T: Flat + ?Sized> {
    _phantom: PhantomData<T>,
}
impl<T: Flat + ?Sized> SingleType<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}
impl<T: Flat + ?Sized> TypeIter for SingleType<T> {
    type Item = T;
    fn align(&self) -> usize {
        T::ALIGN
    }
    fn min_size(&self, pos: usize) -> usize {
        ceil_mul(pos, T::ALIGN) + T::MIN_SIZE
    }
}

#[derive(Clone, Debug)]
pub struct TwoOrMoreTypes<T: Flat + Sized, I: TypeIter> {
    _phantom: PhantomData<T>,
    next: I,
}
impl<T: Flat + Sized, I: TypeIter> TwoOrMoreTypes<T, I> {
    pub fn new(next: I) -> Self {
        Self {
            _phantom: PhantomData,
            next,
        }
    }
    pub fn next(self) -> I {
        self.next
    }
}
impl<T: Flat + Sized, I: TypeIter> TypeIter for TwoOrMoreTypes<T, I> {
    type Item = T;
    fn align(&self) -> usize {
        max(T::ALIGN, self.next.align())
    }
    fn min_size(&self, pos: usize) -> usize {
        self.next.min_size(ceil_mul(pos, T::ALIGN) + T::SIZE)
    }
}

#[derive(Clone, Debug)]
pub struct PosIter<I: TypeIter> {
    pos: usize,
    iter: I,
}
impl<I: TypeIter> PosIter<I> {
    pub fn new(iter: I) -> Self {
        Self { pos: 0, iter }
    }
    pub fn pos(&self) -> usize {
        self.pos
    }
}
impl<T: Flat + Sized, I: TypeIter> PosIter<TwoOrMoreTypes<T, I>> {
    pub fn next(self) -> PosIter<I> {
        PosIter {
            pos: ceil_mul(self.pos + T::SIZE, I::Item::ALIGN),
            iter: self.iter.next(),
        }
    }
}
impl<T: Flat + ?Sized> PosIter<SingleType<T>> {
    pub fn assert_last(&self) {
        // Nothing to do here, this is a static assert.
    }
}

#[derive(Clone, Debug)]
pub struct DataIter<'a, I: TypeIter> {
    data: &'a [u8],
    iter: PosIter<I>,
}
impl<'a, I: TypeIter> DataIter<'a, I> {
    pub fn new(data: &'a [u8], iter: I) -> Result<Self, Error> {
        iter.check_align_and_min_size(data)?;
        Ok(unsafe { Self::new_unchecked(data, iter) })
    }
    /// # Safety
    ///
    /// `data` must be aligned and have sufficient size.
    pub unsafe fn new_unchecked(data: &'a [u8], iter: I) -> Self {
        Self {
            data,
            iter: PosIter::new(iter),
        }
    }
    pub fn pos(&self) -> usize {
        self.iter.pos()
    }
    pub fn value(&self) -> &[u8] {
        self.data
    }
}
impl<'a, T: Flat + Sized, I: TypeIter> DataIter<'a, TwoOrMoreTypes<T, I>> {
    pub fn next(self) -> (DataIter<'a, I>, &'a [u8]) {
        let prev_pos = self.iter.pos();
        let iter = self.iter.next();
        let next_pos = iter.pos();
        let (prev_data, next_data) = self.data.split_at(next_pos - prev_pos);
        (DataIter { data: next_data, iter }, prev_data)
    }
}
impl<'a, T: Flat + ?Sized> DataIter<'a, SingleType<T>> {
    pub fn assert_last(&self) {
        self.iter.assert_last()
    }
    pub fn finalize(self) -> &'a [u8] {
        self.data
    }
}

pub trait ValidateIter {
    fn validate_all(self) -> Result<(), Error>;
}
impl<'a, T: Flat + Sized + 'a, I: TypeIter> ValidateIter for DataIter<'a, TwoOrMoreTypes<T, I>>
where
    DataIter<'a, I>: ValidateIter,
{
    fn validate_all(self) -> Result<(), Error> {
        unsafe { T::validate_unchecked(self.value()) }.map_err(|e| e.offset(self.pos()))?;
        self.next().0.validate_all()
    }
}
impl<'a, T: Flat + ?Sized> ValidateIter for DataIter<'a, SingleType<T>> {
    fn validate_all(self) -> Result<(), Error> {
        self.assert_last();
        unsafe { T::validate_unchecked(self.value()) }.map_err(|e| e.offset(self.pos()))?;
        Ok(())
    }
}

pub trait FoldSizeIter {
    /// # Safety
    ///
    /// Internal data must be valid.
    unsafe fn fold_size(self, size: usize) -> usize;
}
impl<'a, T: Flat + Sized + 'a, I: TypeIter> FoldSizeIter for DataIter<'a, TwoOrMoreTypes<T, I>>
where
    DataIter<'a, I>: FoldSizeIter,
{
    unsafe fn fold_size(self, size: usize) -> usize {
        self.next().0.fold_size(ceil_mul(size, T::ALIGN) + T::SIZE)
    }
}
impl<'a, T: Flat + ?Sized> FoldSizeIter for DataIter<'a, SingleType<T>> {
    unsafe fn fold_size(self, size: usize) -> usize {
        ceil_mul(size, T::ALIGN) + (&*T::ptr_from_bytes(self.finalize())).size()
    }
}

pub mod prelude {
    pub use super::{FoldSizeIter, TypeIter, ValidateIter};
}

#[doc(hidden)]
#[macro_export]
macro_rules! type_list {
    ($first_type:ty, $($types:ty),+ $(,)?) => {
        $crate::utils::iter::TwoOrMoreTypes::<$first_type, _>::new($crate::utils::iter::type_list!($( $types ),*))
    };
    ($type:ty $(,)?) => {
        $crate::utils::iter::SingleType::<$type>::new()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! fold_size {
    ($accum:expr; $first_type:ty, $($types:ty),+ $(,)?) => {
        $crate::utils::iter::fold_size!(
            $crate::utils::ceil_mul($accum, <$first_type as $crate::traits::FlatBase>::ALIGN) + <$first_type as $crate::traits::FlatSized>::SIZE;
            $( $types ),*
        )
    };
    ($accum:expr; $type:ty $(,)?) => {
        $crate::utils::ceil_mul($accum, <$type as $crate::traits::FlatBase>::ALIGN) + <$type as $crate::traits::FlatSized>::SIZE
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! fold_min_size {
    ($accum:expr; $first_type:ty, $($types:ty),+ $(,)?) => {
        $crate::utils::iter::fold_min_size!(
            $crate::utils::ceil_mul($accum, <$first_type as $crate::traits::FlatBase>::ALIGN) + <$first_type as $crate::traits::FlatSized>::SIZE;
            $( $types ),*
        )
    };
    ($accum:expr; $type:ty $(,)?) => {
        $crate::utils::ceil_mul($accum, <$type as $crate::traits::FlatBase>::ALIGN) + <$type as $crate::traits::FlatBase>::MIN_SIZE
    };
}

pub use {fold_min_size, fold_size, type_list};

#[cfg(test)]
mod tests {
    use super::{type_list, PosIter};

    #[test]
    fn pos() {
        let iter = PosIter::new(type_list!(u8, u16, u32));
        assert_eq!(iter.pos(), 0);
        let iter = iter.next();
        assert_eq!(iter.pos(), 2);
        let iter = iter.next();
        assert_eq!(iter.pos(), 4);
        iter.assert_last();
    }
}
