use crate::{size_hint, PeekingNext};
use alloc::collections::VecDeque;
use std::iter::Fuse;

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Debug, Clone)]
pub struct MultiPeekGeneral<I: Iterator, Idx> {
    iter: Fuse<I>,
    buf: VecDeque<I::Item>,
    index: Idx,
}

/// See [`multipeek()`] for more information.
pub type MultiPeek<I> = MultiPeekGeneral<I, usize>;

/// See [`peek_nth()`] for more information.
pub type PeekNth<I> = MultiPeekGeneral<I, ()>;

/// An iterator adaptor that allows the user to peek at multiple `.next()`
/// values without advancing the base iterator.
///
/// [`IntoIterator`] enabled version of [`Itertools::multipeek`](crate::Itertools::multipeek).
pub fn multipeek<I>(iterable: I) -> MultiPeek<I::IntoIter>
where
    I: IntoIterator,
{
    MultiPeek {
        iter: iterable.into_iter().fuse(),
        buf: VecDeque::new(),
        index: 0,
    }
}

/// A drop-in replacement for [`std::iter::Peekable`] which adds a `peek_nth`
/// method allowing the user to `peek` at a value several iterations forward
/// without advancing the base iterator.
///
/// This differs from `multipeek` in that subsequent calls to `peek` or
/// `peek_nth` will always return the same value until `next` is called
/// (making `reset_peek` unnecessary).
pub fn peek_nth<I>(iterable: I) -> PeekNth<I::IntoIter>
where
    I: IntoIterator,
{
    PeekNth {
        iter: iterable.into_iter().fuse(),
        buf: VecDeque::new(),
        index: (),
    }
}

pub trait PeekIndex {
    fn reset_index(&mut self);
    fn add(&mut self, n: usize);
    fn index(&self) -> usize;
}

impl PeekIndex for () {
    fn reset_index(&mut self) {}
    fn add(&mut self, _n: usize) {}
    fn index(&self) -> usize {
        0
    }
}

impl PeekIndex for usize {
    fn reset_index(&mut self) {
        *self = 0;
    }
    fn add(&mut self, n: usize) {
        *self += n
    }
    fn index(&self) -> usize {
        *self
    }
}

impl<I: Iterator, Idx: PeekIndex> MultiPeekGeneral<I, Idx> {
    /// Works similarly to `.next()`, but does not advance the iterator itself.
    ///
    /// - For `MultiPeekGeneral`, calling `.peek()` will increment the internal index,
    ///   so the next call to `.peek()` will advance to the next element in the buffer.
    ///   The actual underlying iterator is not consumed, allowing multiple peeks
    ///   without advancing the iterator itself.
    /// - For `peek_nth`, since there is no internal index used, calling `.peek()`
    ///   multiple times will not advance the internal state or the iterator, providing
    ///   a consistent view of the same element.
    pub fn peek(&mut self) -> Option<&I::Item> {
        let ret = if self.index.index() < self.buf.len() {
            Some(&self.buf[self.index.index()])
        } else {
            match self.iter.next() {
                Some(x) => {
                    self.buf.push_back(x);
                    Some(&self.buf[self.index.index()])
                }
                None => return None,
            }
        };
        self.index.add(1);
        ret
    }

    /// Works exactly like the `peek_mut` method in [`std::iter::Peekable`].
    pub fn peek_mut(&mut self) -> Option<&mut I::Item> {
        self.peek_nth_mut(0)
    }

    /// Returns a reference to the `nth` value without advancing the iterator.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use itertools::peek_nth;
    ///
    /// let xs = vec![1, 2, 3];
    /// let mut iter = peek_nth(xs.into_iter());
    ///
    /// assert_eq!(iter.peek_nth(0), Some(&1));
    /// assert_eq!(iter.next(), Some(1));
    ///
    /// // The iterator does not advance even if we call `peek_nth` multiple times
    /// assert_eq!(iter.peek_nth(0), Some(&2));
    /// assert_eq!(iter.peek_nth(1), Some(&3));
    /// assert_eq!(iter.next(), Some(2));
    ///
    /// // Calling `peek_nth` past the end of the iterator will return `None`
    /// assert_eq!(iter.peek_nth(1), None);
    /// ```
    pub fn peek_nth(&mut self, n: usize) -> Option<&I::Item> {
        let unbuffered_items = (n + 1).saturating_sub(self.buf.len());

        self.buf.extend(self.iter.by_ref().take(unbuffered_items));

        let ret = self.buf.get(n);

        if ret.is_some() {
            self.index.add(n + 1);
        }

        ret
    }

    /// Returns a mutable reference to the `nth` value without advancing the iterator.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use itertools::peek_nth;
    ///
    /// let xs = vec![1, 2, 3, 4, 5];
    /// let mut iter = peek_nth(xs.into_iter());
    ///
    /// assert_eq!(iter.peek_nth_mut(0), Some(&mut 1));
    /// assert_eq!(iter.next(), Some(1));
    ///
    /// // The iterator does not advance even if we call `peek_nth_mut` multiple times
    /// assert_eq!(iter.peek_nth_mut(0), Some(&mut 2));
    /// assert_eq!(iter.peek_nth_mut(1), Some(&mut 3));
    /// assert_eq!(iter.next(), Some(2));
    ///
    /// // Peek into the iterator and set the value behind the mutable reference.
    /// if let Some(p) = iter.peek_nth_mut(1) {
    ///     assert_eq!(*p, 4);
    ///     *p = 9;
    /// }
    ///
    /// // The value we put in reappears as the iterator continues.
    /// assert_eq!(iter.next(), Some(3));
    /// assert_eq!(iter.next(), Some(9));
    ///
    /// // Calling `peek_nth_mut` past the end of the iterator will return `None`
    /// assert_eq!(iter.peek_nth_mut(1), None);
    /// ```
    pub fn peek_nth_mut(&mut self, n: usize) -> Option<&mut I::Item> {
        self.index.add(n);

        let unbuffered_items = (n + 1).saturating_sub(self.buf.len());

        self.buf.extend(self.iter.by_ref().take(unbuffered_items));

        self.buf.get_mut(n)
    }

    /// Works exactly like the `next_if` method in [`std::iter::Peekable`].
    pub fn next_if(&mut self, func: impl FnOnce(&I::Item) -> bool) -> Option<I::Item> {
        match self.next() {
            Some(item) if func(&item) => Some(item),
            Some(item) => {
                self.buf.push_front(item);
                None
            }
            _ => None,
        }
    }

    /// Works exactly like the `next_if_eq` method in [`std::iter::Peekable`].
    pub fn next_if_eq<T>(&mut self, expected: &T) -> Option<I::Item>
    where
        T: ?Sized,
        I::Item: PartialEq<T>,
    {
        self.next_if(|next| next == expected)
    }

    /// Works exactly like `next_if`, but for the `nth` value without advancing the iterator.
    pub fn nth_if(&mut self, n: usize, func: impl FnOnce(&I::Item) -> bool) -> Option<&I::Item> {
        match self.peek_nth(n) {
            Some(item) if func(item) => Some(item),
            _ => None,
        }
    }

    /// Works exactly like `next_if_eq`, but for the `nth` value without advancing the iterator.
    pub fn nth_if_eq<T>(&mut self, n: usize, expected: &T) -> Option<&I::Item>
    where
        T: ?Sized,
        I::Item: PartialEq<T>,
    {
        self.nth_if(n, |item| item == expected)
    }
}
impl<I: Iterator> MultiPeek<I> {
    /// Reset the peeking “cursor”
    pub fn reset_peek(&mut self) {
        self.index = 0
    }
}

impl<I: Iterator, Idx: PeekIndex> Iterator for MultiPeekGeneral<I, Idx> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.index.reset_index();
        self.buf.pop_front().or_else(|| self.iter.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        size_hint::add_scalar(self.iter.size_hint(), self.buf.len())
    }

    fn fold<B, F>(self, mut init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        init = self.buf.into_iter().fold(init, &mut f);
        self.iter.fold(init, f)
    }
}

impl<I: ExactSizeIterator, Idx: PeekIndex> ExactSizeIterator for MultiPeekGeneral<I, Idx> {}

impl<I, Idx> PeekingNext for MultiPeekGeneral<I, Idx>
where
    I: Iterator,
    Idx: PeekIndex,
{
    fn peeking_next<F>(&mut self, accept: F) -> Option<Self::Item>
    where
        F: FnOnce(&Self::Item) -> bool,
    {
        if self.buf.is_empty() {
            if let Some(r) = self.peek() {
                if !accept(r) {
                    return None;
                }
            }
        } else if let Some(r) = self.buf.front() {
            if !accept(r) {
                return None;
            }
        }
        self.next()
    }
}
