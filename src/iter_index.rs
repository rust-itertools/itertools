use crate::iter_index::private_iter_index::{TakeWithUsizeMaxPlusOne, UsizeMaxPlusOne};
#[cfg(doc)]
use crate::Itertools;
use core::iter::{Skip, Take};
use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

mod private_iter_index {
    use core::cmp;
    use core::cmp::Ordering;
    use core::ops;

    pub trait Sealed {}

    impl Sealed for ops::Range<usize> {}
    impl Sealed for ops::RangeInclusive<usize> {}
    impl Sealed for ops::RangeTo<usize> {}
    impl Sealed for ops::RangeToInclusive<usize> {}
    impl Sealed for ops::RangeFrom<usize> {}
    impl Sealed for ops::RangeFull {}

    /// An integer in range `0..=usize::MAX+1`.
    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    pub(crate) struct UsizeMaxPlusOne {
        /// `None -> usize::MAX + 1`.
        pub(crate) n: Option<usize>,
    }

    impl UsizeMaxPlusOne {
        fn checked_dec(self) -> Option<Self> {
            Some(UsizeMaxPlusOne::from(match self.n {
                None => usize::MAX,
                Some(n) => n.checked_sub(1)?,
            }))
        }

        fn saturating_to_usize(self) -> usize {
            self.n.unwrap_or(usize::MAX)
        }

        pub(crate) fn inc(n: usize) -> Self {
            UsizeMaxPlusOne {
                n: n.checked_add(1),
            }
        }
    }

    impl From<usize> for UsizeMaxPlusOne {
        fn from(n: usize) -> Self {
            UsizeMaxPlusOne { n: Some(n) }
        }
    }

    impl PartialOrd for UsizeMaxPlusOne {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for UsizeMaxPlusOne {
        fn cmp(&self, other: &Self) -> Ordering {
            match (self.n, other.n) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(self_n), Some(other_n)) => self_n.cmp(&other_n),
            }
        }
    }

    #[derive(Debug)]
    pub struct TakeWithUsizeMaxPlusOne<I> {
        pub(crate) iter: I,
        pub(crate) rem: UsizeMaxPlusOne,
    }

    impl<I: Iterator> Iterator for TakeWithUsizeMaxPlusOne<I> {
        type Item = I::Item;

        fn next(&mut self) -> Option<Self::Item> {
            self.rem = self.rem.checked_dec()?;
            self.iter.next()
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let (lower, upper) = self.iter.size_hint();
            let lower = UsizeMaxPlusOne::from(lower);
            let upper = upper.map(UsizeMaxPlusOne::from);

            let ret_lower = cmp::min(self.rem, lower).saturating_to_usize();
            let ret_upper = match upper {
                None => self.rem.n,
                Some(upper) => cmp::min(self.rem, upper).n,
            };
            (ret_lower, ret_upper)
        }
    }

    impl<I: DoubleEndedIterator> DoubleEndedIterator for TakeWithUsizeMaxPlusOne<I> {
        fn next_back(&mut self) -> Option<Self::Item> {
            self.rem = self.rem.checked_dec()?;
            self.iter.next_back()
        }
    }
}

/// Used by [`Itertools::get`] to know which iterator
/// to turn different ranges into.
pub trait IteratorIndex<I>: private_iter_index::Sealed
where
    I: Iterator,
{
    /// The type returned for this type of index.
    type Output: Iterator<Item = I::Item>;

    /// Returns an adapted iterator for the current index.
    ///
    /// Prefer calling [`Itertools::get`] instead
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
    type Output = TakeWithUsizeMaxPlusOne<Skip<I>>;

    fn index(self, iter: I) -> Self::Output {
        let length = UsizeMaxPlusOne::inc(self.end().saturating_sub(*self.start()));
        TakeWithUsizeMaxPlusOne {
            rem: length,
            iter: iter.skip(*self.start()),
        }
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
    type Output = TakeWithUsizeMaxPlusOne<I>;

    fn index(self, iter: I) -> Self::Output {
        TakeWithUsizeMaxPlusOne {
            rem: UsizeMaxPlusOne::inc(self.end),
            iter,
        }
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

pub fn get<I, R>(iter: I, index: R) -> R::Output
where
    I: IntoIterator,
    R: IteratorIndex<I::IntoIter>,
{
    index.index(iter.into_iter())
}
