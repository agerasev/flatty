#![allow(dead_code)]

use flatty::AlignedBytes;
use std::ops::{Deref, DerefMut, Range};

pub(crate) struct Buffer {
    data: AlignedBytes,
    window: Range<usize>,
}

impl Buffer {
    pub(crate) fn new(capacity: usize, align: usize) -> Self {
        Self {
            data: AlignedBytes::new(capacity, align),
            window: 0..0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    pub(crate) fn preceding_len(&self) -> usize {
        self.window.start
    }
    pub(crate) fn occupied_len(&self) -> usize {
        self.window.end - self.window.start
    }
    pub(crate) fn vacant_len(&self) -> usize {
        self.capacity() - self.window.end
    }
    pub(crate) fn free_len(&self) -> usize {
        self.preceding_len() + self.vacant_len()
    }

    pub(crate) fn occupied(&self) -> &[u8] {
        &self.data[self.window.clone()]
    }
    pub(crate) fn occupied_mut(&mut self) -> &mut [u8] {
        &mut self.data[self.window.clone()]
    }
    pub(crate) fn vacant_mut(&mut self) -> &mut [u8] {
        &mut self.data[self.window.end..]
    }

    pub(crate) fn clear(&mut self) {
        self.window = 0..0;
    }
    /// Skip first `count` occupied bytes.
    pub(crate) fn skip(&mut self, count: usize) {
        self.window.start += count;
        assert!(self.window.start <= self.window.end);
        if self.window.is_empty() {
            self.window = 0..0;
        }
    }
    /// Make first `count` vacant bytes occupied.
    pub(crate) fn advance(&mut self, count: usize) {
        self.window.end += count;
        assert!(self.window.end <= self.capacity());
    }
    /// Move data to the beginning of buffer to get free room for next data.
    pub(crate) fn make_contiguous(&mut self) {
        self.data.copy_within(self.window.clone(), 0);
        self.window = 0..(self.window.end - self.window.start);
    }
}

pub struct IoBuffer<P> {
    pub(crate) pipe: P,
    pub(crate) buffer: Buffer,
    pub(crate) poisoned: bool,
}

impl<P> IoBuffer<P> {
    pub fn new(pipe: P, capacity: usize, align: usize) -> Self {
        Self {
            pipe,
            buffer: Buffer::new(capacity, align),
            poisoned: false,
        }
    }
    pub(crate) fn split_mut(&mut self) -> (&mut P, &mut Buffer) {
        (&mut self.pipe, &mut self.buffer)
    }
}

impl<P> Deref for IoBuffer<P> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.buffer.occupied()
    }
}
impl<P> DerefMut for IoBuffer<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buffer.occupied_mut()
    }
}
