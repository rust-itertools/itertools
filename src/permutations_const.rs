use alloc::boxed::Box;
use alloc::vec::Vec;
use std::fmt;
use std::iter::once;
use std::iter::FusedIterator;
use std::convert::TryInto;

use super::lazy_buffer::LazyBuffer;
use crate::size_hint::{self, SizeHint};

/// An iterator adaptor that iterates through all the `k`-permutations of the
/// elements from an iterator.
///
/// See [`.permutations()`](crate::Itertools::permutations) for
/// more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct PermutationsConst<I: Iterator, const K: usize> {
    vals: LazyBuffer<I>,
    state: PermutationState<K>,
}

impl<I, const K: usize> Clone for PermutationsConst<I, K>
where
    I: Clone + Iterator,
    I::Item: Clone,
{
    clone_fields!(vals, state);
}

#[derive(Clone, Debug)]
enum PermutationState<const K: usize> {
    /// No permutation generated yet.
    Start,
    /// Values from the iterator are not fully loaded yet so `n` is still unknown.
    Buffered { min_n: usize },
    /// All values from the iterator are known so `n` is known.
    Loaded {
        indices: Box<[usize]>,
        cycles: Box<[usize]>,
    },
    /// No permutation left to generate.
    End,
}

impl<I, const K: usize> fmt::Debug for PermutationsConst<I, K>
where
    I: Iterator + fmt::Debug,
    I::Item: fmt::Debug,
{
    debug_fmt_fields!(Permutations, vals, state);
}

pub fn permutations_const<I: Iterator, const K: usize>(iter: I) -> PermutationsConst<I, K> {
    PermutationsConst {
        vals: LazyBuffer::new(iter),
        state: PermutationState::Start,
    }
}

impl<I, const K: usize> Iterator for PermutationsConst<I, K>
where
    I: Iterator,
    I::Item: Clone + Default,
{
    type Item = [I::Item; K];

    fn next(&mut self) -> Option<Self::Item> {
        let Self { vals, state } = self;
        match state {
            PermutationState::Start => {
                *state = PermutationState::End;
                // TODO: Consider this case and handle it somehow, currently just using default
                Some(std::array::from_fn(|_|I::Item::default()))
            }
            &mut PermutationState::Start => {
                vals.prefill(K);
                if vals.len() != K {
                    *state = PermutationState::End;
                    return None;
                }
                *state = PermutationState::Buffered { min_n: K };
                let mut iter = vals[0..K].into_iter().cloned();
                Some(std::array::from_fn(|_|iter.next().unwrap())) // TODO: Handle error case, maybe make this better
            }
            PermutationState::Buffered { min_n } => {
                if vals.get_next() {
                    let mut item = (0..K - 1)
                        .chain(once(*min_n))
                        .map(|i| vals[i].clone());
                    *min_n += 1;
                    Some(std::array::from_fn(|_|item.next().unwrap()))
                } else {
                    let n = *min_n;
                    let prev_iteration_count = n - K + 1;
                    let mut indices: Box<[_]> = (0..n).collect();
                    let mut cycles: Box<[_]> = (n - K..n).rev().collect();
                    // Advance the state to the correct point.
                    for _ in 0..prev_iteration_count {
                        if advance(&mut indices, &mut cycles) {
                            *state = PermutationState::End;
                            return None;
                        }
                    }
                    let item = vals.get_at(&indices[0..K]); // TODO: Impl const sized variant otherwise this is pointless
                    *state = PermutationState::Loaded { indices, cycles };
                    Some(item.try_into().ok()?) // TODO: Handle error case
                }
            }
            PermutationState::Loaded { indices, cycles } => {
                if advance(indices, cycles) {
                    *state = PermutationState::End;
                    return None;
                }
                let k = cycles.len();
                Some(vals.get_at(&indices[0..k]).try_into().ok()?) // TODO: Handle error case and const size indexing
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

impl<I, const K: usize> FusedIterator for PermutationsConst<I, K>
where
    I: Iterator,
    I::Item: Clone + Default,
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

impl <const K: usize>PermutationState<K> {
    fn size_hint_for(&self, n: usize) -> SizeHint {
        // At the beginning, there are `n!/(n-k)!` items to come.
        let at_start = |n, k| {
            debug_assert!(n >= k);
            let total = (n - k + 1..=n).try_fold(1usize, |acc, i| acc.checked_mul(i));
            (total.unwrap_or(usize::MAX), total)
        };
        match *self {
            Self::Start if n < K => (0, Some(0)),
            Self::Start => at_start(n, K),
            Self::Buffered { min_n } => {
                // Same as `Start` minus the previously generated items.
                size_hint::sub_scalar(at_start(n, K), min_n - K + 1)
            }
            Self::Loaded {
                ref indices,
                ref cycles,
            } => {
                let count = cycles.iter().enumerate().try_fold(0usize, |acc, (i, &c)| {
                    acc.checked_mul(indices.len() - i)
                        .and_then(|count| count.checked_add(c))
                });
                (count.unwrap_or(usize::MAX), count)
            }
            Self::End => (0, Some(0)),
        }
    }
}
