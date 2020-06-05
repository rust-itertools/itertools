use core::ops::{ Range, RangeTo, RangeFrom, RangeFull, RangeInclusive, RangeToInclusive };
use core::iter::{Skip, Take};
use crate::Itertools;

/// Used by the ``range`` function to know which iterator
/// to turn different ranges into.
pub trait IntoRangeIter<IterFrom> {
	type IterTo;

	fn into_range_iter(self, from: IterFrom) -> Self::IterTo;
}

impl<I> IntoRangeIter<I> for Range<usize>
	where I: Iterator
{
	type IterTo = Take<Skip<I>>;

	fn into_range_iter(self, iter: I) -> Self::IterTo {
		iter.skip(self.start)
			.take(self.end.saturating_sub(self.start))
	}
}

impl<I> IntoRangeIter<I> for RangeInclusive<usize>
	where I: Iterator
{
	type IterTo = Take<Skip<I>>;

	fn into_range_iter(self, iter: I) -> Self::IterTo {
		iter.skip(*self.start())
			.take(
				(1 + *self.end())
				.saturating_sub(*self.start())
			)
	}
}

impl<I> IntoRangeIter<I> for RangeTo<usize>
	where I: Iterator
{
	type IterTo = Take<I>;

	fn into_range_iter(self, iter: I) -> Self::IterTo {
		iter.take(self.end)
	}
}

impl<I> IntoRangeIter<I> for RangeToInclusive<usize>
	where I: Iterator
{
	type IterTo = Take<I>;

	fn into_range_iter(self, iter: I) -> Self::IterTo {
		iter.take(self.end + 1)
	}
}

impl<I> IntoRangeIter<I> for RangeFrom<usize>
	where I: Iterator
{
	type IterTo = Skip<I>;

	fn into_range_iter(self, iter: I) -> Self::IterTo {
		iter.skip(self.start)
	}
}

impl<I> IntoRangeIter<I> for RangeFull
	where I: Iterator
{
	type IterTo = I;

	fn into_range_iter(self, iter: I) -> Self::IterTo {
		iter
	}
}

/// Limits an iterator to a range. See [`Itertools::range`]
/// for more information.
pub fn range<I, R>(iter: I, range: R)
    -> R::IterTo
    where I: IntoIterator,
          R: IntoRangeIter<I::IntoIter>
{
	range.into_range_iter(iter.into_iter())
}
