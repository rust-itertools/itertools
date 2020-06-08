use core::ops::{ Range, RangeTo, RangeFrom, RangeFull, RangeInclusive, RangeToInclusive };
use core::iter::{Skip, Take};
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
	impl Sealed for usize {}
}

/// Used by the ``range`` function to know which iterator
/// to turn different ranges into.
pub trait IterIndex<T> : private_iter_index::Sealed {
	type Output;

	/// Returns an iterator(or value) in the specified range.
	///
	/// Prefer calling [`range`] or [`Itertools::range`] instead
	/// of calling this directly.
	fn get(self, from: T) -> Self::Output;
}

impl<I> IterIndex<I> for Range<usize>
	where I: Iterator
{
	type Output = Take<Skip<I>>;

	fn get(self, iter: I) -> Self::Output {
		iter.skip(self.start)
			.take(self.end.saturating_sub(self.start))
	}
}

impl<I> IterIndex<I> for RangeInclusive<usize>
	where I: Iterator
{
	type Output = Take<Skip<I>>;

	fn get(self, iter: I) -> Self::Output {
		iter.skip(*self.start())
			.take(
				(1 + *self.end())
				.saturating_sub(*self.start())
			)
	}
}

impl<I> IterIndex<I> for RangeTo<usize>
	where I: Iterator
{
	type Output = Take<I>;

	fn get(self, iter: I) -> Self::Output {
		iter.take(self.end)
	}
}

impl<I> IterIndex<I> for RangeToInclusive<usize>
	where I: Iterator
{
	type Output = Take<I>;

	fn get(self, iter: I) -> Self::Output {
		iter.take(self.end + 1)
	}
}

impl<I> IterIndex<I> for RangeFrom<usize>
	where I: Iterator
{
	type Output = Skip<I>;

	fn get(self, iter: I) -> Self::Output {
		iter.skip(self.start)
	}
}

impl<I> IterIndex<I> for RangeFull
	where I: Iterator
{
	type Output = I;

	fn get(self, iter: I) -> Self::Output {
		iter
	}
}

impl<I> IterIndex<I> for usize
	where I: Iterator
{
	type Output = Option<I::Item>;

	fn get(self, mut iter: I) -> Self::Output {
		iter.nth(self)
	}
}

/// Limits an iterator to a range. See [`Itertools::range`]
/// for more information.
pub fn get<I, R>(iter: I, range: R)
    -> R::Output
    where I: IntoIterator,
          R: IterIndex<I::IntoIter>
{
	range.get(iter.into_iter())
}
