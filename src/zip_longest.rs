use super::size_hint;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::iter::{Fuse, FusedIterator};

use crate::either_or_both::EitherOrBoth;

// ZipLongest originally written by SimonSapin,
// and dedicated to itertools https://github.com/rust-lang/rust/pull/19283

/// An iterator which iterates two other iterators simultaneously
///
/// This iterator is *fused*.
///
/// See [`.zip_longest()`](crate::Itertools::zip_longest) for more information.
#[derive(Clone, Debug)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct ZipLongest<T, U> {
    a: Fuse<T>,
    b: Fuse<U>,
}

/// Create a new `ZipLongest` iterator.
pub fn zip_longest<T, U>(a: T, b: U) -> ZipLongest<T, U>
where
    T: Iterator,
    U: Iterator,
{
    ZipLongest {
        a: a.fuse(),
        b: b.fuse(),
    }
}

impl<T, U> Iterator for ZipLongest<T, U>
where
    T: Iterator,
    U: Iterator,
{
    type Item = EitherOrBoth<T::Item, U::Item>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match (self.a.next(), self.b.next()) {
            (None, None) => None,
            (Some(a), None) => Some(EitherOrBoth::Left(a)),
            (None, Some(b)) => Some(EitherOrBoth::Right(b)),
            (Some(a), Some(b)) => Some(EitherOrBoth::Both(a, b)),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        size_hint::max(self.a.size_hint(), self.b.size_hint())
    }

    #[inline]
    fn fold<B, F>(self, mut acc: B, mut f: F) -> B
    where
        Self: Sized, F: FnMut(B, Self::Item) -> B
    {
        let ZipLongest { mut a, mut b } = self;

        loop {
            match (a.next(), b.next()) {
                (Some(x), Some(y)) => acc = f(acc, EitherOrBoth::Both(x, y)),
                (Some(x), None) => {
                    acc = f(acc, EitherOrBoth::Left(x));
                    // b is exhausted, so we can drain a.
                    return a.fold(acc, |acc, x| f(acc, EitherOrBoth::Left(x)));
                }
                (None, Some(y)) => {
                    acc = f(acc, EitherOrBoth::Right(y));
                    // a is exhausted, so we can drain b.
                    return b.fold(acc, |acc, y| f(acc, EitherOrBoth::Right(y)));
                }
                (None, None) => return acc, // Both iterators are exhausted.
            }
        }
    }
}

impl<T, U> DoubleEndedIterator for ZipLongest<T, U>
where
    T: DoubleEndedIterator + ExactSizeIterator,
    U: DoubleEndedIterator + ExactSizeIterator,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.a.len().cmp(&self.b.len()) {
            Equal => match (self.a.next_back(), self.b.next_back()) {
                (None, None) => None,
                (Some(a), Some(b)) => Some(EitherOrBoth::Both(a, b)),
                // These can only happen if .len() is inconsistent with .next_back()
                (Some(a), None) => Some(EitherOrBoth::Left(a)),
                (None, Some(b)) => Some(EitherOrBoth::Right(b)),
            },
            Greater => self.a.next_back().map(EitherOrBoth::Left),
            Less => self.b.next_back().map(EitherOrBoth::Right),
        }
    }
}

impl<T, U> ExactSizeIterator for ZipLongest<T, U>
where
    T: ExactSizeIterator,
    U: ExactSizeIterator,
{
}

impl<T, U> FusedIterator for ZipLongest<T, U>
where
    T: Iterator,
    U: Iterator,
{
}
