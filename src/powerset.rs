use alloc::vec::Vec;
use std::fmt;
use std::iter::FusedIterator;
use std::usize;

use super::combinations::{checked_binomial, combinations, Combinations};
use crate::size_hint::{self, SizeHint};

/// An iterator to iterate through the powerset of the elements from an iterator.
///
/// See [`.powerset()`](crate::Itertools::powerset) for more
/// information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Powerset<I: Iterator> {
    combs: Combinations<I>,
}

impl<I> Clone for Powerset<I>
where
    I: Clone + Iterator,
    I::Item: Clone,
{
    clone_fields!(combs);
}

impl<I> fmt::Debug for Powerset<I>
where
    I: Iterator + fmt::Debug,
    I::Item: fmt::Debug,
{
    debug_fmt_fields!(Powerset, combs);
}

/// Create a new `Powerset` from a clonable iterator.
pub fn powerset<I>(src: I) -> Powerset<I>
where
    I: Iterator,
    I::Item: Clone,
{
    Powerset {
        combs: combinations(src, 0),
    }
}

impl<I> Iterator for Powerset<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(elt) = self.combs.next() {
            Some(elt)
        } else if self.combs.k() < self.combs.n() || self.combs.k() == 0 {
            self.combs.reset(self.combs.k() + 1);
            self.combs.next()
        } else {
            None
        }
    }

    fn size_hint(&self) -> SizeHint {
        let k = self.combs.k();
        // Total bounds for source iterator.
        let (n_min, n_max) = self.combs.src().size_hint();
        let low = remaining_for(n_min, k).unwrap_or(usize::MAX);
        let upp = n_max.and_then(|n| remaining_for(n, k));
        size_hint::add(self.combs.size_hint(), (low, upp))
    }

    fn count(self) -> usize {
        let k = self.combs.k();
        let (n, combs_count) = self.combs.n_and_count();
        combs_count + remaining_for(n, k).unwrap()
    }
}

impl<I> FusedIterator for Powerset<I>
where
    I: Iterator,
    I::Item: Clone,
{
}

fn remaining_for(n: usize, k: usize) -> Option<usize> {
    (k + 1..=n).try_fold(0usize, |sum, i| sum.checked_add(checked_binomial(n, i)?))
}
