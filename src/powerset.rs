use std::fmt;
use alloc::vec::Vec;

use super::combinations::{Combinations, combinations};

/// An iterator to iterate through the powerset of the elements from an iterator.
///
/// See [`.powerset()`](../trait.Itertools.html#method.powerset) for more
/// information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Powerset<I: Iterator> {
    combs: Combinations<I>,
}

impl<I> Clone for Powerset<I>
    where I: Clone + Iterator,
          I::Item: Clone,
{
    clone_fields!(combs);
}

impl<I> fmt::Debug for Powerset<I>
    where I: Iterator + fmt::Debug,
          I::Item: fmt::Debug,
{
    debug_fmt_fields!(Powerset, combs);
}

/// Create a new `Powerset` from a clonable iterator.
pub fn powerset<I>(src: I) -> Powerset<I>
    where I: Iterator,
          I::Item: Clone,
{
    Powerset { combs: combinations(src, 0) }
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
        } else if self.combs.k() < self.combs.n()
            || self.combs.k() == 0
        {
            self.combs.reset(self.combs.k() + 1);
            self.combs.next()
        } else {
            None
        }
    }
}
