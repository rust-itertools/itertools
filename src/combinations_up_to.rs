use crate::{Combinations, Itertools};
use std::fmt;
use std::iter::FusedIterator;

/// An iterator to iterate through all the up to `k`-length combinations in an iterator.
///
/// See [`.combinations_up_to()`](Itertools::combinations_up_to) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct CombinationsUpTo<I: Iterator> {
    iter: Combinations<I>,
    orig_iter: I,
    end: bool,
}

impl<I> Clone for CombinationsUpTo<I>
    where
        I: Clone + Iterator,
        I::Item: Clone,
{
    clone_fields!(end, k, orig_iter, iter);
}

impl<I> fmt::Debug for CombinationsUpTo<I>
    where
        I: fmt::Debug + Iterator,
        I::Item: fmt::Debug,
{
    debug_fmt_fields!(CombinationsUpTo, end, k, orig_iter, iter);
}

#[inline]
pub fn combinations_up_to<I>(iter: I, k: usize) -> CombinationsUpTo<I>
    where
        I: Iterator + Sized + Clone,
        I::Item: Clone,
{
    CombinationsUpTo {
        iter: iter.clone().combinations(k),
        orig_iter: iter,
        end: false,
    }
}

impl<I> Iterator for CombinationsUpTo<I>
    where
        I: Iterator + Clone,
        I::Item: Clone,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            None
        } else if let Some(result) = self.iter.next() {
            if result.is_empty() {
                self.end = true;
            }
            Some(result)
        } else {
            self.iter = self.orig_iter.clone().combinations(self.iter.k() - 1);
            let result = self.iter.next();
            if let Some(result) = &result {
                if result.is_empty() {
                    self.end = true;
                }
            } else {
                self.end = true;
            }
            result
        }
    }
}

impl<I> FusedIterator for CombinationsUpTo<I>
    where
        I: Iterator + Clone,
        I::Item: Clone,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combinations_up_to_0() {
        itertools::assert_equal([()].into_iter().combinations_up_to(0), vec![vec![]]);
    }
}
