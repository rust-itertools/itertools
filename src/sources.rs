//! Iterators that are sources (produce elements from parameters,
//! not from another iterator).

use std::fmt;
use std::mem;

/// See [`repeat_call`](../fn.repeat_call.html) for more information.
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
/// let (mut x1, mut x2) = (1u32, 1u32);
/// let mut fibonacci = unfold((), move |_| {
///     // Attempt to get the next Fibonacci number
///     let next = x1.saturating_add(x2);
///
///     // Shift left: ret <- x1 <- x2 <- next
///     let ret = x1;
///     x1 = x2;
///     x2 = next;
///
///     // If addition has saturated at the maximum, we are finished
///     if ret == x1 && ret > 1 {
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

impl<St, F> fmt::Debug for Unfold<St, F>
    where St: fmt::Debug,
{
    debug_fmt_fields!(Unfold, state);
}

/// See [`unfold`](../fn.unfold.html) for more information.
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
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

/// Returns an iterator that starts with the `seed` and repeatedly
/// calls the `step` function to produce the next value, until it
/// returns `None`. The resulting sequence always has at least one
/// element -- the `seed`. To produce a possibly empty sequence,
/// use `Generate::new`.
///
/// ```
/// // an iterator that yields Collatz sequence.
/// // https://en.wikipedia.org/wiki/Collatz_conjecture
///
/// use itertools::generate;
///
/// let collatz = generate(12, |&n| {
///     if n == 1 {
///         None
///     } else if n % 2 == 0 {
///         Some(n / 2)
///     } else {
///         Some(3 * n + 1)
///     }
/// });
///
/// itertools::assert_equal(collatz,
///                         vec![12, 6, 3, 10, 5, 16, 8, 4, 2, 1]);
/// ```
pub fn generate<T, F>(seed: T, step: F) -> Generate<T, F>
    where F: FnMut(&T) -> Option<T>
{
    Generate::new(Some(seed), step)
}

impl<T, F> fmt::Debug for Generate<T, F>
    where T: fmt::Debug,
{
    debug_fmt_fields!(Genrate, next);
}

/// See [`generate`](../fn.generate.html) for more information.
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Generate<T, F> {
    next: Option<T>,
    step: F,
}

impl<T, F> Generate<T, F>
    where F: FnMut(&T) -> Option<T>
{
    /// Like [`generate`](../fn.generate.html), but allows for
    /// empty sequences.
    pub fn new(seed: Option<T>, step: F) -> Generate<T, F> {
        Generate {
            next: seed,
            step: step,
        }
    }
}

impl<T, F> Iterator for Generate<T, F>
    where F: FnMut(&T) -> Option<T>
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.next.take().map(|next| {
            self.next = (self.step)(&next);
            next
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.next.is_some() as usize, None)
    }
}

/// An iterator that infinitely applies function to value and yields results.
///
/// This `struct` is created by the [`iterate()`] function. See its documentation for more.
///
/// [`iterate()`]: ../fn.iterate.html
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
        f: f,
    }
}
