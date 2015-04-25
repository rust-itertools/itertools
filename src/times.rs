
/// Return an iterator with `n` elements, for simple repetition
/// a particular number of times. The iterator yields a counter.
///
/// Iterator element type is `usize`
#[inline]
pub fn times(n: usize) -> Times
{
    Times{i: 0, n: n}
}

/// A simple iterator to repeat a given number of times
///
/// Created with the `times()` function.
///
/// Iterator element type is `usize`
#[derive(Copy, Clone)]
pub struct Times {
    i: usize,
    n: usize,
}

impl Iterator for Times
{
    type Item = usize;
    #[inline]
    fn next(&mut self) -> Option<usize>
    {
        let elt = self.i;
        if self.i < self.n {
            self.i += 1;
            Some(elt)
        } else { None }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>)
    {
        let len = self.n - self.i;
        (len, Some(len))
    }
}

impl DoubleEndedIterator for Times
{
    #[inline]
    fn next_back(&mut self) -> Option<usize>
    {
        if self.i < self.n {
            self.n -= 1;
            Some(self.n)
        } else { None }
    }
}

impl ExactSizeIterator for Times { }
