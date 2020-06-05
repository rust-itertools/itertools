use core::ops::{ RangeBounds, Bound };
use core::iter::FusedIterator;
use crate::Itertools;

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
}

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
