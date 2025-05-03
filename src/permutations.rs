use alloc::boxed::Box;
use std::fmt;
use std::iter::FusedIterator;

use super::lazy_buffer::{ArrayOrVecHelper, LazyBuffer, MaybeConstUsize};
use crate::size_hint::{self, SizeHint};

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct PermutationsGeneric<I: Iterator, Idx: ArrayOrVecHelper> {
    vals: LazyBuffer<I>,
    state: PermutationState<Idx>,
}

/// An iterator adaptor that iterates through all the `k`-permutations of the
/// elements from an iterator.
///
/// See [`.permutations()`](crate::Itertools::permutations) for
/// more information.
pub type Permutations<I> = PermutationsGeneric<I, Box<[usize]>>;

impl<I, Idx> Clone for PermutationsGeneric<I, Idx>
where
    I: Clone + Iterator,
    I::Item: Clone,
    Idx: Clone + ArrayOrVecHelper,
{
    clone_fields!(vals, state);
}

#[derive(Clone, Debug)]
enum PermutationState<Idx: ArrayOrVecHelper> {
    /// No permutation generated yet.
    Start { k: Idx::Length },
    /// Values from the iterator are not fully loaded yet so `n` is still unknown.
    Buffered { indices: Idx, min_n: usize },
    /// All values from the iterator are known so `n` is known.
    Loaded { indices: Box<[usize]>, cycles: Idx },
    /// No permutation left to generate.
    End,
}

impl<I, Idx: ArrayOrVecHelper> fmt::Debug for PermutationsGeneric<I, Idx>
where
    I: Iterator + fmt::Debug,
    I::Item: fmt::Debug,
    Idx: fmt::Debug,
{
    debug_fmt_fields!(PermutationsGeneric, vals, state);
}

pub fn permutations<I: Iterator>(iter: I, k: usize) -> Permutations<I> {
    Permutations {
        vals: LazyBuffer::new(iter),
        state: PermutationState::Start { k },
    }
}

impl<I, Idx: ArrayOrVecHelper> Iterator for PermutationsGeneric<I, Idx>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = Idx::Item<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let Self { vals, state } = self;
        match state {
            &mut PermutationState::Start { k } => {
                if k.value() == 0 {
                    *state = PermutationState::End;
                } else {
                    vals.prefill(k.value());
                    if vals.len() != k.value() {
                        *state = PermutationState::End;
                        return None;
                    }
                    *state = PermutationState::Buffered {
                        indices: Idx::start(k),
                        min_n: k.value(),
                    };
                }
                Some(Idx::item_from_fn(k, |i| vals[i].clone()))
            }
            PermutationState::Buffered { indices, min_n } => {
                let k = indices.len();
                if vals.get_next() {
                    // This could be an unwrap instead of ?, indices.len() should be
                    // k.value() > 0 to be in this case
                    *indices.borrow_mut().last_mut()? += 1;
                    debug_assert!(indices
                        .borrow()
                        .iter()
                        .copied()
                        .eq((0..k.value() - 1).chain([*min_n])));
                    *min_n += 1;
                    Some(indices.extract_item(vals))
                } else {
                    let n = *min_n;
                    let prev_iteration_count = n - k.value() + 1;
                    let mut indices: Box<[_]> = (0..n).collect();
                    let mut cycles = Idx::from_fn(k, |i| n - 1 - i);
                    // Advance the state to the correct point.
                    for _ in 0..prev_iteration_count {
                        if advance(&mut indices, cycles.borrow_mut()) {
                            *state = PermutationState::End;
                            return None;
                        }
                    }
                    let item = Idx::item_from_fn(k, |i| vals[indices[i]].clone());
                    *state = PermutationState::Loaded { indices, cycles };
                    Some(item)
                }
            }
            PermutationState::Loaded { indices, cycles } => {
                if advance(indices, cycles.borrow_mut()) {
                    *state = PermutationState::End;
                    return None;
                }
                Some(Idx::item_from_fn(cycles.len(), |i| {
                    vals[indices[i]].clone()
                }))
            }
            PermutationState::End => None,
        }
    }

    fn count(self) -> usize {
        let Self { vals, state } = self;
        let n = vals.count();
        state.size_hint_for(n).1.unwrap()
    }

    fn size_hint(&self) -> SizeHint {
        let (mut low, mut upp) = self.vals.size_hint();
        low = self.state.size_hint_for(low).0;
        upp = upp.and_then(|n| self.state.size_hint_for(n).1);
        (low, upp)
    }
}

impl<I, Idx: ArrayOrVecHelper> FusedIterator for PermutationsGeneric<I, Idx>
where
    I: Iterator,
    I::Item: Clone,
{
}

fn advance(indices: &mut [usize], cycles: &mut [usize]) -> bool {
    let n = indices.len();
    let k = cycles.len();
    // NOTE: if `cycles` are only zeros, then we reached the last permutation.
    for i in (0..k).rev() {
        if cycles[i] == 0 {
            cycles[i] = n - i - 1;
            indices[i..].rotate_left(1);
        } else {
            let swap_index = n - cycles[i];
            indices.swap(i, swap_index);
            cycles[i] -= 1;
            return false;
        }
    }
    true
}

impl<Idx: ArrayOrVecHelper> PermutationState<Idx> {
    fn size_hint_for(&self, n: usize) -> SizeHint {
        // At the beginning, there are `n!/(n-k)!` items to come.
        let at_start = |n, k: Idx::Length| {
            debug_assert!(n >= k.value());
            let total = (n - k.value() + 1..=n).try_fold(1usize, |acc, i| acc.checked_mul(i));
            (total.unwrap_or(usize::MAX), total)
        };
        match self {
            Self::Start { k } if n < k.value() => (0, Some(0)),
            Self::Start { k } => at_start(n, *k),
            Self::Buffered { indices, min_n } => {
                let k = indices.len();
                // Same as `Start` minus the previously generated items.
                size_hint::sub_scalar(at_start(n, k), min_n - k.value() + 1)
            }
            Self::Loaded {
                ref indices,
                ref cycles,
            } => {
                let count = cycles
                    .borrow()
                    .iter()
                    .enumerate()
                    .try_fold(0usize, |acc, (i, &c)| {
                        acc.checked_mul(indices.len() - i)
                            .and_then(|count| count.checked_add(c))
                    });
                (count.unwrap_or(usize::MAX), count)
            }
            Self::End => (0, Some(0)),
        }
    }
}
