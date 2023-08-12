use alloc::vec::Vec;
use std::fmt;
use std::iter::once;

use super::lazy_buffer::LazyBuffer;
use super::size_hint::{self, SizeHint};

/// An iterator adaptor that iterates through all the `k`-permutations of the
/// elements from an iterator.
///
/// See [`.permutations()`](crate::Itertools::permutations) for
/// more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Permutations<I: Iterator> {
    vals: LazyBuffer<I>,
    state: PermutationState,
}

impl<I> Clone for Permutations<I>
    where I: Clone + Iterator,
          I::Item: Clone,
{
    clone_fields!(vals, state);
}

#[derive(Clone, Debug)]
enum PermutationState {
    StartUnknownLen {
        k: usize,
    },
    OngoingUnknownLen {
        k: usize,
        min_n: usize,
    },
    Complete(CompleteState),
    Empty,
}

#[derive(Clone, Debug)]
enum CompleteState {
    Start {
        n: usize,
        k: usize,
    },
    Ongoing {
        indices: Vec<usize>,
        cycles: Vec<usize>,
    }
}

impl<I> fmt::Debug for Permutations<I>
    where I: Iterator + fmt::Debug,
          I::Item: fmt::Debug,
{
    debug_fmt_fields!(Permutations, vals, state);
}

pub fn permutations<I: Iterator>(iter: I, k: usize) -> Permutations<I> {
    let mut vals = LazyBuffer::new(iter);

    if k == 0 {
        // Special case, yields single empty vec; `n` is irrelevant
        let state = PermutationState::Complete(CompleteState::Start { n: 0, k: 0 });

        return Permutations {
            vals,
            state
        };
    }

    vals.prefill(k);
    let enough_vals = vals.len() == k;

    let state = if enough_vals {
        PermutationState::StartUnknownLen { k }
    } else {
        PermutationState::Empty
    };

    Permutations {
        vals,
        state
    }
}

impl<I> Iterator for Permutations<I>
where
    I: Iterator,
    I::Item: Clone
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance();

        let &mut Permutations { ref vals, ref state } = self;

        match *state {
            PermutationState::StartUnknownLen { .. } => panic!("unexpected iterator state"),
            PermutationState::OngoingUnknownLen { k, min_n } => {
                let latest_idx = min_n - 1;
                let indices = (0..(k - 1)).chain(once(latest_idx));

                Some(indices.map(|i| vals[i].clone()).collect())
            }
            PermutationState::Complete(CompleteState::Ongoing { ref indices, ref cycles }) => {
                let k = cycles.len();
                Some(indices[0..k].iter().map(|&i| vals[i].clone()).collect())
            },
            PermutationState::Complete(CompleteState::Start { .. }) | PermutationState::Empty => None
        }
    }

    fn count(self) -> usize {
        let Permutations { vals, state } = self;
        match state {
            PermutationState::StartUnknownLen { k } => {
                let n = vals.len() + vals.it.count();
                CompleteState::Start { n, k }.count()
            }
            PermutationState::OngoingUnknownLen { k, min_n } => {
                let prev_iteration_count = min_n - k + 1;
                let n = vals.len() + vals.it.count();
                CompleteState::Start { n, k }.count() - prev_iteration_count
            },
            PermutationState::Complete(state) => state.count(),
            PermutationState::Empty => 0
        }
    }

    fn size_hint(&self) -> SizeHint {
        match self.state {
            // Note: the product for `CompleteState::Start` in `remaining` increases with `n`.
            PermutationState::StartUnknownLen { k } => {
                size_hint::try_map(
                    self.vals.size_hint(),
                    |n| CompleteState::Start { n, k }.remaining(),
                )
            }
            PermutationState::OngoingUnknownLen { k, min_n } => {
                let prev_iteration_count = min_n - k + 1;
                size_hint::try_map(self.vals.size_hint(), |n| {
                    CompleteState::Start { n, k }
                        .remaining()
                        .and_then(|count| count.checked_sub(prev_iteration_count))
                })
            }
            PermutationState::Complete(ref state) => match state.remaining() {
                Some(count) => (count, Some(count)),
                None => (::std::usize::MAX, None)
            }
            PermutationState::Empty => (0, Some(0))
        }
    }
}

impl<I> Permutations<I>
where
    I: Iterator,
    I::Item: Clone
{
    fn advance(&mut self) {
        let &mut Permutations { ref mut vals, ref mut state } = self;

        *state = match *state {
            PermutationState::StartUnknownLen { k } => {
                PermutationState::OngoingUnknownLen { k, min_n: k }
            }
            PermutationState::OngoingUnknownLen { k, min_n } => {
                if vals.get_next() {
                    PermutationState::OngoingUnknownLen { k, min_n: min_n + 1 }
                } else {
                    let n = min_n;
                    let prev_iteration_count = n - k + 1;
                    let mut complete_state = CompleteState::Start { n, k };

                    // Advance the complete-state iterator to the correct point
                    for _ in 0..=prev_iteration_count {
                        complete_state.advance();
                    }

                    PermutationState::Complete(complete_state)
                }
            }
            PermutationState::Complete(ref mut state) => {
                state.advance();

                return;
            }
            PermutationState::Empty => { return; }
        };
    }
}

impl CompleteState {
    fn advance(&mut self) {
        *self = match *self {
            CompleteState::Start { n, k } => {
                let indices = (0..n).collect();
                let cycles = ((n - k)..n).rev().collect();

                CompleteState::Ongoing {
                    cycles,
                    indices
                }
            },
            CompleteState::Ongoing { ref mut indices, ref mut cycles } => {
                let n = indices.len();
                let k = cycles.len();

                for i in (0..k).rev() {
                    if cycles[i] == 0 {
                        cycles[i] = n - i - 1;

                        let to_push = indices.remove(i);
                        indices.push(to_push);
                    } else {
                        let swap_index = n - cycles[i];
                        indices.swap(i, swap_index);

                        cycles[i] -= 1;
                        return;
                    }
                }

                CompleteState::Start { n, k }
            }
        }
    }

    /// The remaining count of elements, if it does not overflow.
    fn remaining(&self) -> Option<usize> {
        match *self {
            CompleteState::Start { n, k } => {
                if n < k {
                    return Some(0);
                }
                (n - k + 1..n + 1).fold(Some(1), |acc, i| {
                    acc.and_then(|acc| acc.checked_mul(i))
                })
            }
            CompleteState::Ongoing { ref indices, ref cycles } => {
                cycles.iter().enumerate().fold(Some(0), |acc, (i, c)| {
                    acc.and_then(|count| {
                        let radix = indices.len() - i;
                        count.checked_mul(radix)?.checked_add(*c)
                    })
                })
            }
        }
    }

    /// The remaining count of elements, panics if it overflows.
    fn count(&self) -> usize {
        self.remaining().expect("Iterator count greater than usize::MAX")
    }
}
