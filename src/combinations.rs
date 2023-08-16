use std::fmt;
use std::iter::{Fuse, FusedIterator};

use super::lazy_buffer::LazyBuffer;
use alloc::vec::Vec;

/// An iterator to iterate through all the `k`-length combinations in an iterator.
///
/// See [`.combinations()`](crate::Itertools::combinations) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Combinations<I: Iterator> {
    indices: Vec<usize>,
    pool: LazyBuffer<I>,
    first: bool,
}

impl<I> Clone for Combinations<I>
    where I: Clone + Iterator,
          I::Item: Clone,
{
    clone_fields!(indices, pool, first);
}

impl<I> fmt::Debug for Combinations<I>
    where I: Iterator + fmt::Debug,
          I::Item: fmt::Debug,
{
    debug_fmt_fields!(Combinations, indices, pool, first);
}

/// Create a new `Combinations` from a clonable iterator.
pub fn combinations<I>(iter: I, k: usize) -> Combinations<I>
    where I: Iterator
{
    let mut pool = LazyBuffer::new(iter);
    pool.prefill(k);

    Combinations {
        indices: (0..k).collect(),
        pool,
        first: true,
    }
}

impl<I: Iterator> Combinations<I> {
    /// Returns the length of a combination produced by this iterator.
    #[inline]
    pub fn k(&self) -> usize { self.indices.len() }

    /// Returns the (current) length of the pool from which combination elements are
    /// selected. This value can change between invocations of [`next`](Combinations::next).
    #[inline]
    pub fn n(&self) -> usize { self.pool.len() }

    /// Returns a reference to the source iterator.
    #[inline]
    pub(crate) fn src(&self) -> &Fuse<I> { &self.pool.it }

    /// Resets this `Combinations` back to an initial state for combinations of length
    /// `k` over the same pool data source. If `k` is larger than the current length
    /// of the data pool an attempt is made to prefill the pool so that it holds `k`
    /// elements.
    pub(crate) fn reset(&mut self, k: usize) {
        self.first = true;

        if k < self.indices.len() {
            self.indices.truncate(k);
            for i in 0..k {
                self.indices[i] = i;
            }

        } else {
            for i in 0..self.indices.len() {
                self.indices[i] = i;
            }
            self.indices.extend(self.indices.len()..k);
            self.pool.prefill(k);
        }
    }

    /// For a given size `n`, return the count of remaining elements or None if it would overflow.
    fn remaining_for(&self, n: usize) -> Option<usize> {
        let k = self.k();
        if self.first {
            checked_binomial(n, k)
        } else {
            self.indices
                .iter()
                .enumerate()
                .fold(Some(0), |sum, (k0, n0)| {
                    sum.and_then(|s| s.checked_add(checked_binomial(n - 1 - *n0, k - k0)?))
                })
        }
    }
}

impl<I> Iterator for Combinations<I>
    where I: Iterator,
          I::Item: Clone
{
    type Item = Vec<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            if self.k() > self.n() {
                return None;
            }
            self.first = false;
        } else if self.indices.is_empty() {
            return None;
        } else {
            // Scan from the end, looking for an index to increment
            let mut i: usize = self.indices.len() - 1;

            // Check if we need to consume more from the iterator
            if self.indices[i] == self.pool.len() - 1 {
                self.pool.get_next(); // may change pool size
            }

            while self.indices[i] == i + self.pool.len() - self.indices.len() {
                if i > 0 {
                    i -= 1;
                } else {
                    // Reached the last combination
                    return None;
                }
            }

            // Increment index, and reset the ones to its right
            self.indices[i] += 1;
            for j in i+1..self.indices.len() {
                self.indices[j] = self.indices[j - 1] + 1;
            }
        }

        // Create result vector based on the indices
        Some(self.indices.iter().map(|i| self.pool[*i].clone()).collect())
    }
}

impl<I> FusedIterator for Combinations<I>
    where I: Iterator,
          I::Item: Clone
{}

pub(crate) fn checked_binomial(mut n: usize, mut k: usize) -> Option<usize> {
    if n < k {
        return Some(0);
    }
    // `factorial(n) / factorial(n - k) / factorial(k)` but trying to avoid it overflows:
    k = (n - k).min(k); // symmetry
    let mut c = 1;
    for i in 1..=k {
        c = (c / i).checked_mul(n)?.checked_add((c % i).checked_mul(n)? / i)?;
        n -= 1;
    }
    Some(c)
}
