use std::fmt;

use super::lazy_buffer::LazyBuffer;

/// An iterator to iterate through all the `n`-length combinations in an iterator.
/// It works even for infinite iterators.
///
/// See [`combinations_by_len()`](../trait.Itertools.html#fn.combinations_by_len) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct CombinationsByLen<I: Iterator> {
    n: usize,
    indices: Vec<usize>,
    pool: LazyBuffer<I>,
    first: bool,
}

impl<I> CombinationsByLen<I>
    where I: Iterator,
          I::Item: Clone
{
    // Create result vector based on the indices
    fn result(&self) -> Vec<I::Item> {
        self.indices.iter()
                    .map(|&i| self.pool[i].clone())
                    .collect()
    }
}

impl<I> fmt::Debug for CombinationsByLen<I>
    where I: Iterator + fmt::Debug,
          I::Item: fmt::Debug
{
    debug_fmt_fields!(Combinations, n, indices, pool, first);
}

/// Create a new `Combinations` from a clonable iterator.
pub fn combinations_by_len<I>(iter: I, n: usize) -> CombinationsByLen<I>
    where I: Iterator
{
    let mut pool: LazyBuffer<I> = LazyBuffer::new(iter);
    for _ in 0..n {
        if !pool.get_next() {
            break;
        }
    }

    CombinationsByLen { n,
                        indices: (0..n).collect(),
                        pool,
                        first: true }
}

impl<I> Iterator for CombinationsByLen<I>
    where I: Iterator,
          I::Item: Clone
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let pool_len = self.pool.len();
        if (pool_len == 0 || self.n > pool_len) && !self.pool.get_next() {
            return None;
        } else if self.first {
            self.first = false;
            return Some(self.result());
        } else if self.n == 0 {
            return None;
        }

        // Scan from the end, looking for an index to increment
        let mut i: usize = self.n - 1;

        while self.indices[i] == i + pool_len - self.n {
            if i == 0 {
                if !self.pool.get_next() {
                    // Reached the last combination
                    return None;
                }
                i = self.n - 1;
                for j in 0..i {
                    self.indices[j] = j;
                }
                break;
            }
            i -= 1;
        }

        // Increment index, and reset the ones to its right
        self.indices[i] += 1;
        for j in i + 1..self.n - 1 {
            self.indices[j] = self.indices[j - 1] + 1
        }
        Some(self.result())
    }
}
