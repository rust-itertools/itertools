//! Iterators that are sources (produce elements from parameters,
//! not from another iterator).
#![allow(deprecated)]

use std::fmt;
use std::mem;

/// See [`repeat_call`](crate::repeat_call) for more information.
#[derive(Clone)]
#[deprecated(note="Use std repeat_with() instead", since="0.8.0")]
pub struct RepeatCall<F> {
    f: F,
}

impl<F> fmt::Debug for RepeatCall<F>
{
    debug_fmt_fields!(RepeatCall, );
}

/// An iterator source that produces elements indefinitely by calling
/// a given closure.
///
/// Iterator element type is the return type of the closure.
///
/// ```
/// use itertools::repeat_call;
/// use itertools::Itertools;
/// use std::collections::BinaryHeap;
///
/// let mut heap = BinaryHeap::from(vec![2, 5, 3, 7, 8]);
///
/// // extract each element in sorted order
/// for element in repeat_call(|| heap.pop()).while_some() {
///     print!("{}", element);
/// }
///
/// itertools::assert_equal(
///     repeat_call(|| 1).take(5),
///     vec![1, 1, 1, 1, 1]
/// );
/// ```
#[deprecated(note="Use std repeat_with() instead", since="0.8.0")]
pub fn repeat_call<F, A>(function: F) -> RepeatCall<F>
    where F: FnMut() -> A
{
    RepeatCall { f: function }
}

impl<A, F> Iterator for RepeatCall<F>
    where F: FnMut() -> A
{
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some((self.f)())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::max_value(), None)
    }
}

/// An iterator that infinitely applies function to value and yields results.
///
/// This `struct` is created by the [`iterate()`](crate::iterate) function.
/// See its documentation for more.
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Iterate<St, F> {
    state: St,
    f: F,
}

impl<St, F> fmt::Debug for Iterate<St, F>
    where St: fmt::Debug,
{
    debug_fmt_fields!(Iterate, state);
}

impl<St, F> Iterator for Iterate<St, F>
    where F: FnMut(&St) -> St
{
    type Item = St;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next_state = (self.f)(&self.state);
        Some(mem::replace(&mut self.state, next_state))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::max_value(), None)
    }
}

/// Creates a new iterator that infinitely applies function to value and yields results.
///
/// ```
/// use itertools::iterate;
///
/// itertools::assert_equal(iterate(1, |&i| i * 3).take(5), vec![1, 3, 9, 27, 81]);
/// ```
pub fn iterate<St, F>(initial_value: St, f: F) -> Iterate<St, F>
    where F: FnMut(&St) -> St
{
    Iterate {
        state: initial_value,
        f,
    }
}
