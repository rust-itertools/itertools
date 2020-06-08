use core::ops::{ Range, RangeTo, RangeFrom, RangeFull, RangeInclusive, RangeToInclusive };
use core::iter::{Skip, Take};
use crate::Itertools;

mod private_into_range_iter {
	use core::ops;

	pub trait Sealed {}

	impl Sealed for ops::Range<usize> {}
	impl Sealed for ops::RangeInclusive<usize> {}
	impl Sealed for ops::RangeTo<usize> {}
	impl Sealed for ops::RangeToInclusive<usize> {}
	impl Sealed for ops::RangeFrom<usize> {}
	impl Sealed for ops::RangeFull {}
	impl Sealed for usize {}
}

/// Used by the ``range`` function to know which iterator
/// to turn different ranges into.
pub trait IntoRangeIter<T> : private_into_range_iter::Sealed {
	type Output;

	/// Returns an iterator(or value) in the specified range.
	///
	/// Prefer calling [`range`] or [`Itertools::range`] instead
	/// of calling this directly.
	fn into_range_iter(self, from: T) -> Self::Output;
}

impl<I> IntoRangeIter<I> for Range<usize>
	where I: Iterator
{
	type Output = Take<Skip<I>>;

	fn into_range_iter(self, iter: I) -> Self::Output {
		iter.skip(self.start)
			.take(self.end.saturating_sub(self.start))
	}
}

impl<I> IntoRangeIter<I> for RangeInclusive<usize>
	where I: Iterator
{
	type Output = Take<Skip<I>>;

	fn into_range_iter(self, iter: I) -> Self::Output {
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
	type Output = Take<I>;

	fn into_range_iter(self, iter: I) -> Self::Output {
		iter.take(self.end)
	}
}

impl<I> IntoRangeIter<I> for RangeToInclusive<usize>
	where I: Iterator
{
	type Output = Take<I>;

	fn into_range_iter(self, iter: I) -> Self::Output {
		iter.take(self.end + 1)
	}
}

impl<I> IntoRangeIter<I> for RangeFrom<usize>
	where I: Iterator
{
	type Output = Skip<I>;

	fn into_range_iter(self, iter: I) -> Self::Output {
		iter.skip(self.start)
	}
}

impl<I> IntoRangeIter<I> for RangeFull
	where I: Iterator
{
	type Output = I;

	fn into_range_iter(self, iter: I) -> Self::Output {
		iter
	}
}

impl<I> IntoRangeIter<I> for usize
	where I: Iterator
{
	type Output = Option<I::Item>;

	fn into_range_iter(self, mut iter: I) -> Self::Output {
		iter.nth(self)
	}
}

/// Limits an iterator to a range. See [`Itertools::range`]
/// for more information.
pub fn range<I, R>(iter: I, range: R)
    -> R::Output
    where I: IntoIterator,
          R: IntoRangeIter<I::IntoIter>
{
	range.into_range_iter(iter.into_iter())
}
