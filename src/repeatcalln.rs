
/// An iterator that produces *n* elements by calling a given closure.
///
/// See [`repeat_call_n()`](../fn.repeat_call_n.html) for more information.
pub struct RepeatCallN<F> {
    f: Option<F>,
    n: usize,
}

/// An iterator that produces `n` elements by calling a given closure.
///
/// ```
/// use itertools::repeat_call_n;
/// use itertools::Itertools;
/// use std::collections::BinaryHeap;
///
/// let mut heap = BinaryHeap::from(vec![2, 5, 3, 7, 8]);
///
/// // extract first three elements in sorted order
/// for element in repeat_call_n(|| heap.pop(), 3).while_some() {
///     print!("{}", element);
/// }
///
/// itertools::assert_equal(
///     repeat_call_n(|| 1, 3),
///     vec![1, 1, 1]
/// );
/// ```
pub fn repeat_call_n<F>(element: F, n: usize) -> RepeatCallN<F> {
    if n == 0 {
        RepeatCallN { f: None, n: n, }
    } else {
        RepeatCallN { f: Some(element), n: n, }
    }
}

impl<A, F> Iterator for RepeatCallN<F>
    where F: FnMut() -> A
{
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        if self.n > 1 {
            self.n -= 1;
            self.f.as_mut().map(|f| f())
        } else {
            self.n = 0;
            self.f.take().map(|mut f| f())
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.n, Some(self.n))
    }
}

impl<A, F> DoubleEndedIterator for RepeatCallN<F>
    where F: FnMut() -> A
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

impl<A, F> ExactSizeIterator for RepeatCallN<F>
    where F: FnMut() -> A
{}
