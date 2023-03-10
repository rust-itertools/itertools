use std::fmt;
use std::iter::FusedIterator;

use super::lazy_buffer::LazyBuffer;
use alloc::vec::Vec;

/// Marker indicating the iterator is being used as `LendingIterator` type.
#[cfg(feature = "lending_iters")]
pub struct Lending;
/// Marker indicating the iterator is being used as `Iterator` type.
pub struct NonLending;

/// An iterator to iterate through all the `k`-length combinations in an iterator.
///
/// See [`.combinations()`](crate::Itertools::combinations) and [`.combinations_lending()`](crate::Itertools::combinations_lending) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Combinations<I: Iterator, State = NonLending> {
    indices: Vec<usize>,
    pool: LazyBuffer<I>,
    first: bool,
    // Disambiguate the purpose of the iterator and makes use not require fully qualified path to disambiguate. Instead chosen by constructor.
    phantom: std::marker::PhantomData<State>,
}

impl<I> Clone for Combinations<I>
where
    I: Clone + Iterator,
    I::Item: Clone,
{
    clone_fields!(indices, pool, first, phantom);
}

impl<State, I> fmt::Debug for Combinations<I, State>
where
    I: Iterator + fmt::Debug,
    I::Item: fmt::Debug,
{
    debug_fmt_fields!(Combinations, indices, pool, first);
}

/// Create a new `Combinations` `Iterator` from an iterator.
pub fn combinations<I>(iter: I, k: usize) -> Combinations<I>
where
    I: Iterator,
{
    let mut pool = LazyBuffer::new(iter);
    pool.prefill(k);

    Combinations {
        indices: (0..k).collect(),
        pool,
        first: true,
        phantom: std::marker::PhantomData::<NonLending>,
    }
}

impl<I: Iterator, State> Combinations<I, State> {
    /// Returns the length of a combination produced by this iterator.
    #[inline]
    pub fn k(&self) -> usize {
        self.indices.len()
    }

    /// Returns the (current) length of the pool from which combination elements are
    /// selected. This value can change between invocations of [`next`](Combinations::next).
    #[inline]
    pub fn n(&self) -> usize {
        self.pool.len()
    }

    /// Returns a reference to the source iterator.
    #[inline]
    #[allow(dead_code)] // Not actually dead. Used in powerset.
    pub(crate) fn src(&self) -> &I {
        &self.pool.it
    }

    /// Resets this `Combinations` back to an initial state for combinations of length
    /// `k` over the same pool data source. If `k` is larger than the current length
    /// of the data pool an attempt is made to prefill the pool so that it holds `k`
    /// elements.
    #[allow(dead_code)] // Not actually dead. Used in powerset.
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
}

impl<I> Iterator for Combinations<I, NonLending>
where
    I: Iterator,
    I::Item: Clone,
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
            for j in i + 1..self.indices.len() {
                self.indices[j] = self.indices[j - 1] + 1;
            }
        }

        // Create result vector based on the indices. If there is a combination it is always of length k.
        let mut out = Vec::with_capacity(self.k());
        out.extend(self.indices.iter().map(|i| self.pool[*i].clone()));
        Some(out)
    }
}

impl<I> FusedIterator for Combinations<I>
where
    I: Iterator,
    I::Item: Clone,
{
}

#[cfg(feature = "lending_iters")]
pub mod lending {
    use super::*;
    pub use lending_iterator::prelude::{gat, LendingIterator, LendingIteratorඞItem};

    /// Create a new `Combinations` `LendingIterator` from an iterator.
    pub fn combinations_lending<I>(iter: I, k: usize) -> Combinations<I, Lending>
    where
        I: Iterator,
    {
        let mut pool = LazyBuffer::new(iter);
        pool.prefill(k);

        Combinations {
            indices: (0..k).collect(),
            pool,
            first: true,
            phantom: std::marker::PhantomData::<Lending>,
        }
    }

    #[gat]
    impl<I> LendingIterator for Combinations<I, Lending>
    where
        I: Iterator,
        I::Item: Clone,
    {
        type Item<'next>
        where
            Self: 'next,
        = Combination<'next, I>;
        fn next(&mut self) -> Option<Combination<I>> {
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
                for j in i + 1..self.indices.len() {
                    self.indices[j] = self.indices[j - 1] + 1;
                }
            }

            // Create result vector based on the indices
            // let out: () = Some(self.indices.iter().map(|i| self.pool[*i].clone()));
            Some(Combination {
                combinations: &*self,
                index: 0,
            })
        }
    }

    impl<I> Combinations<I, Lending>
    where
        I: Iterator,
        I::Item: Clone,
    {
        /// Applies `collect_vec()` on interior iterators and then again on the result.
        #[cfg(feature = "use_alloc")]
        pub fn collect_nested_vec(self) -> Vec<Vec<I::Item>>
        where
            Self: Sized,
        {
            use crate::Itertools;

            self.map_into_iter(|x| x.collect_vec()).collect_vec()
        }
    }

    // TODO Should take precedence over LendingIterator blanket impl for IntoIterator. How to do?
    // Appears to works correctly given sufficient type hints/context such as a for loop.
    impl<I> IntoIterator for Combinations<I, Lending>
    where
        I: Iterator,
        I::Item: Clone,
    {
        type Item = Vec<I::Item>;

        type IntoIter = Combinations<I, NonLending>;

        /// The phantom marker changing is sufficient to change this into an iterator because it implements `Iterator` as well and `Lendingiterator`
        #[inline]
        fn into_iter(self) -> Self::IntoIter {
            Combinations {
                indices: self.indices,
                pool: self.pool,
                first: self.first,
                phantom: core::marker::PhantomData::<NonLending>,
            }
        }
    }

    /// Iterator over the elements of a particular combination. This allows avoiding unnecessary heap allocations if the use of the combinations is not a `Vec`.
    #[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
    #[derive(Clone)]
    pub struct Combination<'a, I>
    where
        I: Iterator,
        I::Item: Clone,
    {
        combinations: &'a Combinations<I, Lending>,
        index: usize, // Index of the combinations indices
    }

    impl<'a, I> fmt::Debug for Combination<'a, I>
    where
        I: Iterator + fmt::Debug,
        I::Item: fmt::Debug,
        I::Item: Clone,
    {
        // Allows implementing Debug for items that implement Debug without requiring Debug to use the iterator.
        debug_fmt_fields!(Combination, combinations, index);
    }

    impl<'a, I> Iterator for Combination<'a, I>
    where
        I: Iterator,
        I::Item: Clone,
    {
        type Item = I::Item;

        // Simply increment through the indices that fetch values from the pool.
        fn next(&mut self) -> Option<Self::Item> {
            if self.index >= self.combinations.indices.len() {
                None
            } else {
                self.index += 1;
                Some(self.combinations.pool[self.combinations.indices[self.index - 1]].clone())
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            // If a combination is returned it is always of length k.
            (self.combinations.k(), Some(self.combinations.k()))
        }
    }
}
