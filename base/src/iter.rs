use crate::{mem::Muu, prelude::*, utils::ceil_mul, Error};
use core::marker::PhantomData;

pub trait TypeIter {
    type Item: Flat + ?Sized;
}

#[derive(Clone, Debug)]
pub struct SingleType<T: Flat + ?Sized> {
    _phantom: PhantomData<T>,
}
impl<T: Flat + ?Sized> SingleType<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}
impl<T: Flat + ?Sized> TypeIter for SingleType<T> {
    type Item = T;
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

#[derive(Clone, Debug)]
pub struct RefIter<'a, I: TypeIter> {
    data: &'a [u8],
    iter: PosIter<I>,
}
impl<'a, I: TypeIter> RefIter<'a, I> {
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

    pub fn value(&self) -> &Muu<I::Item> {
        unsafe { Muu::from_bytes_unchecked(self.data.get_unchecked(self.pos()..)) }
    }
}
impl<'a, T: Flat + Sized, I: TypeIter> RefIter<'a, TwoOrMoreTypes<T, I>> {
    pub fn next(self) -> RefIter<'a, I> {
        RefIter {
            data: self.data,
            iter: self.iter.next(),
        }
    }
}

#[derive(Debug)]
pub struct MutIter<'a, I: TypeIter> {
    data: &'a mut [u8],
    iter: PosIter<I>,
}
impl<'a, I: TypeIter> MutIter<'a, I> {
    /// # Safety
    ///
    /// `data` must be aligned and have sufficient size.
    pub unsafe fn new_unchecked(data: &'a mut [u8], iter: I) -> Self {
        Self {
            data,
            iter: PosIter::new(iter),
        }
    }
    pub fn pos(&self) -> usize {
        self.iter.pos()
    }

    pub fn value(&self) -> &Muu<I::Item> {
        unsafe { Muu::from_bytes_unchecked(self.data.get_unchecked(self.pos()..)) }
    }
    pub fn value_mut(&mut self) -> &mut Muu<I::Item> {
        let pos = self.pos();
        unsafe { Muu::from_mut_bytes_unchecked(self.data.get_unchecked_mut(pos..)) }
    }
}
impl<'a, T: Flat + Sized, I: TypeIter> MutIter<'a, TwoOrMoreTypes<T, I>> {
    pub fn next(self) -> MutIter<'a, I> {
        MutIter {
            data: self.data,
            iter: self.iter.next(),
        }
    }
}

pub trait ValidateIter {
    fn validate_all(self) -> Result<(), Error>;
}
impl<'a, T: Flat + Sized, I: TypeIter> ValidateIter for RefIter<'a, TwoOrMoreTypes<T, I>>
where
    RefIter<'a, I>: ValidateIter,
{
    fn validate_all(self) -> Result<(), Error> {
        T::validate(self.value()).map_err(|e| e.offset(self.pos()))?;
        self.next().validate_all()
    }
}
impl<'a, T: Flat + ?Sized> ValidateIter for RefIter<'a, SingleType<T>> {
    fn validate_all(self) -> Result<(), Error> {
        T::validate(self.value()).map_err(|e| e.offset(self.pos()))
    }
}

pub trait InitDefaultIter {
    fn init_default_all(self) -> Result<(), Error>;
}
impl<'a, T: FlatDefault + Sized, I: TypeIter> InitDefaultIter for MutIter<'a, TwoOrMoreTypes<T, I>>
where
    MutIter<'a, I>: InitDefaultIter,
{
    fn init_default_all(mut self) -> Result<(), Error> {
        T::init_default(self.value_mut()).map_err(|e| e.offset(self.pos()))?;
        self.next().init_default_all()
    }
}
impl<'a, T: FlatDefault + ?Sized> InitDefaultIter for MutIter<'a, SingleType<T>> {
    fn init_default_all(mut self) -> Result<(), Error> {
        T::init_default(self.value_mut()).map_err(|e| e.offset(self.pos()))
    }
}

pub trait FoldSizeIter {
    /// # Safety
    ///
    /// Internal data must be valid.
    unsafe fn fold_size(self, size: usize) -> usize;
}
impl<'a, T: FlatDefault + Sized, I: TypeIter> FoldSizeIter for RefIter<'a, TwoOrMoreTypes<T, I>>
where
    RefIter<'a, I>: FoldSizeIter,
{
    unsafe fn fold_size(self, size: usize) -> usize {
        self.next().fold_size(ceil_mul(size, T::ALIGN) + T::SIZE)
    }
}
impl<'a, T: FlatDefault + ?Sized> FoldSizeIter for RefIter<'a, SingleType<T>> {
    unsafe fn fold_size(self, _: usize) -> usize {
        (*self.value().as_ptr()).size()
    }
}

pub mod prelude {
    pub use super::{InitDefaultIter, TypeIter, ValidateIter};
}

#[macro_export]
macro_rules! type_list {
    ($first_type:ty, $($types:ty),+ $(,)?) => {
        $crate::iter::TwoOrMoreTypes::<$first_type, _>::new($crate::iter::type_list!($( $types ),*))
    };
    ($type:ty $(,)?) => {
        $crate::iter::SingleType::<$type>::new()
    };
}

#[macro_export]
macro_rules! fold_min_size {
    ($accum:expr; $first_type:ty, $($types:ty),+ $(,)?) => {
        $crate::iter::fold_min_size!(
            $crate::utils::ceil_mul($accum, <$first_type as $crate::FlatBase>::ALIGN) + <$first_type as $crate::FlatSized>::SIZE;
            $( $types ),*
        )
    };
    ($accum:expr; $type:ty $(,)?) => {
        $crate::utils::ceil_mul($accum, <$type as $crate::FlatBase>::ALIGN) + <$type as $crate::FlatBase>::MIN_SIZE
    };
}

pub use {fold_min_size, type_list};

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
    }
}