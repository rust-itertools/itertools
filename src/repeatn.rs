
/// An iterator that repeats an element exactly *n* times.
pub struct RepeatN<A>
{
    elt: Option<A>,
    n: usize,
}

impl<A> RepeatN<A>
{
    /// Create a new **RepeatN** with **n** repetitions.
    pub fn new(elt: A, n: usize) -> Self
    {
        if n == 0 {
            RepeatN{elt: None, n: n}
        } else {
            RepeatN{elt: Some(elt), n: n}
        }
    }
}

impl<A> Iterator for RepeatN<A> where
    A: Clone,
{
    type Item = A;

    fn next(&mut self) -> Option<Self::Item>
    {
        if self.n > 1 {
            self.n -= 1;
            self.elt.as_ref().cloned()
        } else {
            self.n = 0;
            self.elt.take()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        (self.n, Some(self.n))
    }
}

impl<A> DoubleEndedIterator for RepeatN<A> where
    A: Clone,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item>
    {
        self.next()
    }
}

impl<A> ExactSizeIterator for RepeatN<A> where
    A: Clone,
{
}
