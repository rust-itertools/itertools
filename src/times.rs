
/// Return an iterator with `n` elements, for simple repetition
/// a particular number of times. The iterator yields a counter.
///
/// Iterator element type is `uint`
#[inline]
pub fn times(n: uint) -> Times
{
    Times{i: 0, n: n}
}

/// Iterator to repeat a simple number of times
///
/// Created with the `times()` function.
///
/// Iterator element type is `uint`
#[deriving(Clone)]
pub struct Times {
    i: uint,
    n: uint,
}

impl Iterator<uint> for Times
{
    #[inline]
    fn next(&mut self) -> Option<uint>
    {
        let elt = self.i;
        if self.i < self.n {
            self.i += 1;
            Some(elt)
        } else { None }
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>)
    {
        let len = self.n - self.i;
        (len, Some(len))
    }
}

impl DoubleEndedIterator<uint> for Times
{
    #[inline]
    fn next_back(&mut self) -> Option<uint>
    {
        if self.i < self.n {
            self.n -= 1;
            Some(self.n)
        } else { None }
    }
}

impl ExactSizeIterator<uint> for Times { }
