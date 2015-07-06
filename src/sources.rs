//! Iterators that are sources (produce elements from parameters,
//! not from another iterator).

/// An iterator source that produces elements indefinitely by calling
/// a given closure.
///
/// Iterator element type is the return type of the closure.
///
/// ```
/// use itertools::RepeatCall;
///
/// assert!(itertools::equal(
///     RepeatCall::new(|| "A".to_string()).take(5),
///     vec!["A", "A", "A", "A", "A"]
/// ));
///
/// let mut x = 1;
/// assert!(itertools::equal(
///     RepeatCall::new(|| { x = -x; x }).take(5),
///     vec![-1, 1, -1, 1, -1]
/// ));
/// ```
pub struct RepeatCall<F> {
    f: F,
}

impl<F> RepeatCall<F>
{
    /// Create a new `RepeatCall` from a closure.
    pub fn new<A>(func: F) -> Self where
        F: FnMut() -> A,
    {
        RepeatCall { f: func }
    }
}

impl<A, F> Iterator for RepeatCall<F> where
    F: FnMut() -> A,
{
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<A>
    {
        Some((self.f)())
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        (usize::max_value(), None)
    }
}

impl<A, F> DoubleEndedIterator for RepeatCall<F> where
    F: FnMut() -> A,
{
    #[inline]
    fn next_back(&mut self) -> Option<A> { self.next() }
}


/// `Unfold` is a general iterator builder: it has a mutable state value,
/// and a closure with access to the state that produces the next value.
///
/// This more or less equivalent to a regular struct with an `Iterator`
/// implementation, and is useful for one-off iterators.
///
/// ```
/// // an iterator that yields sequential Fibonacci numbers,
/// // and stops at the maximum representable value.
///
/// use itertools::Unfold;
///
/// let mut fibonacci = Unfold::new((1_u32, 1_u32), |state| {
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
#[derive(Clone)]
pub struct Unfold<St, F> {
    f: F,
    /// Internal state that will be passed to the closure on the next iteration
    pub state: St,
}

impl<A, St, F> Unfold<St, F>
    where F: FnMut(&mut St) -> Option<A>
{
    /// Creates a new iterator with the specified closure as the "iterator
    /// function" and an initial state to eventually pass to the closure
    #[inline]
    pub fn new(initial_state: St, f: F) -> Unfold<St, F> {
        Unfold {
            f: f,
            state: initial_state
        }
    }
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

