use core::iter::FusedIterator;
use std::fmt;

/// An iterator adaptor that consumes elements while the given predicate is false, including the
/// element for which the predicate first returned true.
///
/// See [`.take_until()`](crate::Itertools::take_until) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct TakeUntil<'a, I: 'a, F> {
    iter: &'a mut I,
    f: F,
    done: bool,
}

impl<'a, I, F> TakeUntil<'a, I, F>
where
    I: Iterator,
    F: FnMut(&I::Item) -> bool,
{
    /// Create a new [`TakeUntil`] from an iterator and a predicate.
    pub fn new(iter: &'a mut I, f: F) -> Self {
        Self { iter, f, done: false}
    }
}

impl<'a, I, F> fmt::Debug for TakeUntil<'a, I, F>
    where I: Iterator + fmt::Debug,
{
    debug_fmt_fields!(TakeUntil, iter);
}

impl<'a, I, F> Iterator for TakeUntil<'a, I, F>
where
    I: Iterator,
    F: FnMut(&I::Item) -> bool
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.iter.next().map(|item| {
                if (self.f)(&item) {
                    self.done = true;
                }
                item
            })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            (0, Some(0))
        } else {
            (0, self.iter.size_hint().1)
        }
    }
}

impl<I, F> FusedIterator for TakeUntil<'_, I, F>
where
    I: Iterator,
    F: FnMut(&I::Item) -> bool
{
}