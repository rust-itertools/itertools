use alloc::vec::Vec;
use std::fmt;
use std::iter::FusedIterator;

use super::lazy_buffer::LazyBuffer;
use crate::adaptors::checked_binomial;

/// An iterator to iterate through all the `n`-length combinations in an iterator, with replacement.
///
/// See [`.combinations_with_replacement()`](crate::Itertools::combinations_with_replacement)
/// for more information.
#[derive(Clone)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct CombinationsWithReplacement<I>
where
    I: Iterator,
    I::Item: Clone,
{
    indices: Vec<usize>,
    pool: LazyBuffer<I>,
    first: bool,
}

impl<I> fmt::Debug for CombinationsWithReplacement<I>
where
    I: Iterator + fmt::Debug,
    I::Item: fmt::Debug + Clone,
{
    debug_fmt_fields!(CombinationsWithReplacement, indices, pool, first);
}

impl<I> CombinationsWithReplacement<I>
where
    I: Iterator,
    I::Item: Clone,
{
    /// Map the current mask over the pool to get an output combination
    fn current(&self) -> Vec<I::Item> {
        self.indices.iter().map(|i| self.pool[*i].clone()).collect()
    }
}

/// Create a new `CombinationsWithReplacement` from a clonable iterator.
pub fn combinations_with_replacement<I>(iter: I, k: usize) -> CombinationsWithReplacement<I>
where
    I: Iterator,
    I::Item: Clone,
{
    let indices: Vec<usize> = alloc::vec![0; k];
    let pool: LazyBuffer<I> = LazyBuffer::new(iter);

    CombinationsWithReplacement {
        indices,
        pool,
        first: true,
    }
}

impl<I> Iterator for CombinationsWithReplacement<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = Vec<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        // If this is the first iteration, return early
        if self.first {
            // In empty edge cases, stop iterating immediately
            return if !(self.indices.is_empty() || self.pool.get_next()) {
                None
            // Otherwise, yield the initial state
            } else {
                self.first = false;
                Some(self.current())
            };
        }

        // Check if we need to consume more from the iterator
        // This will run while we increment our first index digit
        self.pool.get_next();

        // Work out where we need to update our indices
        let mut increment: Option<(usize, usize)> = None;
        for (i, indices_int) in self.indices.iter().enumerate().rev() {
            if *indices_int < self.pool.len() - 1 {
                increment = Some((i, indices_int + 1));
                break;
            }
        }

        match increment {
            // If we can update the indices further
            Some((increment_from, increment_value)) => {
                // We need to update the rightmost non-max value
                // and all those to the right
                for indices_index in increment_from..self.indices.len() {
                    self.indices[indices_index] = increment_value;
                }
                Some(self.current())
            }
            // Otherwise, we're done
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (mut low, mut upp) = self.pool.size_hint();
        low = remaining_for(low, self.first, &self.indices).unwrap_or(usize::MAX);
        upp = upp.and_then(|upp| remaining_for(upp, self.first, &self.indices));
        (low, upp)
    }

    fn count(self) -> usize {
        let Self {
            indices,
            pool,
            first,
        } = self;
        let n = pool.count();
        remaining_for(n, first, &indices).unwrap()
    }
}

impl<I> FusedIterator for CombinationsWithReplacement<I>
where
    I: Iterator,
    I::Item: Clone,
{
}

/// For a given size `n`, return the count of remaining combinations with replacement or None if it would overflow.
fn remaining_for(n: usize, first: bool, indices: &[usize]) -> Option<usize> {
    // With a "stars and bars" representation, choose k values with replacement from n values is
    // like choosing k out of k + n − 1 positions (hence binomial(k + n - 1, k) possibilities)
    // to place k stars and therefore n - 1 bars.
    // Example (n=4, k=6): ***|*||** represents [0,0,0,1,3,3].
    let count = |n: usize, k: usize| {
        let positions = if n == 0 {
            k.saturating_sub(1)
        } else {
            (n - 1).checked_add(k)?
        };
        checked_binomial(positions, k)
    };
    let k = indices.len();
    if first {
        count(n, k)
    } else {
        // The algorithm is similar to the one for combinations *without replacement*,
        // except we choose values *with replacement* and indices are *non-strictly* monotonically sorted.

        // The combinations generated after the current one can be counted by counting as follows:
        // - The subsequent combinations that differ in indices[0]:
        //   If subsequent combinations differ in indices[0], then their value for indices[0]
        //   must be at least 1 greater than the current indices[0].
        //   As indices is monotonically sorted, this means we can effectively choose k values with
        //   replacement from (n - 1 - indices[0]), leading to count(n - 1 - indices[0], k) possibilities.
        // - The subsequent combinations with same indices[0], but differing indices[1]:
        //   Here we can choose k - 1 values with replacement from (n - 1 - indices[1]) values,
        //   leading to count(n - 1 - indices[1], k - 1) possibilities.
        // - (...)
        // - The subsequent combinations with same indices[0..=i], but differing indices[i]:
        //   Here we can choose k - i values with replacement from (n - 1 - indices[i]) values: count(n - 1 - indices[i], k - i).
        //   Since subsequent combinations can in any index, we must sum up the aforementioned binomial coefficients.

        // Below, `n0` resembles indices[i].
        indices.iter().enumerate().try_fold(0usize, |sum, (i, n0)| {
            sum.checked_add(count(n - 1 - *n0, k - i)?)
        })
    }
}
