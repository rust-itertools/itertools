//! Iterators that are sources (produce elements from parameters,
//! not from another iterator).

/// An iterator source that produces elements indefinitely by calling
/// a given closure.
///
/// Iterator element type is the return type of the closure.
///
/// ## Example
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
    /// Create a new **RepeatCall** from a closure.
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


