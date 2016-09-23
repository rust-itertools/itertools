//! Iterators that are sources (produce elements from parameters,
//! not from another iterator).

/// See [`repeat_call`](../fn.repeat_call.html) for more information.
pub struct RepeatCall<F> {
    f: F,
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
pub fn repeat_call<F>(function: F) -> RepeatCall<F> {
    RepeatCall { f: function }
}

impl<A, F> Iterator for RepeatCall<F>
    where F: FnMut() -> A
{
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<A> {
        Some((self.f)())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::max_value(), None)
    }
}

/// Creates a new unfold source with the specified closure as the "iterator
/// function" and an initial state to eventually pass to the closure
///
/// `unfold` is a general iterator builder: it has a mutable state value,
/// and a closure with access to the state that produces the next value.
///
/// This more or less equivalent to a regular struct with an `Iterator`
/// implementation, and is useful for one-off iterators.
///
/// ```
/// // an iterator that yields sequential Fibonacci numbers,
/// // and stops at the maximum representable value.
///
/// use itertools::unfold;
///
/// let mut fibonacci = unfold((1_u32, 1_u32), |state| {
///     let (ref mut x1, ref mut x2) = *state;
///
///     // Attempt to get the next Fibonacci number
///     let next = x1.saturating_add(*x2);
///
///     // Shift left: ret <- x1 <- x2 <- next
///     let ret = *x1;
///     *x1 = *x2;
///     *x2 = next;
///
///     // If addition has saturated at the maximum, we are finished
///     if ret == *x1 && ret > 1 {
///         return None;
///     }
///
///     Some(ret)
/// });
///
/// itertools::assert_equal(fibonacci.by_ref().take(8),
///                         vec![1, 1, 2, 3, 5, 8, 13, 21]);
/// assert_eq!(fibonacci.last(), Some(2_971_215_073))
/// ```
pub fn unfold<A, St, F>(initial_state: St, f: F) -> Unfold<St, F>
    where F: FnMut(&mut St) -> Option<A>
{
    Unfold {
        f: f,
        state: initial_state,
    }
}


/// See [`unfold`](../fn.unfold.html) for more information.
#[derive(Clone)]
pub struct Unfold<St, F> {
    f: F,
    /// Internal state that will be passed to the closure on the next iteration
    pub state: St,
}

impl<A, St, F> Iterator for Unfold<St, F>
    where F: FnMut(&mut St) -> Option<A>
{
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<A> {
        (self.f)(&mut self.state)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // no possible known bounds at this point
        (0, None)
    }
}
