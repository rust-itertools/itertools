use core::iter::{Skip, Take};
use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

#[cfg(doc)]
use crate::Itertools;

mod private_iter_index {
    use core::ops;

    pub trait Sealed {}

    impl Sealed for ops::Range<usize> {}
    impl Sealed for ops::RangeInclusive<usize> {}
    impl Sealed for ops::RangeTo<usize> {}
    impl Sealed for ops::RangeToInclusive<usize> {}
    impl Sealed for ops::RangeFrom<usize> {}
    impl Sealed for ops::RangeFull {}
}

/// Used by [`get`] and [`Itertools::get`] to know which iterator
/// to turn different ranges into.
pub trait IteratorIndex<I>: private_iter_index::Sealed
where
    I: Iterator,
{
    /// The type returned for this type of index.
    type Output: Iterator<Item = I::Item>;

    /// Returns an adapted iterator for the current index.
    ///
    /// Prefer calling [`get`] or [`Itertools::get`] instead
    /// of calling this directly.
    fn index(self, from: I) -> Self::Output;
}

impl<I> IteratorIndex<I> for Range<usize>
where
    I: Iterator,
{
    type Output = Skip<Take<I>>;

    fn index(self, iter: I) -> Self::Output {
        iter.take(self.end).skip(self.start)
    }
}

impl<I> IteratorIndex<I> for RangeInclusive<usize>
where
    I: Iterator,
{
    type Output = Take<Skip<I>>;

    fn index(self, iter: I) -> Self::Output {
        iter.skip(*self.start())
            .take((1 + *self.end()).saturating_sub(*self.start()))
    }
}

impl<I> IteratorIndex<I> for RangeTo<usize>
where
    I: Iterator,
{
    type Output = Take<I>;

    fn index(self, iter: I) -> Self::Output {
        iter.take(self.end)
    }
}

impl<I> IteratorIndex<I> for RangeToInclusive<usize>
where
    I: Iterator,
{
    type Output = Take<I>;

    fn index(self, iter: I) -> Self::Output {
        iter.take(self.end + 1)
    }
}

impl<I> IteratorIndex<I> for RangeFrom<usize>
where
    I: Iterator,
{
    type Output = Skip<I>;

    fn index(self, iter: I) -> Self::Output {
        iter.skip(self.start)
    }
}

impl<I> IteratorIndex<I> for RangeFull
where
    I: Iterator,
{
    type Output = I;

    fn index(self, iter: I) -> Self::Output {
        iter
    }
}

/// Returns an iterator over a subsection of the iterator.
///
/// See [`Itertools::get`] for more information.
pub fn get<I, R>(iter: I, index: R) -> R::Output
where
    I: IntoIterator,
    R: IteratorIndex<I::IntoIter>,
{
    index.index(iter.into_iter())
}
