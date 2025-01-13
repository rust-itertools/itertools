use alloc::vec::Vec;
use core::borrow::BorrowMut;
use std::iter::Fuse;
use std::ops::Index;

use crate::size_hint::{self, SizeHint};

#[derive(Debug, Clone)]
pub struct LazyBuffer<I: Iterator> {
    it: Fuse<I>,
    buffer: Vec<I::Item>,
}

impl<I> LazyBuffer<I>
where
    I: Iterator,
{
    pub fn new(it: I) -> Self {
        Self {
            it: it.fuse(),
            buffer: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn size_hint(&self) -> SizeHint {
        size_hint::add_scalar(self.it.size_hint(), self.len())
    }

    pub fn count(self) -> usize {
        self.len() + self.it.count()
    }

    pub fn get_next(&mut self) -> bool {
        if let Some(x) = self.it.next() {
            self.buffer.push(x);
            true
        } else {
            false
        }
    }

    pub fn prefill(&mut self, len: usize) {
        let buffer_len = self.buffer.len();
        if len > buffer_len {
            let delta = len - buffer_len;
            self.buffer.extend(self.it.by_ref().take(delta));
        }
    }
}

impl<I> LazyBuffer<I>
where
    I: Iterator,
    I::Item: Clone,
{
    pub fn get_at(&self, indices: &[usize]) -> Vec<I::Item> {
        indices.iter().map(|i| self.buffer[*i].clone()).collect()
    }

    pub fn get_array<const K: usize>(&self, indices: [usize; K]) -> [I::Item; K] {
        indices.map(|i| self.buffer[i].clone())
    }
}

impl<I, J> Index<J> for LazyBuffer<I>
where
    I: Iterator,
    I::Item: Sized,
    Vec<I::Item>: Index<J>,
{
    type Output = <Vec<I::Item> as Index<J>>::Output;

    fn index(&self, index: J) -> &Self::Output {
        self.buffer.index(index)
    }
}

pub trait MaybeConstUsize: Clone + Copy + std::fmt::Debug {
    /*TODO const*/
    fn value(self) -> usize;
}

#[derive(Clone, Copy, Debug)]
pub struct ConstUsize<const N: usize>;
impl<const N: usize> MaybeConstUsize for ConstUsize<N> {
    fn value(self) -> usize {
        N
    }
}

impl MaybeConstUsize for usize {
    fn value(self) -> usize {
        self
    }
}

/// A type holding indices of elements in a pool or buffer of items from an inner iterator
/// and used to pick out different combinations in a generic way.
pub trait PoolIndex: BorrowMut<[usize]> {
    type Item<T>;
    type Length: MaybeConstUsize;

    fn extract_item<I: Iterator>(&self, pool: &LazyBuffer<I>) -> Self::Item<I::Item>
    where
        I::Item: Clone;

    fn from_fn<T, F: Fn(usize) -> T>(k: Self::Length, f: F) -> Self::Item<T>;

    fn len(&self) -> Self::Length;
}

impl PoolIndex for Vec<usize> {
    type Item<T> = Vec<T>;
    type Length = usize;

    fn extract_item<I: Iterator>(&self, pool: &LazyBuffer<I>) -> Self::Item<I::Item>
    where
        I::Item: Clone,
    {
        pool.get_at(self)
    }

    fn from_fn<T, F: Fn(usize) -> T>(k: Self::Length, f: F) -> Self::Item<T> {
        (0..k).map(f).collect()
    }

    fn len(&self) -> Self::Length {
        self.len()
    }
}

impl<const K: usize> PoolIndex for [usize; K] {
    type Item<T> = [T; K];
    type Length = ConstUsize<K>;

    fn extract_item<I: Iterator>(&self, pool: &LazyBuffer<I>) -> Self::Item<I::Item>
    where
        I::Item: Clone,
    {
        pool.get_array(*self)
    }

    fn from_fn<T, F: Fn(usize) -> T>(_k: Self::Length, f: F) -> Self::Item<T> {
        std::array::from_fn(f)
    }

    fn len(&self) -> Self::Length {
        ConstUsize::<K>
    }
}
