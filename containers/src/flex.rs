use crate::{error::EmptyError, vec::Length};
use core::{marker::PhantomData, ptr, slice};
use flatty_base::{
    emplacer::Emplacer,
    error::{Error, ErrorKind},
    traits::{Flat, FlatBase, FlatDefault, FlatSized, FlatUnsized, FlatValidate},
    utils::{
        iter::{Data, UncheckedMutData, UncheckedRefData},
        max,
        mem::slice_ptr_len,
    },
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
    const OFFSET_SIZE: usize = max(L::SIZE, T::ALIGN);

    pub fn len(&self) -> usize {
        self.bytes_iter().map(Result::unwrap).count()
    }
    pub fn is_empty(&self) -> bool {
        self.bytes_iter().next().is_none()
    }

    fn bytes_iter(&self) -> DataIter<'_, T, L, &'_ [u8]> {
        DataIter::new(&self.data)
    }
    fn bytes_mut_iter(&mut self) -> DataIter<'_, T, L, &'_ mut [u8]> {
        DataIter::new(&mut self.data)
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ T> {
        DataIter::<'_, T, L, _>::new(unsafe { UncheckedRefData::new(&self.data) }).map(|res| res.unwrap())
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &'_ mut T> {
        DataIter::<'_, T, L, _>::new(unsafe { UncheckedMutData::new(&mut self.data) }).map(|res| res.unwrap())
    }
}

struct DataIter<'a, T, L, D>
where
    T: Flat + ?Sized,
    L: Flat + Length,
    D: Data<'a>,
{
    data: D,
    pos: usize,
    _ghost: PhantomData<&'a (L, T)>,
}

impl<'a, T, L, D> DataIter<'a, T, L, D>
where
    T: Flat + ?Sized,
    L: Flat + Length,
    D: Data<'a>,
{
    fn new(data: D) -> Self {
        Self {
            data,
            pos: 0,
            _ghost: PhantomData,
        }
    }
}

impl<'a, T, L, D> Iterator for DataIter<'a, T, L, D>
where
    T: Flat + ?Sized,
    L: Flat + Length,
    D: Data<'a>,
{
    type Item = Result<D::Output<T>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_offset = match L::from_bytes(self.data.bytes()) {
            Ok(x) => x.to_usize().unwrap(),
            Err(e) => return Some(Err(e.offset(self.pos))),
        };

        if next_offset == 0 {
            return None;
        }

        let payload_offset = FlexVec::<T, L>::OFFSET_SIZE;
        if payload_offset > next_offset {
            return Some(Err(Error {
                kind: ErrorKind::InsufficientSize,
                pos: self.pos + payload_offset,
            }));
        }

        let mut data = None::<D>;
        take_mut::take(&mut self.data, |d| {
            let (d, next_data) = d.split(next_offset);
            data = Some(d);
            next_data
        });
        self.pos += next_offset;

        let (_, payload) = data.unwrap().split(payload_offset);
        Some(Ok(payload.value()))
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
    const MIN_SIZE: usize = Self::OFFSET_SIZE;

    fn size(&self) -> usize {
        let mut iter = self.bytes_iter();
        (&mut iter).map(Result::unwrap).count(); // Exhaust iterator
        iter.pos + Self::OFFSET_SIZE
    }
}

unsafe impl<T, L> FlatUnsized for FlexVec<T, L>
where
    T: Flat + ?Sized,
    L: Flat + Length,
{
    type AlignAs = FlexVecAlignAs<T, L>;

    unsafe fn ptr_from_bytes(bytes: *mut [u8]) -> *mut Self {
        ptr::slice_from_raw_parts_mut(bytes as *mut u8, slice_ptr_len(bytes)) as *mut Self
    }
    unsafe fn ptr_to_bytes(this: *mut Self) -> *mut [u8] {
        ptr::slice_from_raw_parts_mut(this as *mut u8, slice_ptr_len(this as *mut [u8]))
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
        let mut iter = vec.bytes_mut_iter();
        for item_emplacer in self.iter {
            let (mut next_offset, mut payload) = iter.data.split_at_mut(FlexVec::<T, L>::OFFSET_SIZE);
            let item = item_emplacer.emplace(&mut payload)?;
            L::emplace(L::from_usize(item.size()).unwrap(), &mut next_offset)?;
            assert!(iter.next().is_some());
        }
        L::emplace(L::zero(), iter.data)?;
        Ok(vec)
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
        for item_bytes in DataIter::<'_, T, L, _>::new(bytes) {
            T::validate(item_bytes?)?;
        }
        Ok(())
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
        let mut bytes_ptr_iter = self.bytes_ptr_iter(CounterStep {
            count: self.len(),
            f: step_size_unchecked::<T>,
        });
        let _ = (&mut bytes_ptr_iter).last(); // Skip to the end.
        let bytes = unsafe { slice::from_raw_parts_mut(bytes_ptr_iter.ptr, bytes_ptr_iter.len) };
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
