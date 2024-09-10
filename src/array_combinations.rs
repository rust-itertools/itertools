use core::iter::FusedIterator;
use core::{array, fmt};

use crate::combinations::{n_and_count, remaining_for};
use crate::lazy_buffer::LazyBuffer;

/// An iterator to iterate through all combinations in an iterator of `Clone`-able items that
/// produces arrays of a specific size.
///
/// See [`.array_combinations()`](crate::Itertools::array_combinations) for more
/// information.
#[derive(Clone)]
#[must_use = "this iterator adaptor is not lazy but does nearly nothing unless consumed"]
pub struct ArrayCombinations<I: Iterator, const K: usize>
where
    I::Item: Clone,
{
    indices: [usize; K],
    pool: LazyBuffer<I>,
    first: bool,
}

/// Create a new `ArrayCombinations` from a clonable iterator.
pub fn array_combinations<I: Iterator, const K: usize>(iter: I) -> ArrayCombinations<I, K>
where
    I::Item: Clone,
{
    let indices = array::from_fn(|i| i);
    let pool = LazyBuffer::new(iter);

    ArrayCombinations {
        indices,
        pool,
        first: true,
    }
}

impl<I: Iterator, const K: usize> ArrayCombinations<I, K>
where
    I::Item: Clone,
{
    /// Returns the (current) length of the pool from which combination elements are
    /// selected. This value can change between invocations of [`next`](Combinations::next).
    #[inline]
    pub fn n(&self) -> usize {
        self.pool.len()
    }

    /// Initialises the iterator by filling a buffer with elements from the
    /// iterator. Returns true if there are no combinations, false otherwise.
    fn init(&mut self) -> bool {
        self.pool.prefill(K);
        let done = K > self.n();
        if !done {
            self.first = false;
        }

        done
    }

    /// Increments indices representing the combination to advance to the next
    /// (in lexicographic order by increasing sequence) combination. For example
    /// if we have n=4 & k=2 then `[0, 1] -> [0, 2] -> [0, 3] -> [1, 2] -> ...`
    ///
    /// Returns true if we've run out of combinations, false otherwise.
    fn increment_indices(&mut self) -> bool {
        if K == 0 {
            return true; // Done
        }

        // Scan from the end, looking for an index to increment
        let mut i: usize = K - 1;

        // Check if we need to consume more from the iterator
        if self.indices[i] == self.pool.len() - 1 {
            _ = self.pool.get_next(); // may change pool size
        }

        while self.indices[i] == i + self.pool.len() - K {
            if i > 0 {
                i -= 1;
            } else {
                // Reached the last combination
                return true;
            }
        }

        // Increment index, and reset the ones to its right
        self.indices[i] += 1;
        for j in i + 1..K {
            self.indices[j] = self.indices[j - 1] + 1;
        }

        // If we've made it this far, we haven't run out of combos
        false
    }

    /// Returns the n-th item or the number of successful steps.
    pub(crate) fn try_nth(&mut self, n: usize) -> Result<<Self as Iterator>::Item, usize>
    where
        I::Item: Clone,
    {
        let done = if self.first {
            self.init()
        } else {
            self.increment_indices()
        };
        if done {
            return Err(0);
        }
        for i in 0..n {
            if self.increment_indices() {
                return Err(i + 1);
            }
        }
        Ok(self.pool.get_array(self.indices))
    }
}

impl<I: Iterator, const K: usize> Iterator for ArrayCombinations<I, K>
where
    I::Item: Clone,
{
    type Item = [I::Item; K];

    fn next(&mut self) -> Option<Self::Item> {
        let done = if self.first {
            self.init()
        } else {
            self.increment_indices()
        };

        (!done).then(|| self.pool.get_array(self.indices))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.try_nth(n).ok()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (mut low, mut upp) = self.pool.size_hint();
        low = remaining_for(low, self.first, &self.indices).unwrap_or(usize::MAX);
        upp = upp.and_then(|upp| remaining_for(upp, self.first, &self.indices));
        (low, upp)
    }

    #[inline]
    fn count(self) -> usize {
        n_and_count(self.pool, self.first, &self.indices).1
    }
}

impl<I, const K: usize> fmt::Debug for ArrayCombinations<I, K>
where
    I: Iterator + fmt::Debug,
    I::Item: Clone + fmt::Debug,
{
    debug_fmt_fields!(ArrayCombinations, indices, pool, first);
}

impl<I, const K: usize> FusedIterator for ArrayCombinations<I, K>
where
    I: FusedIterator,
    I::Item: Clone,
{
}
