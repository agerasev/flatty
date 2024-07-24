use crate::{error::EmptyError, vec::Length};
use core::{marker::PhantomData, ptr, slice};
use flatty_base::{
    emplacer::Emplacer,
    error::{Error, ErrorKind},
    traits::{Flat, FlatBase, FlatDefault, FlatSized, FlatUnsized, FlatValidate},
    utils::{floor_mul, max, mem::slice_ptr_len},
};

/// Growable flat vector of possibly **unsized** items.
///
///
/// # Internal representation
///
/// ```
///    +-----------------+                 +---------------+
///    |                 v                 |               v
/// [next0][data0][..][next1][data1][..][next2][data2][..][0]
///                      |                 ^
///                      +-----------------+
/// ```
#[repr(C)]
pub struct FlexVec<T, L = usize>
where
    T: Flat + ?Sized,
    L: Flat + Length,
{
    _align: [FlexVecAlignAs<T, L>; 0],
    data: [u8],
}

impl<T, L> FlexVec<T, L>
where
    T: Flat + ?Sized,
    L: Flat + Length,
{
    pub fn len(&self) -> usize {
        self.len.to_usize().unwrap()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn cursor<S: CursorStep>(&self, step: S) -> Cursor<'_, L, S> {
        Cursor::new(&self.data, step)
    }
    pub fn iter(&self) -> impl Iterator<Item = &'_ T> {
        self.cursor(CounterStep {
            count: self.len(),
            f: step_from_bytes_unchecked::<'_, T>,
        })
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &'_ mut T> {
        self.cursor(CounterStep {
            count: self.len(),
            f: step_from_mut_bytes_unchecked::<'_, T>,
        })
    }
}

struct Cursor<'a, L: Flat + Length, S: CursorStep> {
    /// Current position in the data.
    ptr: *mut u8,
    /// Remaining data len in bytes.
    len: usize,
    /// Step function
    step: S,

    _ghost: PhantomData<&'a L>,
}

unsafe impl<'a, L: Flat + Length, S: CursorStep> Send for Cursor<'a, L, S> where S: Send {}
unsafe impl<'a, L: Flat + Length, S: CursorStep> Sync for Cursor<'a, L, S> where S: Sync {}

impl<'a, L: Flat + Length, S: CursorStep> Cursor<'a, L, S> {
    fn new(data: &'a [u8], step: S) -> Self {
        Self {
            ptr: data.as_ptr() as *mut u8,
            len: data.len(),
            step,
            _ghost: PhantomData,
        }
    }
}

impl<'a, L: Flat + Length, S: CursorStep> Iterator for Cursor<'a, L, S> {
    type Item = S::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((item, size)) = self.step.step(self.ptr, self.len) {
            assert!(size <= self.len);
            self.ptr = unsafe { self.ptr.add(size) };
            self.len -= size;
            Some(item)
        } else {
            None
        }
    }
}

trait CursorStep {
    type Item: Sized;
    fn step(&mut self, ptr: *mut u8, len: usize) -> Result<Self::Item, Error>;
}

struct CounterStep<U, F: FnMut(*mut [u8], bool) -> (U, usize)> {
    /// Remaining items count.
    count: usize,
    f: F,
}

impl<U, F: FnMut(*mut [u8], bool) -> (U, usize)> CursorStep for CounterStep<U, F> {
    type Item = U;

    fn step(&mut self, ptr: *mut u8, len: usize) -> Option<(Self::Item, usize)> {
        if self.count != 0 {
            self.count -= 1;
            Some((self.f)(ptr::slice_from_raw_parts_mut(ptr, len), self.count == 0))
        } else {
            None
        }
    }
}

fn step_from_bytes_unchecked<'a, T: Flat + ?Sized>(data_ptr: *mut [u8], last: bool) -> (&'a T, usize) {
    let data = unsafe { &*(data_ptr as *const [u8]) };
    let mut item = unsafe { T::from_bytes_unchecked(data) };
    let size = item.size();
    if !last {
        item = unsafe { T::from_bytes_unchecked(data.get_unchecked(..size)) };
        debug_assert_eq!(size, item.size());
    }
    (item, size)
}

fn step_from_mut_bytes_unchecked<'a, T: Flat + ?Sized>(data_ptr: *mut [u8], last: bool) -> (&'a mut T, usize) {
    let data = unsafe { &mut *data_ptr };
    if !last {
        let item = unsafe { T::from_mut_bytes_unchecked(data) };
        let size = item.size();
        (item, size)
    } else {
        let size = unsafe { T::from_bytes_unchecked(data) }.size();
        let item = unsafe { T::from_mut_bytes_unchecked(data.get_unchecked_mut(..size)) };
        debug_assert_eq!(size, item.size());
        (item, size)
    }
}

/// Sized type that has same alignment as [`FlexVec<T, L>`](`FlexVec`).
#[repr(C)]
pub struct FlexVecAlignAs<T, L>(T::AlignAs, L)
where
    T: Flat + ?Sized,
    L: Flat + Length;

unsafe impl<T, L> FlatBase for FlexVec<T, L>
where
    T: Flat + ?Sized,
    L: Flat + Length,
{
    const ALIGN: usize = max(L::ALIGN, T::ALIGN);
    const MIN_SIZE: usize = Self::DATA_OFFSET;

    fn size(&self) -> usize {
        Self::DATA_OFFSET
            + self
                .cursor(CounterStep {
                    count: self.len(),
                    f: step_size_unchecked::<T>,
                })
                .sum::<usize>()
    }
}

fn step_size_unchecked<T: Flat + ?Sized>(data_ptr: *mut [u8], _: bool) -> (usize, usize) {
    let data = unsafe { &*(data_ptr as *const [u8]) };
    let item = unsafe { T::from_bytes_unchecked(data) };
    let size = item.size();
    (size, size)
}

unsafe impl<T, L> FlatUnsized for FlexVec<T, L>
where
    T: Flat + ?Sized,
    L: Flat + Length,
{
    type AlignAs = FlexVecAlignAs<T, L>;

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
pub struct FromIterator<T, E, I>
where
    T: Flat + ?Sized,
    E: Emplacer<T>,
    I: Iterator<Item = E>,
{
    iter: I,
    _ghost: PhantomData<T>,
}

impl<T, E, I> FromIterator<T, E, I>
where
    T: Flat + ?Sized,
    E: Emplacer<T>,
    I: Iterator<Item = E>,
{
    pub fn new<J: IntoIterator<IntoIter = I>>(into_iter: J) -> Self {
        Self {
            iter: into_iter.into_iter(),
            _ghost: PhantomData,
        }
    }
}

unsafe impl<T, L> Emplacer<FlexVec<T, L>> for Empty
where
    T: Flat + ?Sized,
    L: Flat + Length,
{
    unsafe fn emplace_unchecked(self, bytes: &mut [u8]) -> Result<&mut FlexVec<T, L>, Error> {
        unsafe { (bytes.as_mut_ptr() as *mut L).write(L::zero()) };
        Ok(unsafe { FlexVec::from_mut_bytes_unchecked(bytes) })
    }
}

unsafe impl<T, L, E, I> Emplacer<FlexVec<T, L>> for FromIterator<T, E, I>
where
    T: Flat + ?Sized,
    L: Flat + Length,
    E: Emplacer<T>,
    I: Iterator<Item = E>,
{
    unsafe fn emplace_unchecked(self, bytes: &mut [u8]) -> Result<&mut FlexVec<T, L>, Error> {
        let vec = unsafe { <Empty as Emplacer<FlexVec<T, L>>>::emplace_unchecked(Empty, bytes) }?;
        let mut cursor = vec.cursor(EmplacerStep {
            iter: self.iter,
            offset: 0,
            max_count: L::max_value().to_usize().unwrap(),
            _ghost: PhantomData,
        });
        let count = cursor
            .try_fold(0, |count, res| res.map(|()| count + 1))
            .map_err(|e| e.offset(FlexVec::<T, L>::DATA_OFFSET))?;
        vec.len = L::from_usize(count).unwrap();
        Ok(vec)
    }
}

struct EmplacerStep<T, E, I>
where
    T: Flat + ?Sized,
    E: Emplacer<T>,
    I: Iterator<Item = E>,
{
    iter: I,
    offset: usize,
    max_count: usize,
    _ghost: PhantomData<T>,
}

impl<T, E, I> CursorStep for EmplacerStep<T, E, I>
where
    T: Flat + ?Sized,
    E: Emplacer<T>,
    I: Iterator<Item = E>,
{
    type Item = Result<(), Error>;

    fn step(&mut self, ptr: *mut u8, len: usize) -> Option<(Self::Item, usize)> {
        let emplacer = self.iter.next()?;
        if self.max_count != 0 {
            self.max_count -= 1;
            let data = unsafe { slice::from_raw_parts_mut(ptr, len) };
            Some(match emplacer.emplace(data) {
                Ok(vec) => (Ok(()), vec.size()),
                Err(e) => (Err(e.offset(self.offset)), 0),
            })
        } else {
            Some((
                Err(Error {
                    kind: ErrorKind::InsufficientSize,
                    pos: self.offset,
                }),
                0,
            ))
        }
    }
}

impl<T, L> FlatDefault for FlexVec<T, L>
where
    T: Flat + ?Sized,
    L: Flat + Length,
{
    type DefaultEmplacer = Empty;

    fn default_emplacer() -> Empty {
        Empty
    }
}

unsafe impl<T, L> FlatValidate for FlexVec<T, L>
where
    T: Flat + ?Sized,
    L: Flat + Length,
{
    unsafe fn validate_unchecked(bytes: &[u8]) -> Result<(), Error> {
        unsafe { L::validate_unchecked(bytes) }?;
        let this = unsafe { Self::from_bytes_unchecked(bytes) };
        let mut cursor = this.cursor(CounterStep {
            count: this.len(),
            f: step_validate::<T>,
        });
        cursor.try_fold(0, |offset, res| match res {
            Ok(size) => Ok(offset + size),
            Err(e) => Err(e.offset(offset)),
        })?;
        Ok(())
    }
}

fn step_validate<T: Flat + ?Sized>(data_ptr: *mut [u8], _: bool) -> (Result<usize, Error>, usize) {
    let data = unsafe { &*(data_ptr as *const [u8]) };
    match T::validate(data) {
        Ok(()) => {
            let size = unsafe { T::from_bytes_unchecked(data) }.size();
            (Ok(size), size)
        }
        Err(e) => (Err(e), 0),
    }
}

unsafe impl<T, L> Flat for FlexVec<T, L>
where
    T: Flat + ?Sized,
    L: Flat + Length,
{
}

impl<T, L> FlexVec<T, L>
where
    T: Flat + ?Sized,
    L: Flat + Length,
{
    pub fn push<E: Emplacer<T>>(&mut self, emplacer: E) -> Result<&mut T, Error> {
        if self.len() >= L::max_value().to_usize().unwrap() {
            return Err(Error {
                kind: ErrorKind::InsufficientSize,
                pos: 0,
            });
        }
        let mut cursor = self.cursor(CounterStep {
            count: self.len(),
            f: step_size_unchecked::<T>,
        });
        let _ = (&mut cursor).last(); // Skip to the end.
        let bytes = unsafe { slice::from_raw_parts_mut(cursor.ptr, cursor.len) };
        emplacer.emplace(bytes)?;
        self.len += L::one();
        Ok(unsafe { T::from_mut_bytes_unchecked(bytes) })
    }
    pub fn push_default(&mut self) -> Result<&mut T, Error>
    where
        T: FlatDefault,
    {
        self.push(T::default_emplacer())
    }

    pub fn clear(&mut self) {
        self.truncate(0);
    }
    pub fn pop(&mut self) -> Result<(), EmptyError> {
        self.iter_mut()
            .last()
            .map(|x| unsafe { ptr::drop_in_place(x as *mut T) })
            .ok_or(EmptyError)
    }
    pub fn truncate(&mut self, len: usize) {
        if len >= self.len() {
            return;
        }
        for x in self.iter_mut().skip(len) {
            unsafe { ptr::drop_in_place(x as *mut T) };
        }
        self.len = L::from_usize(len).unwrap();
    }
}

/// Creates [`FlexVec`] emplacer from given array of emplacers.
#[macro_export]
macro_rules! flex_vec {
    () => {
        $crate::flex::FromIter([])
    };
    ($($x:expr),+ $(,)?) => {
        $crate::flex::FromIter([$($x),+])
    };
}
pub use flex_vec;

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::{bytes::AlignedBytes, vec::FlatVec};
    use core::mem::{align_of_val, size_of_val};

    #[test]
    fn align() {
        let mut bytes = AlignedBytes::new(4 + 3 * 2, 4);
        let flex_vec = FlexVec::<FlatVec<i32, u16>, u16>::default_in_place(&mut bytes).unwrap();
        assert_eq!(FlexVec::<FlatVec<i32, u16>, u16>::DATA_OFFSET, 4);

        assert_eq!(align_of_val(flex_vec), 4);
        assert_eq!(size_of_val(flex_vec), 8);
    }

    #[test]
    fn push() {
        let mut bytes = AlignedBytes::new(4 + 4 * 6, 4);
        let flex_vec = FlexVec::<FlatVec<i32, u16>, u16>::default_in_place(&mut bytes).unwrap();
        assert_eq!(FlexVec::<FlatVec<i32, u16>, u16>::DATA_OFFSET, flex_vec.size());

        flex_vec.push_default().unwrap().extend_from_slice(&[0, 1]).unwrap();
        flex_vec.push_default().unwrap();
        flex_vec.push_default().unwrap().extend_from_slice(&[2]).unwrap();

        assert_eq!(flex_vec.len(), 3);
        assert_eq!(flex_vec.iter().next().unwrap().as_slice(), [0, 1].as_slice());
        assert_eq!(flex_vec.iter().nth(1).unwrap().as_slice(), [].as_slice());
        assert_eq!(flex_vec.iter().nth(2).unwrap().as_slice(), [2].as_slice());
        assert!(flex_vec.iter().nth(3).is_none());
    }

    #[test]
    fn push_modify() {
        let mut bytes = AlignedBytes::new(4 + 4 * 6, 4);
        let flex_vec = FlexVec::<FlatVec<i32, u16>, u16>::default_in_place(&mut bytes).unwrap();
        assert_eq!(FlexVec::<FlatVec<i32, u16>, u16>::DATA_OFFSET, flex_vec.size());

        flex_vec.push_default().unwrap().extend_from_slice(&[0, 1]).unwrap();
        flex_vec.push_default().unwrap().extend_from_slice(&[2]).unwrap();

        assert_eq!(flex_vec.iter_mut().next().unwrap().pop(), Some(1));

        assert_eq!(flex_vec.len(), 2);
        assert_eq!(flex_vec.iter().next().unwrap().as_slice(), [0].as_slice());
        assert_eq!(flex_vec.iter().nth(1).unwrap().as_slice(), [2].as_slice());
        assert!(flex_vec.iter().nth(2).is_none());
    }
}
