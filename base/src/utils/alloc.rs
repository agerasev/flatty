use std::{
    alloc::{alloc, dealloc, Layout},
    ops::{Deref, DerefMut},
    slice,
};

pub struct AlignedBytes {
    data: *mut u8,
    layout: Layout,
}

impl AlignedBytes {
    pub fn new(size: usize, align: usize) -> Self {
        let layout = Layout::from_size_align(size, align).unwrap();
        let data = unsafe { alloc(layout) };
        assert!(!data.is_null());
        Self { data, layout }
    }
    pub fn from_slice(src: &[u8], align: usize) -> Self {
        let mut this = Self::new(src.len(), align);
        this.copy_from_slice(src);
        this
    }
    pub fn layout(&self) -> Layout {
        self.layout
    }
}

impl Drop for AlignedBytes {
    fn drop(&mut self) {
        unsafe { dealloc(self.data, self.layout) };
    }
}

impl Deref for AlignedBytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data, self.layout.size()) }
    }
}

impl DerefMut for AlignedBytes {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data, self.layout.size()) }
    }
}

unsafe impl Send for AlignedBytes {}
unsafe impl Sync for AlignedBytes {}
