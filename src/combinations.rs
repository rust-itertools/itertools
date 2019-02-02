use std::fmt;

use super::lazy_buffer::LazyBuffer;

/// An iterator to iterate through all the `k`-length combinations in an iterator.
/// Note: it iterates over combinations in lexicographic order and
/// thus may not work as expected with infinite iterators.
///
/// See [`.combinations()`](../trait.Itertools.html#method.combinations) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Combinations<I: Iterator> {
    k: usize,
    indices: Vec<usize>,
    pool: LazyBuffer<I>,
    first: bool,
}

impl<I> Clone for Combinations<I>
    where I: Clone + Iterator,
          I::Item: Clone,
{
    clone_fields!(k, indices, pool, first);
}

impl<I: Iterator> Combinations<I> {
    fn advance(&mut self) {
        let pool_len = self.pool.len();

        // Scan from the end, looking for an index to increment
        let mut i = self.k - 1;
        while self.indices[i] + self.k == i + pool_len {
            i -= 1;
        }

        // Increment index, and reset the ones to its right
        self.indices[i] += 1;
        for j in i + 1..self.k {
            self.indices[j] = self.indices[j - 1] + 1;
        }
    }
}

impl<I> fmt::Debug for Combinations<I>
    where I: Iterator + fmt::Debug,
          I::Item: fmt::Debug
{
    debug_fmt_fields!(Combinations, k, indices, pool, first);
}

/// Create a new `Combinations` from a clonable iterator.
pub fn combinations<I>(iter: I, k: usize) -> Combinations<I>
    where I: Iterator
{
    let indices: Vec<usize> = (0..k).collect();
    let mut pool: LazyBuffer<I> = LazyBuffer::new(iter);

    for _ in 0..k {
        if !pool.get_next() {
            break;
        }
    }

    Combinations {
        k: k,
        indices: indices,
        pool: pool,
        first: true,
    }
}

impl<I> Iterator for Combinations<I>
    where I: Iterator,
          I::Item: Clone
{
    type Item = Vec<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut pool_len = self.pool.len();

        if self.first {
            if self.pool.is_done() {
                return None;
            }
            self.first = false;
        } else {
            if self.k == 0 {
                return None;
            }

            // Check if we need to consume more from the iterator
            if self.indices[self.k - 1] == pool_len - 1 && self.pool.get_next() {
                pool_len += 1;
            }

            if self.indices[0] == pool_len - self.k {
                return None;
            }

            self.advance();
        }

        // Create result vector based on the indices
        let mut result = Vec::with_capacity(self.k);
        for i in self.indices.iter() {
            result.push(self.pool[*i].clone());
        }
        Some(result)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n == 0 {
            return self.next();
        }
        
        let mut pool_len = self.pool.len();
        if self.k == 0 || self.pool.is_done() && (pool_len == 0 || self.k > pool_len) {
            return None;
        }

        let mut n = n;
        if self.first {
            self.first = false;
        } else { 
            n += 1;
        }

        // Drain iterator and increase last index.
        while n > 0 && self.pool.get_next() {
            self.indices[self.k - 1] += 1;
            pool_len += 1;
            n -= 1;
        }

        for _ in 0..n {    
            // check if we have reached the end            
            if self.indices[0] == pool_len - self.k {
                return None;
            }
            self.advance();
        }

        // Create result vector based on the indices
        let mut result = Vec::with_capacity(self.k);
        for i in self.indices.iter() {
            result.push(self.pool[*i].clone());
        }
        Some(result)
    }
}
