use alloc::vec::Vec;
use std::fmt;
use std::iter::FusedIterator;

use super::lazy_buffer::LazyBuffer;
use crate::combinations::checked_binomial;

/// An iterator to iterate through all the `n`-length combinations in an iterator, with replacement.
///
/// See [`.combinations_with_replacement()`](crate::Itertools::combinations_with_replacement)
/// for more information.
#[derive(Clone)]
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
    debug_fmt_fields!(Combinations, indices, pool, first);
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
            if *indices_int < self.pool.len()-1 {
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
}

impl<I> FusedIterator for CombinationsWithReplacement<I>
where
    I: Iterator,
    I::Item: Clone,
{}

/// For a given size `n`, return the count of remaining combinations with replacement or None if it would overflow.
fn remaining_for(n: usize, first: bool, indices: &[usize]) -> Option<usize> {
    let count = |n: usize, k: usize| checked_binomial((n + k).saturating_sub(1), k);
    let k = indices.len();
    if first {
        count(n, k)
    } else {
        indices
            .iter()
            .enumerate()
            // TODO: Once the MSRV hits 1.37.0, we can sum options instead:
            // .map(|(i, n0)| count(n - 1 - *n0, k - i))
            // .sum()
            .fold(Some(0), |sum, (i, n0)| {
                sum.and_then(|s| s.checked_add(count(n - 1 - *n0, k - i)?))
            })
    }
}
