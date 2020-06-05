use core::ops::{ RangeBounds, Bound };
use core::iter::{ExactSizeIterator, FusedIterator};
use crate::Itertools;

// We may want to allow this to become a double ended
// iterator if the thing it's iterating over is double ended,
// and if the range has a specified end point.

/// An iterator over a range of values.
///
/// Acquired by the [`range`] function or the
/// [`Itertools::range`] method.
pub struct Range<I, R> {
    range: R,
    internal: I,
    counter: usize,
}

impl<I, R> Clone for Range<I, R> 
    where I: Clone, R: Clone 
{
    fn clone(&self) -> Self {
        Range {
            counter: self.counter,
            range: self.range.clone(),
            internal: self.internal.clone(),
        }
    }
}

impl<T, I, R> Iterator for Range<I, R>
    where I: Iterator<Item = T>, 
          R: RangeBounds<usize> 
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter == 0 {
            match self.range.start_bound() {
                Bound::Included(&n) => {
                    (0..n).for_each(|_| { self.internal.next(); });
                    self.counter = n;
                },
                Bound::Excluded(&n) => {
                    (0..=n).for_each(|_| { self.internal.next(); });
                    self.counter = n + 1;
                },
                Bound::Unbounded => (),
            }
        }

        match self.range.end_bound() {
            Bound::Unbounded => self.internal.next(),
            Bound::Included(&n)  => { 
                if self.counter > n { return None; }

                self.counter += 1;
                self.internal.next()
            },
            Bound::Excluded(&n) => {
                if self.counter >= n { return None; }

                self.counter += 1;
                self.internal.next()
            },
        } 
    }

	/// # Examples
	///
	/// ```
	/// use itertools::Itertools;
	///
	/// assert_eq!(
	///		(0..10).range(2..4).size_hint(),
	///		(2, Some(2))
	///	);
	///
	/// assert_eq!(
	///		(0..10).range(5..15).size_hint(),
	///		(5, Some(5))
	///	);
	///
	/// assert_eq!(
	///		(0..10).range(2..).size_hint(),
	///		(8, Some(8))
	///	);
	///
	/// assert_eq!(
	///		(0..).range(..8).size_hint(),
	///		(8, Some(8))
	///	);
	///
	/// assert_eq!(
	///		(0..).range(..).size_hint(),
	///		(std::usize::MAX, None)
	///	);
	///
	///	assert_eq!(
	///		(0..5).range(..).size_hint(),
	///		(5, Some(5))
	///	);
	///
	///	assert_eq!(
	///		(0..).range(5..10).size_hint(),
	///		(5, Some(5))
	///	);
	///
	///	assert_eq!(
	///		(0..).range(..10).size_hint(),
	///		(10, Some(10))
	///	);
	/// ```
	fn size_hint(&self) -> (usize, Option<usize>) {
		// Absolute mind melt, yes
		let (pre_lower, pre_upper) = self.internal.size_hint();

		let start = match self.range.start_bound() {
			Bound::Included(&n) => n,
			Bound::Excluded(&n) => n + 1,
			Bound::Unbounded => 0,
		};
		let end = match self.range.end_bound() {
			Bound::Included(&n) => Some(n + 1),
			Bound::Excluded(&n) => Some(n),
			Bound::Unbounded => None,
		};

		let lower = match end {
			Some(end) => 
				pre_lower.min(end).saturating_sub(start),
			None => pre_lower.saturating_sub(start),
		};

		let upper = match (end, pre_upper) {
			(Some(end), Some(pre_upper)) => 
				Some(pre_upper.min(end).saturating_sub(start)),
			(Some(end), None) =>
				Some(end - start),
			(None, Some(pre_upper)) =>
				Some(pre_upper.saturating_sub(start)),
			(None, None) => None,
		};

		(lower, upper)
	}
}

impl<T, I, R> ExactSizeIterator for Range<I, R>
where I: Iterator<Item = T> + ExactSizeIterator,
	  R: RangeBounds<usize> {}

impl<T, I, R> FusedIterator for Range<I, R>
where I: Iterator<Item = T> + FusedIterator, 
      R: RangeBounds<usize> {}

/// Limits an iterator to a range. See [`Itertools::range`]
/// for more information.
pub fn range<I, R>(iter: I, range: R)
    -> Range<I, R>
    where I: Iterator,
          R: RangeBounds<usize>
{
    Range {
        internal: iter,
        range: range,
        counter: 0,
    }
}
