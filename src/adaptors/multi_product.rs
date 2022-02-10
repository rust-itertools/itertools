#![cfg(feature = "use_alloc")]

use crate::size_hint;

use alloc::vec::Vec;

#[derive(Clone)]
/// An iterator adaptor that iterates over the cartesian product of
/// multiple iterators of type `I`.
///
/// An iterator element type is `Vec<I>`.
///
/// See [`.multi_cartesian_product()`](crate::Itertools::multi_cartesian_product)
/// for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct MultiProduct<I>
where
    I: Iterator + Clone,
    I::Item: Clone,
{
    state: MultiProductState<I::Item>,
    iters: Vec<MultiProductIter<I>>,
}

impl<I> std::fmt::Debug for MultiProduct<I>
where
    I: Iterator + Clone + std::fmt::Debug,
    I::Item: Clone + std::fmt::Debug,
{
    debug_fmt_fields!(CoalesceBy, iters);
}

/// Stores the current state of the iterator.
#[derive(Clone)]
enum MultiProductState<I> {
    /// In the middle of an iteration. The `Vec<I>` is the last value we returned
    InProgress(Vec<I>),
    /// At the beginning of an iteration. The `Vec<I>` is the next value to be returned.
    Restarted(Vec<I>),
    /// Iteration has not been started
    Unstarted,
}
use MultiProductState::*;

/// Create a new cartesian product iterator over an arbitrary number
/// of iterators of the same type.
///
/// Iterator element is of type `Vec<H::Item::Item>`.
pub fn multi_cartesian_product<H>(iters: H) -> MultiProduct<<H::Item as IntoIterator>::IntoIter>
where
    H: Iterator,
    H::Item: IntoIterator,
    <H::Item as IntoIterator>::IntoIter: Clone,
    <H::Item as IntoIterator>::Item: Clone,
{
    MultiProduct {
        state: MultiProductState::Unstarted,
        iters: iters
            .map(|i| MultiProductIter::new(i.into_iter()))
            .collect(),
    }
}

#[derive(Clone, Debug)]
/// Holds the state of a single iterator within a `MultiProduct`.
struct MultiProductIter<I>
where
    I: Iterator + Clone,
    I::Item: Clone,
{
    iter: I,
    iter_orig: I,
}

impl<I> MultiProductIter<I>
where
    I: Iterator + Clone,
    I::Item: Clone,
{
    fn new(iter: I) -> Self {
        MultiProductIter {
            iter: iter.clone(),
            iter_orig: iter,
        }
    }

    fn next(&mut self) -> Option<I::Item> {
        self.iter.next()
    }
}

impl<I> Iterator for MultiProduct<I>
where
    I: Iterator + Clone,
    I::Item: Clone,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let last = match &mut self.state {
            InProgress(v) => v,
            Restarted(v) => {
                let v = core::mem::replace(v, Vec::new());
                self.state = InProgress(v.clone());
                return Some(v);
            }
            Unstarted => {
                let next: Option<Vec<_>> = self.iters.iter_mut().map(|i| i.next()).collect();
                if let Some(v) = &next {
                    self.state = InProgress(v.clone());
                }
                return next;
            }
        };

        // Starting from the last iterator, advance each iterator until we find one that returns a
        // value.
        for i in (0..self.iters.len()).rev() {
            let iter = &mut self.iters[i];
            let loc = &mut last[i];
            if let Some(val) = iter.next() {
                *loc = val;
                return Some(last.clone());
            } else {
                iter.iter = iter.iter_orig.clone();
                if let Some(val) = iter.next() {
                    *loc = val;
                } else {
                    // This case should not really take place; we had an in progress iterator, reset
                    // it, and called `.next()`, but now its empty. In any case, the product is
                    // empty now and we should handle things accordingly.
                    self.state = Unstarted;
                    return None;
                }
            }
        }

        // Reaching here indicates that all the iterators returned none, and so iteration has completed
        let v = core::mem::replace(last, Vec::new());
        self.state = Restarted(v);
        None
    }

    fn count(self) -> usize {
        // `remaining` is the number of remaining iterations before the current iterator is
        // exhausted. `per_reset` is the number of total iterations that take place each time the
        // current iterator is reset
        let (remaining, per_reset) =
            self.iters
                .into_iter()
                .rev()
                .fold((0, 1), |(remaining, per_reset), iter| {
                    let remaining = remaining + per_reset * iter.iter.count();
                    let per_reset = per_reset * iter.iter_orig.count();
                    (remaining, per_reset)
                });
        if let Restarted(_) | Unstarted = &self.state {
            per_reset
        } else {
            remaining
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let initial = ((0, Some(0)), (1, Some(1)));
        // Exact same logic as for `count`
        let (remaining, per_reset) =
            self.iters
                .iter()
                .rev()
                .fold(initial, |(remaining, per_reset), iter| {
                    let prod = size_hint::mul(per_reset, iter.iter.size_hint());
                    let remaining = size_hint::add(remaining, prod);
                    let per_reset = size_hint::mul(per_reset, iter.iter_orig.size_hint());
                    (remaining, per_reset)
                });
        if let Restarted(_) | Unstarted = &self.state {
            per_reset
        } else {
            remaining
        }
    }

    fn last(self) -> Option<Self::Item> {
        // The way resetting works makes the first iterator a little bit special
        let mut iter = self.iters.into_iter();
        if let Some(first) = iter.next() {
            let first = if let Restarted(_) | Unstarted = &self.state {
                first.iter_orig.last()
            } else {
                first.iter.last()
            };
            core::iter::once(first)
                .chain(iter.map(|sub| sub.iter_orig.last()))
                .collect()
        } else {
            if let Restarted(_) | Unstarted = &self.state {
                Some(Vec::new())
            } else {
                None
            }
        }
    }
}
