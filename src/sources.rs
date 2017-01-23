//! Iterators that are sources (produce elements from parameters,
//! not from another iterator).

use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ptr;
use nodrop::NoDrop;

#[doc(hidden)]
pub unsafe trait FixedSizeArray<T>: AsRef<[T]> {}

macro_rules! impl_fixed_size_array {
    ($($N:expr),*) => (
        $(unsafe impl<T> FixedSizeArray<T> for [T; $N] {})*
    )
}

impl_fixed_size_array!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
                       18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32);

/// Create an iterator over a fixed number of elements. See [`iter!`](../macro.iter.html)
/// for more information.
pub struct ListIterator<T, A: FixedSizeArray<T>> {
    array: NoDrop<A>,
    cur: usize,
    _marker: PhantomData<T>,
}

#[doc(hidden)]
pub fn list_iterator<T, A: FixedSizeArray<T>>(a: A) -> ListIterator<T, A> {
    ListIterator {
        array: NoDrop::new(a),
        cur: 0,
        _marker: PhantomData,
    }
}

impl<T, A: FixedSizeArray<T>> Iterator for ListIterator<T, A> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = &mut self.cur;
        self.array.as_ref().get(*cur).map(|v| {
            *cur += 1;
            unsafe { ptr::read(v) }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let s = self.array.as_ref().len() - self.cur;
        (s, Some(s))
    }
}

impl<T, A: FixedSizeArray<T>> ExactSizeIterator for ListIterator<T, A> {}

impl<T, A: FixedSizeArray<T>> Drop for ListIterator<T, A> {
    fn drop(&mut self) {
        for _ in self {}
    }
}


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

impl<St, F> fmt::Debug for Unfold<St, F>
    where St: fmt::Debug,
{
    debug_fmt_fields!(Unfold, state);
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

/// An iterator that infinitely applies function to value and yields results.
///
/// This `struct` is created by the [`iterate()`] function. See its documentation for more.
///
/// [`iterate()`]: ../fn.iterate.html
#[derive(Clone)]
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
